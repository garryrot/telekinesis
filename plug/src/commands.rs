use std::{time::Duration, sync::Arc};

use buttplug::client::{ButtplugClient, ScalarValueCommand, ButtplugClientDevice};
use itertools::Itertools;
use tokio::{runtime::Handle, select, time::sleep};
use tracing::{debug, error, info, span, Level};

use crate::{event::TkEvent, Speed, settings::TkSettings};

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
    pub async fn execute_single(&self, devices: Vec<&Arc<ButtplugClientDevice>>, stop: bool) -> (i32, Speed) {
        let mut vibrated = 0;
        let mut top_speed = Speed::min();
        match self.action {
            TkDeviceAction::Vibrate(speed) => {
                for device in devices.iter().filter(|d| d.message_attributes().scalar_cmd().is_some())
                {
                    if speed.value != 0 {
                        info!("Vibrating device {} with speed {}", device.name(), speed);
                    } else {
                        info!("Stopping device {}", device.name())
                    }
                    match device.vibrate(&ScalarValueCommand::ScalarValue(match stop {
                        true => Speed::min().as_float(),
                        false => speed.as_float(),
                    })).await {
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
            }
            _ => todo!(),
        }
        (vibrated, top_speed)
    }

    pub async fn execute(&self, devices: Vec<Arc<ButtplugClientDevice>>, sender: tokio::sync::mpsc::UnboundedSender<TkEvent>) {
        let selected_devices : Vec<&Arc<ButtplugClientDevice>>  = devices
            .iter()
            .filter(|d| match &self.devices {
                TkDeviceSelector::All => true,
                TkDeviceSelector::ByNames(names) => {
                    let matches = names.iter().any(|x| x == d.name());
                    matches
                }
            })
            .collect();

        let (vibrated, top_speed) = self.execute_single(selected_devices.clone(), false).await;
        sender.send(TkEvent::DeviceVibrated(vibrated, top_speed)); // .unwrap_or_else(|_| error!("Event sender full"))
        sleep(self.duration).await;

        self.execute_single(selected_devices, true).await;
        sender.send(TkEvent::DeviceStopped());

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
        loop {
            let next_cmd = command_receiver.recv().await;
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
                        let sender_clone = event_sender.clone();
                        let devices = client.devices();
                        Handle::current().spawn(async move {
                            control.execute(devices, sender_clone).await;
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
