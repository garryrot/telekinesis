use std::{ffi::c_float, sync::Arc};

use buttplug::{
    client::{ButtplugClient, ButtplugClientError, ButtplugClientEvent, VibrateCommand, ButtplugClientDevice},
    server::ButtplugServerError, core::errors::ButtplugError,
};
use futures::{Future, StreamExt};
use tokio::runtime::Runtime;
use tracing::{error, info, warn};

pub struct Telekinesis {
    pub runtime: Runtime,
    pub event_receiver: tokio::sync::mpsc::Receiver<TkEventEnum>,
    pub command_sender: tokio::sync::mpsc::Sender<TkCommand>
}

pub enum TkError {
    ServerError(ButtplugServerError),
    ClientError(ButtplugClientError),
}

impl From<ButtplugServerError> for TkError {
    fn from(e: ButtplugServerError) -> Self {
        TkError::ServerError(e)
    }
}
impl From<ButtplugClientError> for TkError {
    fn from(e: ButtplugClientError) -> Self {
        TkError::ClientError(e)
    }
}

pub enum TkEventEnum {
    DeviceAdded(Arc<ButtplugClientDevice>),
    DeviceRemoved(Arc<ButtplugClientDevice>),
    DeviceVibrated(i32),
    DeviceStopped(i32),
    TkError(ButtplugError),
    Other(ButtplugClientEvent)
}


impl TkEventEnum {
    fn from_event(event: ButtplugClientEvent) -> TkEventEnum {
        match event {
            ButtplugClientEvent::DeviceAdded(device) => TkEventEnum::DeviceAdded(device),
            ButtplugClientEvent::DeviceRemoved(device) => TkEventEnum::DeviceRemoved(device),
            ButtplugClientEvent::Error(err) => TkEventEnum::TkError(err),
            other => TkEventEnum::Other(other)
        }
    }

    pub fn as_string(&self) -> String {
        match self {
            TkEventEnum::DeviceAdded(device) => format!("Device '{}' connected.", device.name()),
            TkEventEnum::DeviceRemoved(device) => format!("Device '{}' Removed.", device.name()),
            TkEventEnum::DeviceVibrated(i) => format!("Vibrating '{}' devices.", i),
            TkEventEnum::DeviceStopped(i) => format!("Stopping '{}' devices.", i),
            TkEventEnum::TkError(err) => format!("Error '{}'", err.to_string()),
            TkEventEnum::Other(other) => format!("{:?}", other),
        }
    }
}

pub enum TkCommand {
    TkScan,
    TkVibrateAll(f32),
    TkStopAll,
    TkDiscconect
}

pub async fn cmd_scan_for_devices(client: &ButtplugClient) -> bool {
    info!("Scanning for devices.");
    if let Err(err) = client.start_scanning().await {
        error!(error = err.to_string(), "Failed scanning for devices.");
        return false
    }
    true
}

pub async fn cmd_vibrate_all(client: &ButtplugClient, speed: c_float) -> i32 {
    let mut vibrated = 0;
    for device in client
        .devices()
        .iter()
        .filter(|d| d.message_attributes().scalar_cmd().is_some())
    {
        info!("Device can vibrate. Setting vibration speed to {}.", speed);
        match device.vibrate(&VibrateCommand::Speed(speed.into())).await {
            Ok(_) => vibrated += 1,
            Err(err) => error!(
                dev = device.name(),
                error = err.to_string(),
                "Failed to set device vibration speed."
            ),
        }
    }
    vibrated
}

pub async fn cmd_stop_all(client: &ButtplugClient) -> i32 {
    let mut stopped = 0;
    for device in client.devices() {
        info!(dev = device.name(), "Stopping device.");
        match device.stop().await {
            Ok(_) => stopped += 1,
            Err(err) => error!(
                dev = device.name(),
                error = err.to_string(),
                "Failed to stop device."
            ),
        }
    }
    stopped
}

