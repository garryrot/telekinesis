use buttplug::{client::{ButtplugClient, ScalarCommand}, core::message::ActuatorType};
use tokio::{runtime::Handle, select, time::sleep};
use tracing::{debug, error, info, span, Level};

use crate::{event::TkEvent, telekinesis::Speed};

#[derive(Debug)]
pub enum TkAction {
    TkScan,
    TkVibrateAll(Speed),
    TkVibrateAllDelayed(Speed, std::time::Duration),
    TkStopAll,
    TkDiscconect,
}

pub async fn cmd_scan_for_devices(client: &ButtplugClient) -> bool {
    if let Err(err) = client.start_scanning().await {
        error!(error = err.to_string(), "Failed scanning for devices.");
        return false;
    }
    true
}

pub async fn cmd_vibrate_all(client: &ButtplugClient, speed: Speed) -> i32 {
    let mut vibrated = 0;
    for device in client
        .devices()
        .iter()
        .filter(|d| d.message_attributes().scalar_cmd().is_some())
    {
        debug!("Vibrating device {} with speed {}", device.name(), speed);
        match device.scalar(&ScalarCommand::Scalar((speed.as_0_to_1_f64(), ActuatorType::Vibrate))).await {
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

pub fn create_cmd_thread(
    client: ButtplugClient,
    event_sender: tokio::sync::mpsc::UnboundedSender<TkEvent>,
    mut command_receiver: tokio::sync::mpsc::Receiver<TkAction>,
) {
    Handle::current().spawn(async move {
        info!("Comand handling thread started");
        let _ = span!(Level::INFO, "cmd_handling_thread").entered();
        let mut delayed_cmd: Option<TkAction> = None;
        loop {
            let recv_fut = command_receiver.recv();
            let cmd = if let Some(TkAction::TkVibrateAllDelayed(speed, duration)) = delayed_cmd {
                debug!("Select delayed command");
                select! {
                    () = sleep(duration) => Some(TkAction::TkVibrateAll(speed)),
                    cmd = recv_fut => cmd
                }
            } else {
                recv_fut.await
            };
            delayed_cmd = None; // always overwrite delayed with new command

            if let Some(cmd) = cmd {
                info!("Executing command {:?}", cmd);
                match cmd {
                    TkAction::TkScan => {
                        cmd_scan_for_devices(&client).await;
                    }
                    TkAction::TkVibrateAll(speed) => {
                        let vibrated = cmd_vibrate_all(&client, speed).await;
                        event_sender
                            .send(TkEvent::DeviceVibrated(vibrated))
                            .expect("Open");
                    }
                    TkAction::TkStopAll => {
                        let stopped = cmd_stop_all(&client).await;
                        event_sender
                            .send(TkEvent::DeviceStopped(stopped))
                            .expect("Open");
                    }
                    TkAction::TkDiscconect => {
                        client
                            .disconnect()
                            .await
                            .unwrap_or_else(|_| error!("Failed to disconnect."));
                    }
                    TkAction::TkVibrateAllDelayed(_, _) => {
                        delayed_cmd = Some(cmd);
                    }
                }
            } else {
                info!("Command stream closed");
                break;
            }
        }
    });
}
