use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};

use buttplug::client::{ButtplugClient, ButtplugClientDevice, ButtplugClientEvent};
use futures::StreamExt;
use tokio::runtime::Handle;
use tracing::{error, info, span, Level};

use crate::{
    input::TkParams,
    DeviceList, pattern::Speed, settings::TkConnectionType,
};

#[derive(Debug, Clone, PartialEq)]
pub enum TkConnectionStatus {
    NotConnected,
    Connected,
    Failed(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TkDeviceStatus {
    pub status: TkConnectionStatus,
    pub device: Arc<ButtplugClientDevice>,
}

impl TkDeviceStatus {
    pub fn new(device: &Arc<ButtplugClientDevice>, status: TkConnectionStatus) -> Self {
        TkDeviceStatus {
            device: device.clone(),
            status
        }
    }
}

#[derive(Debug)]
pub struct TkStatus {
    pub connection_status: TkConnectionStatus,
    pub device_status: HashMap<u32, TkDeviceStatus>,
}

impl TkStatus {
    pub fn new() -> Self {
        TkStatus {
            connection_status: TkConnectionStatus::NotConnected,
            device_status: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TkDeviceEvent {
    pub elapsed_sec: f32,
    pub events: Vec<String>,
    pub devices: DeviceList,
    pub speed: Speed,
    pub pattern: String,
}

impl TkDeviceEvent {
    pub fn new(elapsed: Duration, devices: &DeviceList, params: TkParams, pattern_name: String) -> Self {
        let (speed, pattern) = match params.pattern {
            crate::TkPattern::Linear(_, speed) => (speed, String::from("Linear")),
            crate::TkPattern::Funscript(_, _) => (Speed::max(), pattern_name),
        };
        TkDeviceEvent {
            elapsed_sec: elapsed.as_secs_f32(),
            events: params.events,
            devices: devices.clone(),
            speed: speed,
            pattern: pattern,
        }
    }
}

#[derive(Debug)]
pub enum TkConnectionEvent {
    Connected(String),
    ConnectionFailure(String),
    DeviceAdded(Arc<ButtplugClientDevice>),
    DeviceRemoved(Arc<ButtplugClientDevice>),
    ActionStarted(TkDeviceEvent),
    ActionDone(TkDeviceEvent),
    ActionError(TkDeviceEvent, String)
}

#[derive(Clone, Debug)]
pub enum TkAction {
    Scan,
    StopScan,
    StopAll,
    Disconect,
}

pub async fn handle_connection(
    event_sender: crossbeam_channel::Sender<TkConnectionEvent>,
    mut command_receiver: tokio::sync::mpsc::Receiver<TkAction>,
    client: ButtplugClient,
    connection_status: Arc<Mutex<TkStatus>>,
    type_name: TkConnectionType
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
                            error!("Device error {}", error);
                            event_sender_clone
                                .send(TkConnectionEvent::ConnectionFailure(err.to_string()))
                                .unwrap_or_else(|_| error!(queue_full_err));
                            connection_status_clone
                                .lock()
                                .expect("mutex healthy")
                                .connection_status = TkConnectionStatus::Failed(err.to_string());
                        } else {
                            event_sender_clone
                                .send(TkConnectionEvent::Connected(type_name.to_string()))
                                .unwrap_or_else(|_| error!(queue_full_err));
                            connection_status_clone
                                .lock()
                                .expect("mutex healthy")
                                .connection_status = TkConnectionStatus::Connected;
                        }
                    }
                    TkAction::StopScan => {
                        if let Err(err) = client.stop_scanning().await {
                            let error = err.to_string();
                            error!(error, "Failed to stop scanning for devices.");
                            connection_status_clone
                                .lock()
                                .expect("mutex healthy")
                                .connection_status = TkConnectionStatus::Failed(error);
                        }
                    }
                    TkAction::Disconect => {
                        client
                            .disconnect()
                            .await
                            .unwrap_or_else(|_| error!("Failed to disconnect."));
                        connection_status_clone
                            .lock()
                            .expect("mutex healthy")
                            .connection_status = TkConnectionStatus::NotConnected;
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
                    connection_status
                        .device_status
                        .insert(device.index(), TkDeviceStatus::new(&device, TkConnectionStatus::Connected));
                } else {
                    error!("mutex poisoned")
                }
                event_sender
                    .send(TkConnectionEvent::DeviceAdded(device))
                    .expect("queue full");
            }
            ButtplugClientEvent::DeviceRemoved(device) => {
                if let Ok(mut connection_status) = connection_status.lock() {
                    connection_status
                        .device_status
                        .insert(device.index(), TkDeviceStatus::new(&device, TkConnectionStatus::Connected));
                } else {
                    error!("mutex poisoned")
                }
                event_sender
                    .send(TkConnectionEvent::DeviceRemoved(device))
                    .expect("queue full");
            }
            ButtplugClientEvent::Error(err) => {
                error!("Server error {:?}", err);
            }
            _ => {}
        };
    }
}
