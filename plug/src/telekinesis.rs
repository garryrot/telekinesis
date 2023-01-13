use std::{ffi::c_float, sync::Arc};

use buttplug::{
    client::{
        ButtplugClient, ButtplugClientDevice, ButtplugClientError, ButtplugClientEvent,
        VibrateCommand,
    },
    core::{connector::ButtplugInProcessClientConnectorBuilder, errors::ButtplugError},
    server::{
        device::hardware::communication::btleplug::BtlePlugCommunicationManagerBuilder,
        ButtplugServerBuilder, ButtplugServerError,
    },
};
use futures::{Future, StreamExt};
use tokio::{runtime::Runtime, select, time::sleep};
use tracing::{debug, error, info, instrument, span, warn, Level};

use crate::util::Narrow;

#[derive(Debug)]
pub struct Telekinesis {
    pub runtime: Runtime,
    pub event_receiver: tokio::sync::mpsc::Receiver<TkEventEnum>,
    pub command_sender: tokio::sync::mpsc::Sender<TkCommand>,
}

#[derive(Debug)]
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

#[derive(Debug)]
pub enum TkEventEnum {
    DeviceAdded(Arc<ButtplugClientDevice>),
    DeviceRemoved(Arc<ButtplugClientDevice>),
    DeviceVibrated(i32),
    DeviceStopped(i32),
    TkError(ButtplugError),
    Other(ButtplugClientEvent),
}

