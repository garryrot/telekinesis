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
use tracing::{debug, error, info, warn};
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


impl Telekinesis 
{
    pub fn connect_with_default_settings() -> Result<Telekinesis, anyhow::Error> {
        info!("Connecting with defualt settings");
        Telekinesis::connect(async {
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
    
    pub fn connect(fn_create_client: impl Future<Output = Result<ButtplugClient, anyhow::Error>> + std::marker::Send + 'static) -> Result<Telekinesis, anyhow::Error> {
        let (event_sender, event_receiver) = tokio::sync::mpsc::channel(2048); // big, cause we dont know if client reads them fast
        let (command_sender, command_receiver) = tokio::sync::mpsc::channel(128); // small cause we handle them immediately
        let runtime = Runtime::new()?;
        runtime.spawn(async move {
            let res = fn_create_client.await;
            if let Err(e) = res {
                error!("Could not create buttplug client: {}", e);
                return
            }
            info!("Event reading thread started");
            let client = res.unwrap();
            let mut events = client.event_stream();
            create_cmd_thread(client, event_sender.clone(), command_receiver);
            while let Some(event) = events.next().await {
                event_sender
                    .send(TkEvent::from_event(event))
                    .await
                    .unwrap_or_else(|_| warn!("Dropped event cause queue is full."));
            }
        });
        Ok(Telekinesis {
            command_sender: command_sender,
            event_receiver: event_receiver,
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
    fn scan_for_devices(&self) -> bool {
        info!("Sending Command: Scan for devices");
        if let Err(_) = self.command_sender.blocking_send(TkAction::TkScan) {
            error!("Failed to send vibrate_all"); // whats skyrim gonna do about it
            return false;
        }
        true
    }

    // TODO: Drop messages if event queue is full
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

    fn stop_all(&self) -> bool {
        info!("Sending Command: Stop all");
        if let Err(_) = self.command_sender.blocking_send(TkAction::TkStopAll) {
            error!("Failed to send stop_all");
            return false;
        }
        true
    }

    fn disconnect(&mut self) {
        info!("Sending Command: Disconnecting client");
        if let Err(_) = self.command_sender.blocking_send(TkAction::TkDiscconect) {
            error!("Failed to send disconnect");
        }
    }

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
