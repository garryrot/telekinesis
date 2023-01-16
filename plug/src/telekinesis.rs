use std::{sync::Arc, fmt::{self, Display}};

use buttplug::{
    client::{
        ButtplugClient, ButtplugClientDevice, ButtplugClientEvent
    },
    core::{connector::ButtplugInProcessClientConnectorBuilder, errors::ButtplugError},
    server::{
        device::hardware::communication::btleplug::BtlePlugCommunicationManagerBuilder,
        ButtplugServerBuilder,
    },
};
use futures::{Future, StreamExt};
use tokio::{runtime::Runtime};
use tracing::{debug, error, info, instrument, warn};

use crate::{util::Narrow, commands::{create_cmd_handling_thread, TkCommand}, Tk};

pub struct Telekinesis {
    pub runtime: Runtime,
    pub event_receiver: tokio::sync::mpsc::Receiver<TkEventEnum>,
    pub command_sender: tokio::sync::mpsc::Sender<TkCommand>,
}
impl fmt::Debug for Telekinesis 
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Telekinesis").finish()
    }
}

pub enum TkEventEnum {
    DeviceAdded(Arc<ButtplugClientDevice>),
    DeviceRemoved(Arc<ButtplugClientDevice>),
    DeviceVibrated(i32),
    DeviceStopped(i32),
    TkError(ButtplugError),
    Other(ButtplugClientEvent),
}

impl Display for TkEventEnum 
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let _ = match self {
            TkEventEnum::DeviceAdded(device) => write!(f, "Device '{}' connected.", device.name()),
            TkEventEnum::DeviceRemoved(device) => write!(f, "Device '{}' Removed.", device.name()),
            TkEventEnum::DeviceVibrated(speed)=> write!(f, "Vibrating '{}' devices.", speed),
            TkEventEnum::DeviceStopped(speed) => write!(f, "Stopping '{}' devices.", speed),
            TkEventEnum::TkError(err) => write!(f, "Error '{:?}'", err),
            TkEventEnum::Other(other) => write!(f, "{:?}", other),
        };
        Ok(())
    }
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

impl Telekinesis 
{
    pub fn new_with_default_settings() -> Result<Telekinesis, anyhow::Error> {
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
            Ok::<ButtplugClient, anyhow::Error>(client)
        })
    }

    pub fn new(
        fut: impl Future<Output = Result<ButtplugClient, anyhow::Error>>,
    ) -> Result<Telekinesis, anyhow::Error> {
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
}

impl Tk for Telekinesis {
    #[instrument]
    fn scan_for_devices(&self) -> bool {
        info!("Sending Command: Scan for devices");
        if let Err(_) = self.command_sender.blocking_send(TkCommand::TkScan) {
            error!("Failed to send vibrate_all"); // whats skyrim gonna do about it
            return false;
        }
        true
    }

    // TODO: Drop Messages if event queue has overflow to not force users to consume
    #[instrument]
    fn vibrate_all(&self, speed: f64) -> bool {
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
    fn vibrate_all_delayed(&self, speed: f64, duration: std::time::Duration) -> bool {
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
    fn stop_all(&self) -> bool {
        info!("Sending Command: Stop all");
        if let Err(_) = self.command_sender.blocking_send(TkCommand::TkStopAll) {
            error!("Failed to send stop_all");
            return false;
        }
        true
    }

    #[instrument]
    fn disconnect(&mut self) {
        info!("Sending Command: Disconnecting client");
        if let Err(_) = self.command_sender.blocking_send(TkCommand::TkDiscconect) {
            error!("Failed to send disconnect");
        }
    }

    #[instrument]
    fn get_next_event(&mut self) -> Option<TkEventEnum> {
        debug!("get_next_event");
        if let Ok(msg) = self.event_receiver.try_recv() {
            debug!("Got event {}", msg.to_string());
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
