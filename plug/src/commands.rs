use std::{collections::HashMap, sync::Arc};

use buttplug::client::{ButtplugClient, ButtplugClientDevice, ScalarValueCommand, ButtplugClientError};
use tokio::{runtime::Handle, sync::mpsc::unbounded_channel};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, span, trace, warn, Level};

use crate::{
    event::TkEvent,
    pattern::TkPatternPlayer,
    settings::{TkDeviceSettings, TkSettings},
    Speed, TkPattern,
};

type DeviceNameList = Box<Vec<String>>;

#[derive(Clone, Debug)]
pub enum TkAction {
    Scan,
    StopScan,
    Control(i32, TkParams),
    Stop(i32),
    StopAll,
    Disconect,
}

#[derive(Clone, Debug)]
pub struct TkParams {
    pub selector: TkDeviceSelector,
    pub pattern: TkPattern,
}

impl TkParams {
    pub fn filter_devices(
        &self,
        devices: Vec<Arc<ButtplugClientDevice>>,
    ) -> Vec<Arc<ButtplugClientDevice>> {
        // always assumes vibration for now
        self.selector
            .filter_devices(devices)
            .iter()
            .filter(|d| d.message_attributes().scalar_cmd().is_some())
            .map(|d| d.clone())
            .collect()
    }
}

#[derive(Clone, Debug)]
pub enum TkDeviceSelector {
    All,
    ByNames(DeviceNameList),
}

impl TkDeviceSelector {
    pub fn filter_devices(
        &self,
        devices: Vec<Arc<ButtplugClientDevice>>,
    ) -> Vec<Arc<ButtplugClientDevice>> {
        devices
            .iter()
            .filter(|d| match self {
                TkDeviceSelector::All => true,
                TkDeviceSelector::ByNames(names) => {
                    let matches = names.iter().any(|x| x == d.name());
                    matches
                }
            })
            .map(|d| d.clone())
            .collect()
    }

    pub fn from_events(events: Vec<String>, settings: &Vec<TkDeviceSettings>) -> TkDeviceSelector {
        TkDeviceSelector::ByNames(Box::new(
            settings
                .iter()
                .filter(|d| {
                    d.enabled && (events.len() == 0 || d.events.iter().any(|e| events.contains(e)))
                })
                .map(|d| d.name.clone())
                .collect(),
        ))
    }
}

impl From<&TkSettings> for TkDeviceSelector {
    fn from(settings: &TkSettings) -> Self {
        TkDeviceSelector::ByNames(Box::new(
            settings
                .get_enabled_devices()
                .iter()
                .map(|d| d.name.clone())
                .collect(),
        ))
    }
}

#[derive(Clone, Debug)]
pub enum TkDeviceAction {
    Start(Arc<ButtplugClientDevice>, Speed, bool, i32),
    Update(Arc<ButtplugClientDevice>, Speed),
    End(Arc<ButtplugClientDevice>, bool, i32),
    StopAll, // global but required for resetting device state
}

pub async fn cmd_scan_for_devices(client: &ButtplugClient) -> Result<(), ButtplugClientError> {
    if let Err(err) = client.start_scanning().await {
        error!(error = err.to_string(), "Failed scanning for devices.");
        return Err(err);
    }
    Ok(())
}

pub async fn cmd_stop_scan(client: &ButtplugClient) -> bool {
    if let Err(err) = client.stop_scanning().await {
        error!(
            error = err.to_string(),
            "Failed to stop scanning for devices."
        );
        return false;
    }
    true
}

pub struct DeviceAccess {
    device_actions: HashMap<u32, Vec<(i32, Speed)>>,
}

impl DeviceAccess {
    pub fn new() -> Self {
        DeviceAccess {
            device_actions: HashMap::new(),
        }
    }

    pub fn record_start(&mut self, device: &Arc<ButtplugClientDevice>, handle: i32, action: Speed) {
        self.device_actions
            .entry(device.index())
            .and_modify(|bag| bag.push((handle, action)))
            .or_insert(vec![(handle, action)]);
    }

    pub fn record_stop(&mut self, device: &Arc<ButtplugClientDevice>, handle: i32) {
        if let Some(action) = self.device_actions.remove(&device.index()) {
            let except_handle: Vec<(i32, Speed)> =
                action.into_iter().filter(|t| t.0 != handle).collect();
            self.device_actions.insert(device.index(), except_handle);
        }
    }