impl TkEventEnum {
    fn from_event(event: ButtplugClientEvent) -> TkEventEnum {
        match event {
            ButtplugClientEvent::DeviceAdded(device) => TkEventEnum::DeviceAdded(device),
            ButtplugClientEvent::DeviceRemoved(device) => TkEventEnum::DeviceRemoved(device),
            ButtplugClientEvent::Error(err) => TkEventEnum::TkError(err),
            other => TkEventEnum::Other(other),
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

#[derive(Debug)]
pub enum TkCommand {
    TkScan,
    TkVibrateAll(f32),
    TkVibrateAllDelayed(f32, std::time::Duration),
    TkStopAll,
    TkDiscconect,
}

pub async fn cmd_scan_for_devices(client: &ButtplugClient) -> bool {
    info!("Scanning for devices.");
    if let Err(err) = client.start_scanning().await {
        error!(error = err.to_string(), "Failed scanning for devices.");
        return false;
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
        debug!("Vibrating device {} with speed {}", device.name(), speed);
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
    let (command_sender, mut command_receiver) = tokio::sync::mpsc::channel(128); // shouldn't be big, we consume cmds immediately
    runtime.spawn(async move {
        info!("Comand worker thread started");
        let _ = span!(Level::INFO, "cmd_handling_thread").entered();

        let mut delayed_cmd: Option<TkCommand> = None;
        loop {
            let recv_fut = command_receiver.recv();
            let cmd = if let Some(TkCommand::TkVibrateAllDelayed(speed, duration)) = delayed_cmd {
                debug!("Select delayed command");
                select! {
                    () = sleep(duration) => Some(TkCommand::TkVibrateAll(speed)),
                    cmd = recv_fut => cmd
                }
            } else {
                recv_fut.await
            };
            delayed_cmd = None; // always overwrite delayed with new command

            if let Some(cmd) = cmd {
                info!("Executing command {:?}", cmd);
                match cmd {
                    TkCommand::TkScan => {
                        cmd_scan_for_devices(&client).await;
                    }
                    TkCommand::TkVibrateAll(speed) => {
                        let vibrated = cmd_vibrate_all(&client, speed).await;
                        event_sender
                            .send(TkEventEnum::DeviceVibrated(vibrated))
                            .await
                            .unwrap_or_else(|_| error!("Queue full"));
                    }
                    TkCommand::TkStopAll => {
                        let stopped = cmd_stop_all(&client).await;
                        event_sender
                            .send(TkEventEnum::DeviceStopped(stopped))
                            .await
                            .unwrap_or_else(|_| error!("Queue full"));
                    }
                    TkCommand::TkDiscconect => {
                        client
                            .disconnect()
                            .await
                            .unwrap_or_else(|_| error!("Failed to send disconnect to queue."));
                    }
                    TkCommand::TkVibrateAllDelayed(_, duration) => {
                        info!("Delayed command {:?}", duration);
                        delayed_cmd = Some(cmd);
                    }
                }
            } else {
                info!("Command stream closed");
                break;
            }
        }
    });
    command_sender
}

pub fn create_event_handling_thread(
    runtime: &Runtime,
    client: &ButtplugClient,
) -> (
    tokio::sync::mpsc::Receiver<TkEventEnum>,
    tokio::sync::mpsc::Sender<TkEventEnum>,
) {
    let (event_sender, event_receiver) = tokio::sync::mpsc::channel(2048); // big in case events are not consumed
    let sender_clone = event_sender.clone();
    let mut events = client.event_stream();
    runtime.spawn(async move {
        info!("Event polling thread started");
        while let Some(event) = events.next().await {
            event_sender
                .send(TkEventEnum::from_event(event))
                .await
                .unwrap_or_else(|_| warn!("Dropped event cause queue is full."));
        }
    });
    (event_receiver, sender_clone)
}

impl Telekinesis {
    pub fn new_with_default_settings() -> Result<Telekinesis, TkError> {
        info!("Connecting with defualt settings");
        Telekinesis::new(async {
            let server = ButtplugServerBuilder::default()
                .comm_manager(BtlePlugCommunicationManagerBuilder::default())
                .finish()?;
            let connector = ButtplugInProcessClientConnectorBuilder::default()
                .server(server)
                .finish();
            let client = ButtplugClient::new("Telekinesis");
            client.connect(connector).await?;
            Ok::<ButtplugClient, TkError>(client)
        })
    }

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
            command_sender: command_sender,
        })
    }

    #[instrument]
    pub fn scan_for_devices(&self) -> bool {
        info!("Sending Command: Scan for devices");
        if let Err(_) = self.command_sender.blocking_send(TkCommand::TkScan) {
            error!("Failed to send vibrate_all"); // whats skyrim gonna do about it
            return false;
        }
        true
    }

    // TODO: Drop Messages if event queue has overflow to not force users to consume
    #[instrument]
    pub fn vibrate_all(&self, speed: f32) -> bool {
        info!("Sending Command: Vibrate all");
        if let Err(_) = self
            .command_sender
            .blocking_send(TkCommand::TkVibrateAll( speed.narrow( 0.0, 1.0 )))
        {
            error!("Failed to send vibrate_all");
            return false;
        }
        true
    }

    #[instrument]
    pub fn vibrate_all_delayed(&self, speed: f32, duration: std::time::Duration) -> bool {
        info!("Sending Command: Vibrate all delayed");
        if let Err(_) = self
            .command_sender
            .blocking_send(TkCommand::TkVibrateAllDelayed( speed.narrow( 0.0, 1.0 ), duration))
        {
            error!("Failed to send delayed command");
            return false;
        }
        true
    }

    #[instrument]
    pub fn stop_all(&self) -> bool {
        info!("Sending Command: Stop all");
        if let Err(_) = self.command_sender.blocking_send(TkCommand::TkStopAll) {
            error!("Failed to send stop_all");
            return false;
        }
        true
    }

    #[instrument]
    pub fn disconnect(&mut self) {
        info!("Sending Command: Disconnecting client");
        if let Err(_) = self.command_sender.blocking_send(TkCommand::TkDiscconect) {
            error!("Failed to send disconnect");
        }
    }

    pub fn get_next_event(&mut self) -> Option<TkEventEnum> {
        debug!("get_next_event");
        if let Ok(msg) = self.event_receiver.try_recv() {
            debug!("Got event {:?}", msg);
            return Some(msg);
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
