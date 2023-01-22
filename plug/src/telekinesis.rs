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
use crate::{util::Narrow, commands::{create_cmd_handling_thread, TkAction}, Tk};

pub struct Telekinesis {
    pub runtime: Runtime,
    pub event_receiver: tokio::sync::mpsc::Receiver<TkEvent>,
    pub command_sender: tokio::sync::mpsc::Sender<TkAction>,
}

pub enum TkEvent {
    DeviceAdded(Arc<ButtplugClientDevice>),
    DeviceRemoved(Arc<ButtplugClientDevice>),
    DeviceVibrated(i32),
    DeviceStopped(i32),
    TkError(ButtplugError),
    Other(ButtplugClientEvent),
}

impl Display for TkEvent 
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let _ = match self {
            TkEvent::DeviceAdded(device) => write!(f, "Device '{}' connected.", device.name()),
            TkEvent::DeviceRemoved(device) => write!(f, "Device '{}' Removed.", device.name()),
            TkEvent::DeviceVibrated(speed)=> write!(f, "Vibrating '{}' devices.", speed),
            TkEvent::DeviceStopped(speed) => write!(f, "Stopping '{}' devices.", speed),
            TkEvent::TkError(err) => write!(f, "Error '{:?}'", err),
            TkEvent::Other(other) => write!(f, "{:?}", other),
        };
        Ok(())
    }
}

impl TkEvent {
    fn from_event(event: ButtplugClientEvent) -> TkEvent {
        match event {
            ButtplugClientEvent::DeviceAdded(device) => TkEvent::DeviceAdded(device),
            ButtplugClientEvent::DeviceRemoved(device) => TkEvent::DeviceRemoved(device),
            ButtplugClientEvent::Error(err) => TkEvent::TkError(err),
            other => TkEvent::Other(other),
        }
    }
}

pub fn create_event_handling_thread(
    runtime: &Runtime,
    client: &ButtplugClient,
) -> (
    tokio::sync::mpsc::Receiver<TkEvent>,
    tokio::sync::mpsc::Sender<TkEvent>,
) {
    let (event_sender, event_receiver) = tokio::sync::mpsc::channel(2048); // big in case events are not consumed
    let sender_clone = event_sender.clone();
    let mut events = client.event_stream();
    runtime.spawn(async move {
        info!("Event polling thread started");
        while let Some(event) = events.next().await {
            event_sender
                .send(TkEvent::from_event(event))
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

impl fmt::Debug for Telekinesis 
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Telekinesis").finish()
    }
}

impl Tk for Telekinesis {
    #[instrument]
    fn scan_for_devices(&self) -> bool {
        info!("Sending Command: Scan for devices");
        if let Err(_) = self.command_sender.blocking_send(TkAction::TkScan) {
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
            .blocking_send(TkAction::TkVibrateAll( speed.narrow( 0.0, 1.0 )))
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
            .blocking_send(TkAction::TkVibrateAllDelayed( speed.narrow( 0.0, 1.0 ), duration))
        {
            error!("Failed to send delayed command");
            return false;
        }
        true
    }

    #[instrument]
    fn stop_all(&self) -> bool {
        info!("Sending Command: Stop all");
        if let Err(_) = self.command_sender.blocking_send(TkAction::TkStopAll) {
            error!("Failed to send stop_all");
            return false;
        }
        true
    }

    #[instrument]
    fn disconnect(&mut self) {
        info!("Sending Command: Disconnecting client");
        if let Err(_) = self.command_sender.blocking_send(TkAction::TkDiscconect) {
            error!("Failed to send disconnect");
        }
    }

    #[instrument]
    fn get_next_event(&mut self) -> Option<TkEvent> {
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
