use std::sync::Arc;

use buttplug::client::{ButtplugClient, ButtplugClientDevice, ButtplugClientError};
use tokio::runtime::Handle;
use tracing::{error, info, span, Level};

use crate::{
    event::TkEvent,
    settings::{TkDeviceSettings, TkSettings},
    Speed, TkPattern,
};

type DeviceNameList = Box<Vec<String>>;

#[derive(Clone, Debug)]
pub enum TkAction {
    Scan,
    StopScan,
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

pub fn create_cmd_thread(
    event_sender: tokio::sync::mpsc::UnboundedSender<TkEvent>,
    mut command_receiver: tokio::sync::mpsc::Receiver<TkAction>,
    client: ButtplugClient
) {
    Handle::current().spawn(async move {
        info!("Comand handling thread started");
        let _ = span!(Level::INFO, "cmd_handling_thread").entered();

        // global operations and long running pattern execution
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
                        client
                            .stop_all_devices()
                            .await
                            .unwrap_or_else(|_| error!("Failed to stop all devices."));
                    }
                }
            } else {
                info!("Command stream closed");
                break;
            }
        }
    });
}
