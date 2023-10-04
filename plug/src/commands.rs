use std::sync::Arc;

use buttplug::client::{ButtplugClient, ButtplugClientDevice};
use tokio::runtime::Handle;
use tracing::{error, info, span, Level};

use crate::{
    event::TkEvent,
    Speed
};


#[derive(Clone, Debug)]
pub enum TkAction {
    Scan,
    StopScan,
    StopAll,
    Disconect,
}


#[derive(Clone, Debug)]
pub enum TkDeviceAction {
    Start(Arc<ButtplugClientDevice>, Speed, bool, i32),
    Update(Arc<ButtplugClientDevice>, Speed),
    End(Arc<ButtplugClientDevice>, bool, i32),
    StopAll, // global but required for resetting device state
}

pub fn create_cmd_thread(
    event_sender: tokio::sync::mpsc::UnboundedSender<TkEvent>,
    mut command_receiver: tokio::sync::mpsc::Receiver<TkAction>,
    client: ButtplugClient
) {
    Handle::current().spawn(async move {
        info!("Comand handling thread started");
        let _ = span!(Level::INFO, "cmd_handling_thread").entered();

        // global operations and long running pattern execution
        loop {
            let next_cmd = command_receiver.recv().await;
            if let Some(cmd) = next_cmd {
                let queue_full_err = "Event sender full";
                info!("Executing command {:?}", cmd);
                match cmd {
                    TkAction::Scan => {
                        if let Err(err) = client.start_scanning().await {
                            error!(error = err.to_string(), "Failed scanning for devices.");
                            event_sender
                                .send(TkEvent::ScanFailed(err))
                                .unwrap_or_else(|_| error!(queue_full_err));
                        } else {
                            event_sender
                                .send(TkEvent::ScanStarted)
                                .unwrap_or_else(|_| error!(queue_full_err))
                        }

                    }
                    TkAction::StopScan => {
                        if let Err(err) = client.stop_scanning().await {
                            error!(
                                error = err.to_string(),
                                "Failed to stop scanning for devices."
                            );
                        } else {
                            event_sender
                                .send(TkEvent::ScanStopped)
                                .unwrap_or_else(|_| error!(queue_full_err));
                        }
                    }
                    TkAction::Disconect => {
                        client
                            .disconnect()
                            .await
                            .unwrap_or_else(|_| error!("Failed to disconnect."));
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
}
