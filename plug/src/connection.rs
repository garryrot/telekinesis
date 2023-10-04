
use std::{collections::HashMap, sync::{Arc, Mutex}};

use buttplug::client::{ButtplugClient, ButtplugClientEvent, ButtplugClientError, ButtplugClientDevice};
use futures::StreamExt;
use tokio::runtime::Handle;
use tracing::{error, info, span, Level, warn};

#[derive(Debug, Clone)]
pub enum TkConnectionStatus {
    NotConnected,
    Connected,
    Failed(String)
}

#[derive(Debug)]
pub struct TkStatus {
    pub connection_status: TkConnectionStatus,
    pub device_status: HashMap<u32, (TkConnectionStatus, Arc<ButtplugClientDevice>)>
}

impl TkStatus {
    pub fn new() -> Self {
        TkStatus { connection_status: TkConnectionStatus::NotConnected, device_status: HashMap::new() }
    }
}

#[derive(Debug)]
pub enum TkConnectionEvent {
    ButtplugClientEvent(ButtplugClientEvent),
    ScanFailed(ButtplugClientError),
    ScanStarted,
    ScanStopped
}

impl TkConnectionEvent {
    pub fn from_event(event: ButtplugClientEvent) -> TkConnectionEvent {
        TkConnectionEvent::ButtplugClientEvent(event)
    }
}

#[derive(Clone, Debug)]
pub enum TkAction {
    Scan,
    StopScan,
    StopAll,
    Disconect,
}

pub async fn handle_connection(
    event_sender: tokio::sync::mpsc::UnboundedSender<TkConnectionEvent>,
    mut command_receiver: tokio::sync::mpsc::Receiver<TkAction>,
    client: ButtplugClient,
    connection_status: Arc<Mutex<TkStatus>>
) {
    let mut buttplug_events = client.event_stream();
    let event_sender_clone = event_sender.clone();
    let queue_full_err = "Event sender full";

    let connection_status_clone = connection_status.clone();
    Handle::current().spawn(async move {
        info!("Handling connection commands");
        let _ = span!(Level::INFO, "cmd_handling_thread").entered();
        loop {
            let next_cmd = command_receiver.recv().await;
            if let Some(cmd) = next_cmd {
                info!("Executing command {:?}", cmd);
                match cmd {
                    TkAction::Scan => {
                        if let Err(err) = client.start_scanning().await {
                            let error = err.to_string();
                            error!(error, "Failed scanning for devices.");
                            event_sender_clone
                                .send(TkConnectionEvent::ScanFailed(err))
                                .unwrap_or_else(|_| error!(queue_full_err));
                                connection_status_clone.lock().expect("mutex healthy").connection_status = TkConnectionStatus::Failed(error);
                        } else {
                            event_sender_clone
                                .send(TkConnectionEvent::ScanStarted)
                                .unwrap_or_else(|_| error!(queue_full_err));
                            connection_status_clone.lock().expect("mutex healthy").connection_status = TkConnectionStatus::Connected;
                        }
                    }
                    TkAction::StopScan => {
                        if let Err(err) = client.stop_scanning().await {
                            let error = err.to_string();
                            error!(error, "Failed to stop scanning for devices.");
                            connection_status_clone.lock().expect("mutex healthy").connection_status = TkConnectionStatus::Failed(error);
                        } else {
                            event_sender_clone
                                .send(TkConnectionEvent::ScanStopped)
                                .unwrap_or_else(|_| error!(queue_full_err));
                        }
                    }
                    TkAction::Disconect => {
                        client
                            .disconnect()
                            .await
                            .unwrap_or_else(|_| error!("Failed to disconnect."));
                        connection_status_clone.lock().expect("mutex healthy").connection_status = TkConnectionStatus::NotConnected;
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

    while let Some(event) = buttplug_events.next().await {
        match event.clone() {
            ButtplugClientEvent::DeviceAdded(device) => {
                if let Ok(mut connection_status) = connection_status.lock() {
                    connection_status.device_status.insert(
                        device.index(),
                        (TkConnectionStatus::Connected, device.clone()),
                    );
                } else {
                    error!("mutex poisoned")
                }
            }
            ButtplugClientEvent::DeviceRemoved(device) => {
                if let Ok(mut connection_status) = connection_status.lock() {
                    connection_status.device_status.insert(
                        device.index(),
                        (TkConnectionStatus::NotConnected, device.clone()),
                    );
                } else {
                    error!("mutex poisoned")
                }
            }
            ButtplugClientEvent::Error(err) => {
                error!("Server error {:?}", err);
            }
            _ => {}
        };

        event_sender
            .send(TkConnectionEvent::from_event(event))
            .unwrap_or_else(|_| warn!("Dropped event cause queue is full."));
    }
}
