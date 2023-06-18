use buttplug::{
    client::ButtplugClient,
    core::connector::ButtplugInProcessClientConnectorBuilder,
    server::{
        device::hardware::communication::btleplug::BtlePlugCommunicationManagerBuilder,
        ButtplugServerBuilder,
    },
};
use futures::{Future, StreamExt};
use std::{fmt::{self}};
use std::fmt::Display;
use tokio::{runtime::Runtime, sync::mpsc::channel, sync::mpsc::unbounded_channel};
use tracing::{debug, error, info, warn};

use crate::{
    commands::{create_cmd_thread, TkAction},
    util::Narrow,
    Tk, TkEvent,
};

pub struct Telekinesis {
    pub event_receiver: tokio::sync::mpsc::UnboundedReceiver<TkEvent>,
    pub command_sender: tokio::sync::mpsc::Sender<TkAction>,
    pub thread: Runtime,
}

pub async fn in_process_server() -> ButtplugClient {
    let connector = ButtplugInProcessClientConnectorBuilder::default()
        .server(
            ButtplugServerBuilder::default()
                .comm_manager(BtlePlugCommunicationManagerBuilder::default())
                .finish()
                .expect("Could not create in-process-server."),
        )
        .finish();

    let buttplug = ButtplugClient::new("Telekinesis");
    buttplug
        .connect(connector)
        .await
        .expect("Could not connect client");
    buttplug
}

impl Telekinesis {
    pub fn connect_with(
        connector: impl Future<Output = ButtplugClient> + Send + 'static,
    ) -> Result<Telekinesis, anyhow::Error> {
        let (event_sender, event_receiver) = unbounded_channel();
        let (command_sender, command_receiver) = channel(256); // we handle them immediately
        let runtime = Runtime::new()?;
        runtime.spawn(async move {
            info!("Main thread started");
            let buttplug = connector.await;
            let mut events = buttplug.event_stream();
            create_cmd_thread(buttplug, event_sender.clone(), command_receiver);

            while let Some(event) = events.next().await {
                event_sender
                    .send(TkEvent::from_event(event))
                    .unwrap_or_else(|_| warn!("Dropped event cause queue is full."));
            }
        });
        Ok(Telekinesis {
            command_sender: command_sender,
            event_receiver: event_receiver,
            thread: runtime,
        })
    }
}

impl fmt::Debug for Telekinesis {
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
    fn vibrate_all(&self, speed: Speed) -> bool {
        info!("Sending Command: Vibrate all");
        if let Err(_) = self
            .command_sender
            .blocking_send(TkAction::TkVibrateAll(speed))
        {
            error!("Failed to send vibrate_all");
            return false;
        }
        true
    }

    fn vibrate_all_delayed(&self, speed: Speed, duration: std::time::Duration) -> bool {
        info!("Sending Command: Vibrate all delayed");
        if let Err(_) = self
            .command_sender
            .blocking_send(TkAction::TkVibrateAllDelayed(
                speed,
                duration,
            ))
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
        if let Ok(msg) = self.event_receiver.try_recv() {
            debug!("Got event {}", msg.to_string());
            return Some(msg);
        }
        None
    }

    fn get_next_events(&mut self) -> Vec<TkEvent> {
        let mut events = vec![];
        while let Some(event) = self.get_next_event() {
            events.push(event);
            if events.len() >= 128 {
                break;
            }
        }
        events
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

#[derive(Debug, Clone, Copy)]
pub struct Speed {
    pub value: u16 
}
impl Speed {
    pub fn new(percentage: i64) -> Speed {
        Speed { 
            value: percentage.narrow(0, 100) as u16
        }
    }
    pub fn min() -> Speed {
        Speed { value: 0 }
    }
    pub fn as_0_to_1_f64(self) -> f64 {
        (self.value / 100) as f64
    } 
}
impl Display for Speed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}