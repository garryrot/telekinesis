use crate::{
    commands::{create_cmd_thread, TkAction},
    util::Narrow,
    Tk, TkEvent,
};
use buttplug::{
    client::ButtplugClient,
    core::connector::{ButtplugInProcessClientConnectorBuilder},
    server::{
        device::hardware::communication::btleplug::BtlePlugCommunicationManagerBuilder,
        ButtplugServerBuilder,
    },
};
use futures::{StreamExt};
use std::fmt::{self};
use tokio::runtime::Runtime;
use tracing::{debug, error, info, warn};

pub struct Telekinesis {
    pub event_receiver: tokio::sync::mpsc::Receiver<TkEvent>,
    pub command_sender: tokio::sync::mpsc::Sender<TkAction>,
    pub thread: Runtime,
}

impl Telekinesis {
    pub fn connect_with_default_settings() -> Result<Telekinesis, anyhow::Error> {
        let (event_sender, event_receiver) = tokio::sync::mpsc::channel(2048); // big, we dont know if client reads them fast
        let (command_sender, command_receiver) = tokio::sync::mpsc::channel(128); // small, we handle them immediately
        let runtime = Runtime::new()?;
        runtime.spawn(async move {
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

            info!("Main thread started");
            let mut events = buttplug.event_stream();
            create_cmd_thread(buttplug, event_sender.clone(), command_receiver);

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
    fn vibrate_all(&self, speed: f64) -> bool {
        info!("Sending Command: Vibrate all");
        if let Err(_) = self
            .command_sender
            .blocking_send(TkAction::TkVibrateAll(speed.narrow(0.0, 1.0)))
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
            .blocking_send(TkAction::TkVibrateAllDelayed(
                speed.narrow(0.0, 1.0),
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
