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
use crate::{util::Narrow, commands::{create_cmd_thread, TkAction}, Tk};

pub struct Telekinesis {
    pub event_receiver: tokio::sync::mpsc::Receiver<TkEvent>,
    pub command_sender: tokio::sync::mpsc::Sender<TkAction>,
    pub thread: Runtime
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

pub fn create_main_thread(fn_create_client: impl Future<Output = Result<ButtplugClient, anyhow::Error>> + std::marker::Send + 'static)
-> (tokio::sync::mpsc::Receiver<TkEvent>, tokio::sync::mpsc::Sender<TkAction>, Runtime) {
    let (event_sender, event_receiver) = tokio::sync::mpsc::channel(2048); 
    let (command_sender, command_receiver) = tokio::sync::mpsc::channel(128);

    let runtime = Runtime::new().unwrap();
    runtime.spawn(async move {
        info!("Event polling thread started");
        match fn_create_client.await {
            Ok(client) => {
                let mut events = client.event_stream();
                let sender_clone_cmd = event_sender.clone();
                create_cmd_thread(client, sender_clone_cmd, command_receiver);
                while let Some(event) = events.next().await {
                    event_sender
                        .send(TkEvent::from_event(event))
                        .await
                        .unwrap_or_else(|_| warn!("Dropped event cause queue is full."));
                }
            }
            Err(err) => error!("Could not create buttplug client: {}", err.to_string()),
        }
    });
    (event_receiver, command_sender, runtime)
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

    pub fn new(fut: impl Future<Output = Result<ButtplugClient, anyhow::Error>> + std::marker::Send + 'static) -> Result<Telekinesis, anyhow::Error> {
        let (event_receiver, command_sender, runtime) = create_main_thread(fut);
        Ok(Telekinesis {
            event_receiver: event_receiver,
            command_sender: command_sender,
            thread: runtime
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
