use std::time::Duration;

use buttplug::client::{ButtplugClient, ScalarValueCommand};
use tokio::{runtime::Handle, select, time::sleep};
use tracing::{debug, error, info, span, Level};

use crate::{event::TkEvent, Speed, settings::TkSettings};

type DeviceNameList = Box<Vec<String>>;

#[derive(Clone, Debug)]
pub enum TkAction {
    Scan,
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

impl TkControl {
    pub fn get_stop_action(&self) -> TkAction {
        TkAction::Control(TkControl {
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
                    info!("Vibrating device {} with speed {}", device.name(), speed);
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
        let mut delayed_cmd: Option<TkAction> = None;
        let mut delayed_timer: Duration = Duration::ZERO;
        loop {
            let recv_fut = command_receiver.recv();
            let cmd = if let Some(TkAction::Control(control)) = delayed_cmd {
                debug!("Select delayed command");
                select! {
                    () = sleep(delayed_timer) => Some(TkAction::Control(control)),
                    cmd = recv_fut => cmd
                }
            } else {
                recv_fut.await
            };
            delayed_cmd = None; // always overwrite delayed with new command

            if let Some(cmd) = cmd {
                info!("Executing command {:?}", cmd);
                match cmd {
                    TkAction::Scan => {
                        cmd_scan_for_devices(&client).await;
                    }
                    TkAction::StopAll => {
                        client.stop_all_devices().await.unwrap_or_else(|_| error!("Failed to stop all devices."));
                        event_sender
                            .send(TkEvent::DeviceStopped())
                            .expect("Open");
                    }
                    TkAction::Disconect => {
                        client
                            .disconnect()
                            .await
                            .unwrap_or_else(|_| error!("Failed to disconnect."));
                    }
                    TkAction::Control(control) => {
                        let vibrated =  control.execute(&client).await;
                        event_sender.send(vibrated).unwrap_or_else(|_| error!("Failed sending event."));
                        if !control.duration.is_zero() {
                            delayed_timer = control.duration;
                            delayed_cmd = Some(control.get_stop_action());
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
