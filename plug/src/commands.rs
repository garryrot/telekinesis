use std::time::Duration;

use buttplug::client::{ButtplugClient, ScalarValueCommand};
use tokio::{runtime::Handle, select, time::sleep};
use tracing::{debug, error, info, span, Level};

use crate::{event::TkEvent, Speed, settings::TkSettings};

type DeviceNameList = Box<Vec<String>>;

#[derive(Clone, Debug)]
pub enum TkAction {
    Scan,
    StopScan,
    Control(TkControl),
    Stop(TkControl),
    StopAll,
    Disconect,
}

#[derive(Clone, Debug)]
pub struct TkControl {
    pub duration: Duration,
    pub devices: TkDeviceSelector,
    pub action: TkDeviceAction,
}

#[derive(Clone, Debug)]
pub enum TkDeviceSelector {
    All,
    ByNames(DeviceNameList),
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
    Vibrate(Speed),
    VibratePattern(String),
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
        error!(error = err.to_string(), "Failed to stop scanning for devices.");
        return false;
    }
    true
}

impl TkControl {
    pub fn get_stop_action(&self) -> TkAction {
        TkAction::Stop(TkControl {
            duration: Duration::ZERO,
            devices: self.devices.clone(),
            action: TkDeviceAction::Vibrate(Speed::min()),
        })
    }

    pub async fn execute(&self, client: &ButtplugClient) -> TkEvent {
        let devices = client.devices();
        let selected_devices = devices
            .iter()
            .filter(|d| match &self.devices {
                TkDeviceSelector::All => true,
                TkDeviceSelector::ByNames(names) => {
                    let matches = names.iter().any(|x| x == d.name());
                    matches
                }
            });
            
        match self.action {
            TkDeviceAction::Vibrate(speed) => {
                let mut vibrated = 0;
                let mut top_speed = Speed::min();
                for device in selected_devices.filter(|d| d.message_attributes().scalar_cmd().is_some())
                {
                    if speed.value != 0 {
                        info!("Vibrating device {} with speed {}", device.name(), speed);
                    } else {
                        info!("Stopping device {}", device.name())
                    }
                    match device.vibrate(&ScalarValueCommand::ScalarValue(speed.as_float())).await {
                        Ok(_) => vibrated += 1,
                        Err(err) => error!(
                            dev = device.name(),
                            error = err.to_string(),
                            "Failed to set device vibration speed."
                        ),
                    };
                    if speed.value > top_speed.value {
                        top_speed = speed;
                    }
                }
                TkEvent::DeviceVibrated(vibrated, top_speed)
            }
            _ => todo!(),
        }
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
        let mut stop_cmd: Option<TkAction> = None;
        let mut stop_timer: Duration = Duration::ZERO;
        loop {
            let recv_fut = command_receiver.recv();
            let next_cmd = if let Some(action) = stop_cmd {
                select! {
                    () = sleep(stop_timer) => Some(action),
                    cmd = recv_fut => cmd
                }
            } else {
                recv_fut.await
            };
            stop_cmd = None; // always overwrite delayed with new command

            if let Some(cmd) = next_cmd {
                let queue_full_err = "Event sender full";
                info!("Executing command {:?}", cmd);
                match cmd {
                    TkAction::Scan => {
                        cmd_scan_for_devices(&client).await;
                        event_sender.send(TkEvent::ScanStarted).unwrap_or_else( |_| error!(queue_full_err));
                    }
                    TkAction::StopScan => {
                        cmd_stop_scan(&client).await;
                        event_sender.send(TkEvent::ScanStopped).unwrap_or_else( |_| error!(queue_full_err));
                    },
                    TkAction::StopAll => {
                        client.stop_all_devices().await.unwrap_or_else(|_| error!("Failed to stop all devices."));
                        event_sender
                            .send(TkEvent::DeviceStopped())
                            .unwrap_or_else( |_| error!(queue_full_err));
                    }
                    TkAction::Disconect => {
                        client
                            .disconnect()
                            .await
                            .unwrap_or_else(|_| error!("Failed to disconnect."));
                        break;
                    }
                    TkAction::Control(control) => {
                        let vibrated =  control.execute(&client).await;
                        event_sender.send(vibrated).unwrap_or_else(|_| error!(queue_full_err));
                        stop_timer = control.duration;
                        stop_cmd = Some(control.get_stop_action());
                        debug!("Queing delayed stop command {:?} {:?}", stop_cmd, stop_timer);
                    }
                    TkAction::Stop(control) => {   
                        let vibrated = control.execute(&client).await;
                        event_sender.send(vibrated).unwrap_or_else(|_| error!(queue_full_err));
                    }
                }
            } else {
                info!("Command stream closed");
                break;
            }
        }
    });
}