    pub fn get_remaining_speed(&self, device: &Arc<ButtplugClientDevice>) -> Option<Speed> {
        if let Some(actions) = self.device_actions.get(&device.index()) {
            let mut sorted: Vec<(i32, Speed)> = actions.clone();
            sorted.sort_by_key(|b| b.0);
            if let Some(tuple) = sorted.last() {
                return Some(tuple.1);
            }
        }
        None
    }

    pub fn calculate_actual_speed(
        &self,
        device: &Arc<ButtplugClientDevice>,
        new_speed: Speed,
    ) -> Speed {
        if let Some(actions) = self.device_actions.get(&device.index()) {
            let mut sorted: Vec<(i32, Speed)> = actions.clone();
            sorted.sort_by_key(|b| b.0);
            if let Some(tuple) = sorted.last() {
                warn!("Speed {} overrids provided speed {}", tuple.1, new_speed);
                return tuple.1;
            }
        }
        new_speed
    }
}

pub struct ReferenceCounter {
    access_list: HashMap<u32, u32>,
}

impl ReferenceCounter {
    pub fn new() -> Self {
        ReferenceCounter {
            access_list: HashMap::new(),
        }
    }
    pub fn reserve(&mut self, device: &Arc<ButtplugClientDevice>) {
        self.access_list
            .entry(device.index())
            .and_modify(|counter| *counter += 1)
            .or_insert(1);
        trace!(
            "Reserved device={} ref-count={}",
            device.name(),
            self.current_references(&device)
        )
    }
    pub fn release(&mut self, device: &Arc<ButtplugClientDevice>) {
        self.access_list
            .entry(device.index())
            .and_modify(|counter| {
                if *counter > 0 {
                    *counter -= 1
                } else {
                    warn!("Release on ref-count=0")
                }
            })
            .or_insert(0);
        trace!(
            "Released device={} ref-count={}",
            device.name(),
            self.current_references(&device)
        )
    }

    pub fn needs_to_stop(&self, device: &Arc<ButtplugClientDevice>) -> bool {
        self.current_references(device) == 0
    }
    fn current_references(&self, device: &Arc<ButtplugClientDevice>) -> u32 {
        match self.access_list.get(&device.index()) {
            Some(count) => *count,
            None => 0,
        }
    }
    pub fn clear(&mut self) {
        self.access_list.clear();
    }
}