pub fn create_cmd_handling_thread(
    runtime: &Runtime,
    client: ButtplugClient,
    event_sender: tokio::sync::mpsc::Sender<TkEventEnum>,
) -> tokio::sync::mpsc::Sender<TkCommand> {
    let (command_sender, mut command_receiver) = tokio::sync::mpsc::channel(4096);
    runtime.spawn(async move {
        info!("Comand worker thread started");
        while let Some(command) = command_receiver.recv().await {
            match command {
                TkCommand::TkScan => {
                   cmd_scan_for_devices(&client).await;
                }
                TkCommand::TkVibrateAll(speed) => {
                    let vibrated = cmd_vibrate_all(&client, speed).await;
                    event_sender.send(TkEventEnum::DeviceVibrated(vibrated)).await.unwrap_or_else(|_| error!("Failed to send vibrated to queue."));
                }
                TkCommand::TkStopAll => {
                    let stopped = cmd_stop_all(&client).await;
                    event_sender.send(TkEventEnum::DeviceStopped(stopped)).await.unwrap_or_else(|_| error!("Failed to send stopped to queue."));
                }
                TkCommand::TkDiscconect => {
                    client.disconnect().await.unwrap_or_else(|_| error!("Failed to send disconnect to queue."));
                },
            }
        }
    });
    command_sender
}

pub fn create_event_handling_thread(
    runtime: &Runtime,
    client: &ButtplugClient,
) -> (tokio::sync::mpsc::Receiver<TkEventEnum>, tokio::sync::mpsc::Sender<TkEventEnum>) {
    let (event_sender, event_receiver) = tokio::sync::mpsc::channel(512);
    let sender_clone = event_sender.clone();
    let mut events = client.event_stream();
    runtime.spawn(async move {
        info!("Event polling thread started");
        while let Some(event) = events.next().await {
            event_sender.send(TkEventEnum::from_event(event)).await.unwrap_or_else(|_| warn!("Dropped event cause queue is full."));
        }
    });
    (event_receiver, sender_clone)
}

impl Telekinesis {
    pub fn new(
        fut: impl Future<Output = Result<ButtplugClient, TkError>>,
    ) -> Result<Telekinesis, TkError> {
        let runtime = Runtime::new().unwrap();
        let client = runtime.block_on(fut)?;
        let (event_receiver, event_sender) = create_event_handling_thread(&runtime, &client);
        let command_sender = create_cmd_handling_thread(&runtime, client, event_sender);
        Ok(Telekinesis {
            runtime: runtime,
            event_receiver: event_receiver,
            command_sender: command_sender
        })
    }

    pub fn scan_for_devices(&self) -> bool {
        if let Err(_) = self.command_sender.blocking_send(TkCommand::TkScan) {
            error!("Failed to send vibrate_all"); // whats skyrim gonna do about it
            return false
        }
        return true
    }

    pub fn vibrate_all(&self, speed: f32) -> bool {
        if let Err(_) = self.command_sender.blocking_send(TkCommand::TkVibrateAll(speed)) {
            error!("Failed to send vibrate_all"); 
            return false
        }
        return true
    }

    pub fn stop_all(&self) -> bool {
        if let Err(_) = self.command_sender.blocking_send(TkCommand::TkStopAll) {
            error!("Failed to send stop_all");
            return false
        }
        return true
    }

    pub fn disconnect(&mut self) {
        info!("Disconnecting client.");
        if let Err(_) = self.command_sender.blocking_send(TkCommand::TkDiscconect) {
            error!("Failed to send disconnect");
        }
    }

    pub fn get_next_event(&mut self) -> Option<TkEventEnum> {
        if let Ok(msg) = self.event_receiver.try_recv() {
            return Some(msg)
        }
        None
    }

    // pub fn tk_get_connected_devices(&self) {
    //     self.runtime.block_on(async {
    //         self.client
    //                .devices()
    //                .iter()
    //                .filter( |f| self.is_vibrator(&f))
    //                .map(|f| f.name().clone() )
    //                .collect::<Vec<String>>()
    //     });
    // }
}
