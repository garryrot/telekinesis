use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};

use buttplug::client::{ButtplugClient, ButtplugClientDevice, ButtplugClientEvent};
use crossbeam_channel::Sender;
use futures::StreamExt;
use tokio::runtime::Handle;
use tracing::{debug, error, info, span, Level};

use crate::{input::TkParams, pattern::Speed, settings::TkConnectionType, DeviceList};

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
            status,
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
    pub tags: Vec<String>,
    pub devices: DeviceList,
    pub speed: Speed,
    pub pattern: String,
    pub event_type: TkDeviceEventType,
}

#[derive(Debug, Clone)]
pub enum TkDeviceEventType {
    Scalar,
    Linear,
}

impl TkDeviceEvent {
    pub fn new(
        elapsed: Duration,
        devices: &DeviceList,
        params: TkParams,
        pattern_name: String,
        event_type: TkDeviceEventType,
    ) -> Self {
        let (speed, pattern) = match params.pattern {
            crate::TkPattern::Linear(_, speed) => (speed, String::from("Linear")),
            crate::TkPattern::Funscript(_, _) => (Speed::max(), pattern_name),
        };
        TkDeviceEvent {
            elapsed_sec: elapsed.as_secs_f32(),
            tags: params.events,
            devices: devices.clone(),
            speed,
            pattern,
            event_type,
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
    ActionError(TkDeviceEvent, String),
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
    connection_type: TkConnectionType,
) {
    let mut buttplug_events = client.event_stream();
    let sender_clone = event_sender.clone();
    let status_clone = connection_status.clone();
    Handle::current().spawn(async move {
        let _ = span!(Level::INFO, "connection control").entered();
        loop {
            let next_cmd = command_receiver.recv().await;
            if let Some(cmd) = next_cmd {
                info!("Executing command {:?}", cmd);
                match cmd {
                    TkAction::Scan => {
                        if let Err(err) = client.start_scanning().await {
                            let error = err.to_string();
                            error!("connection failure {}", error);
                            try_send_event(
                                &sender_clone,
                                TkConnectionEvent::ConnectionFailure(err.to_string()),
                            );
                            try_set_status(
                                &status_clone,
                                TkConnectionStatus::Failed(err.to_string()),
                            );
                        } else {
                            let settings = connection_type.to_string();
                            info!(settings, "connection success");
                            try_send_event(&sender_clone, TkConnectionEvent::Connected(settings));
                            try_set_status(&status_clone, TkConnectionStatus::Connected);
                        }
                    }
                    TkAction::StopScan => {
                        if let Err(err) = client.stop_scanning().await {
                            let error = err.to_string();
                            error!(error, "failed stop scan");
                            try_set_status(&status_clone, TkConnectionStatus::Failed(error));
                        }
                    }
                    TkAction::Disconect => {
                        client
                            .disconnect()
                            .await
                            .unwrap_or_else(|_| error!("Failed to disconnect."));
                        try_set_status(&status_clone, TkConnectionStatus::NotConnected);
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
                break;
            }
        }
        info!("stream closed");
    });

    let _ = span!(Level::INFO, "device control").entered();
    while let Some(event) = buttplug_events.next().await {
        match event.clone() {
            ButtplugClientEvent::DeviceAdded(device) => {
                info!("device added {} ({})", device.name(), device.index() );
                try_set_device_status(&connection_status, &device, TkConnectionStatus::Connected);
                try_send_event(&event_sender, TkConnectionEvent::DeviceAdded(device));
            }
            ButtplugClientEvent::DeviceRemoved(device) => {
                info!("device removed {} ({})", device.name(), device.index() );
                try_set_device_status(&connection_status, &device, TkConnectionStatus::NotConnected);             
                try_send_event(&event_sender, TkConnectionEvent::DeviceRemoved(device));
            }
            ButtplugClientEvent::Error(err) => {
                error!("client error event {:?}", err);
            }
            _ => {}
        };
    }
}

fn try_set_device_status(connection_status: &Arc<Mutex<TkStatus>>, device: &Arc<ButtplugClientDevice>, status: TkConnectionStatus) {
    connection_status
        .lock()
        .expect("mutex healthy")
        .device_status
        .insert(
            device.index(),
            TkDeviceStatus::new(&device, status),
        );
}

fn try_set_status(connection_status: &Arc<Mutex<TkStatus>>, status: TkConnectionStatus) {
    connection_status
        .lock()
        .expect("mutex healthy")
        .connection_status = status;
}

fn try_send_event(sender: &Sender<TkConnectionEvent>, evt: TkConnectionEvent) {
    sender
        .try_send(evt)
        .unwrap_or_else(|_| error!("Event sender full"));
}