pub fn create_cmd_thread(
    client: ButtplugClient,
    event_sender: tokio::sync::mpsc::UnboundedSender<TkEvent>,
    mut command_receiver: tokio::sync::mpsc::Receiver<TkAction>,
    pattern_path: String,
) {
    Handle::current().spawn(async move {
        info!("Comand handling thread started");
        let _ = span!(Level::INFO, "cmd_handling_thread").entered();

        let (device_action_sender, mut device_action_receiver) =
            unbounded_channel::<TkDeviceAction>();

        // immediate device actions received from pattern handling
        let mut device_counter = ReferenceCounter::new();
        let mut device_access = DeviceAccess::new();

        Handle::current().spawn(async move {
            loop {
                if let Some(next_action) = device_action_receiver.recv().await {
                    trace!("Exec device action {:?}", next_action);
                    match next_action {
                        TkDeviceAction::Start(device, speed, priority, handle) => {
                            device_counter.reserve(&device);
                            if priority {
                                device_access.record_start(&device, handle, speed);
                            }
                            let result = device
                                .vibrate(&ScalarValueCommand::ScalarValue(
                                    device_access
                                        .calculate_actual_speed(&device, speed)
                                        .as_float(),
                                ))
                                .await;

                            match result {
                                Err(err) => {
                                    // TODO: Send device error event 
                                    // TODO: Implement better connected/disconnected handling for devices
                                    error!("Failed to set device vibration speed {:?}", err)
                                },
                                _ => {}
                            }
                        }
                        TkDeviceAction::Update(device, speed) => device
                            .vibrate(&ScalarValueCommand::ScalarValue(
                                device_access
                                    .calculate_actual_speed(&device, speed)
                                    .as_float(),
                            ))
                            .await
                            .unwrap_or_else(|_| error!("Failed to set device vibration speed.")),
                        TkDeviceAction::End(device, priority, handle) => {
                            device_counter.release(&device);
                            if priority {
                                device_access.record_stop(&device, handle);
                            }
                            if device_counter.needs_to_stop(&device) {
                                // nothing else is controlling the device, stop it
                                device
                                    .vibrate(&ScalarValueCommand::ScalarValue(0.0))
                                    .await
                                    .unwrap_or_else(|_| error!("Failed to stop vibration"));
                                info!("Device stopped {}", device.name())
                            } else if let Some(remaining_speed) =
                                device_access.get_remaining_speed(&device)
                            {
                                // see if we have a lower priority vibration still running
                                device
                                    .vibrate(&ScalarValueCommand::ScalarValue(
                                        remaining_speed.as_float(),
                                    ))
                                    .await
                                    .unwrap_or_else(|_| error!("Failed to reset vibration"))
                            }
                        }
                        TkDeviceAction::StopAll => {
                            device_counter.clear();
                            info!("Stop all action");
                        }
                    }
                }
            }
        });

        // global operations and long running pattern execution
        let mut cancellation_tokens: HashMap<i32, CancellationToken> = HashMap::new(); // TODO do cleanup of cancelled
        loop {
            let next_cmd = command_receiver.recv().await;
            if let Some(cmd) = next_cmd {
                let queue_full_err = "Event sender full";
                info!("Executing command {:?}", cmd);
                match cmd {
                    TkAction::Scan => {
                        match cmd_scan_for_devices(&client).await {
                            Ok(()) => event_sender
                                        .send(TkEvent::ScanStarted)
                                        .unwrap_or_else(|_| error!(queue_full_err)),
                            Err(err) => event_sender
                                        .send(TkEvent::ScanFailed(err))
                                        .unwrap_or_else(|_| error!(queue_full_err))
                        }
                       
                    }
                    TkAction::StopScan => {
                        cmd_stop_scan(&client).await;
                        event_sender
                            .send(TkEvent::ScanStopped)
                            .unwrap_or_else(|_| error!(queue_full_err));
                    }
                    TkAction::Disconect => {
                        client
                            .disconnect()
                            .await
                            .unwrap_or_else(|_| error!("Failed to disconnect."));
                        break;
                    }
                    TkAction::StopAll => {
                        for entry in cancellation_tokens.drain() {
                            entry.1.cancel();
                        }
                        client
                            .stop_all_devices()
                            .await
                            .unwrap_or_else(|_| error!("Failed to stop all devices."));
                        device_action_sender
                            .send(TkDeviceAction::StopAll)
                            .unwrap_or_else(|_| error!(queue_full_err));
                        event_sender
                            .send(TkEvent::StopAll())
                            .unwrap_or_else(|_| error!(queue_full_err));
                    }
                    TkAction::Control(handle, params) => {
                        let devices = client.devices().clone();
                        let selection = params.filter_devices(devices);
                        let event_sender_clone = event_sender.clone();
                        let device_action_sender_clone = device_action_sender.clone();

                        let cancel_token = CancellationToken::new();
                        if let Some(_old) = cancellation_tokens.insert(handle, cancel_token.clone())
                        {
                            error!("Handle {} already existed", handle);
                        }
                        let pattern_path_clone = pattern_path.clone();
                        Handle::current().spawn(async move {
                            let player = TkPatternPlayer {
                                devices: selection,
                                action_sender: device_action_sender_clone,
                                event_sender: event_sender_clone,
                                resolution_ms: 100,
                                pattern_path: pattern_path_clone,
                            };
                            player.play(params.pattern, cancel_token, handle).await;
                        });
                    }
                    TkAction::Stop(handle) => {
                        if cancellation_tokens.contains_key(&handle) {
                            cancellation_tokens.remove(&handle).unwrap().cancel();
                            event_sender
                                .send(TkEvent::DeviceStopped())
                                .unwrap_or_else(|_| error!(queue_full_err));
                        } else {
                            error!("Unknown handle {}", handle);
                        }
                    }
                }
            } else {
                info!("Command stream closed");
                break;
            }
        }
    });
}
