use std::{collections::HashMap, sync::Arc};

use buttplug::client::{ButtplugClient, ButtplugClientDevice, ScalarValueCommand};
use tokio::{runtime::Handle, sync::mpsc::unbounded_channel};
use tracing::{error, info, span, Level, debug, trace};

use crate::{event::TkEvent, pattern::TkPatternPlayer, settings::{TkSettings, PATTERN_PATH}, Speed, TkPattern};

type DeviceNameList = Box<Vec<String>>;

#[derive(Clone, Debug)]
pub enum TkAction {
    Scan,
    StopScan,
    Control(TkControl),
    StopAll,
    Disconect,
}

#[derive(Clone, Debug)]
pub struct TkControl {
    pub selector: TkDeviceSelector,
    pub pattern: TkPattern,
}

impl TkControl {
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
    Start(Arc<ButtplugClientDevice>, Speed),
    Update(Arc<ButtplugClientDevice>, Speed),
    End(Arc<ButtplugClientDevice>),
    StopAll, // global but required for resetting device state
}

pub async fn cmd_scan_for_devices(client: &ButtplugClient) -> bool {
    if let Err(err) = client.start_scanning().await {
        error!(error = err.to_string(), "Failed scanning for devices.");
        return false;
    }
    true
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
    access_list: HashMap<u32, u32>,
}

impl DeviceAccess {
    pub fn new() -> Self {
        DeviceAccess {
            access_list: HashMap::new(),
        }
    }
    pub fn reserve(&mut self, device: &Arc<ButtplugClientDevice>) {
        self.access_list
            .entry(device.index())
            .and_modify(|counter| *counter += 1)
            .or_insert(1);
        trace!("Reserved device={} ref-count={}", device.name(), self.current_references(&device))
    }
    pub fn release(&mut self, device: &Arc<ButtplugClientDevice>) {
        self.access_list
            .entry(device.index())
            .and_modify(|counter| *counter -= 1)
            .or_insert(0);
        trace!("Released device={} ref-count={}", device.name(), self.current_references(&device))
    }
    pub fn current_references(&self, device: &Arc<ButtplugClientDevice>) -> u32 {
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
) {
    Handle::current().spawn(async move {
        info!("Comand handling thread started");
        let _ = span!(Level::INFO, "cmd_handling_thread").entered();

        let (device_action_sender, mut device_action_receiver) =
            unbounded_channel::<TkDeviceAction>();

        // immediate device actions received from pattern handling
        let mut device_access = DeviceAccess::new();
        Handle::current().spawn(async move {
            loop {
                if let Some(next_action) = device_action_receiver.recv().await {
                    trace!("Exec device action {:?}", next_action);
                    match next_action {
                        TkDeviceAction::Start(device, speed) => {
                            device_access.reserve(&device);
                            device
                                .vibrate(&ScalarValueCommand::ScalarValue(speed.as_float()))
                                .await
                                .unwrap_or_else(|_| {
                                    error!("Failed to set device vibration speed.")
                                });
                        }
                        TkDeviceAction::Update(device, speed) => {
                            device.vibrate(&ScalarValueCommand::ScalarValue(speed.as_float()))
                                .await
                                .unwrap_or_else(|_| {
                                    error!("Failed to set device vibration speed.")
                                })
                        },
                        TkDeviceAction::End(device) => {
                            device_access.release(&device);
                            if device_access.current_references(&device) == 0 {
                                // nothing else is controlling the device, stop it
                                device
                                    .vibrate(&ScalarValueCommand::ScalarValue(0.0))
                                    .await
                                    .unwrap_or_else(|_| error!("Failed to stop vibration"));
                                info!("Device stopped {}", device.name())
                            }
                            else
                            {
                                info!("Device not stopped, open references: {}", device_access.current_references(&device));
                            }
                        }
                        TkDeviceAction::StopAll => {
                            device_access.clear();
                            info!("Stop all action");
                        }
                    }
                }
            }
        });

        // global operations and long running pattern execution
        loop {
            let next_cmd = command_receiver.recv().await;
            if let Some(cmd) = next_cmd {
                let queue_full_err = "Event sender full";
                info!("Executing command {:?}", cmd);
                match cmd {
                    TkAction::Scan => {
                        cmd_scan_for_devices(&client).await;
                        event_sender
                            .send(TkEvent::ScanStarted)
                            .unwrap_or_else(|_| error!(queue_full_err));
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

                    TkAction::Control(control) => {
                        let devices = client.devices().clone();
                        let selection = control.filter_devices(devices);
                        let event_sender_clone = event_sender.clone();
                        let device_action_sender_clone = device_action_sender.clone();
                        Handle::current().spawn(async move {
                            let player = TkPatternPlayer {
                                devices: selection,
                                action_sender: device_action_sender_clone,
                                event_sender: event_sender_clone,
                                resolution_ms: 100,
                                pattern_path: String::from(PATTERN_PATH)
                            };
                            player.play(control.pattern).await;
                        });
                    }
                }
            } else {
                info!("Command stream closed");
                break;
            }
        }
    });
}
