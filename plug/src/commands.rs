use buttplug::client::{ButtplugClient, VibrateCommand};
use tokio::{select, runtime::Runtime, time::sleep};
use tracing::{error, info, span, debug, Level};

use crate::telekinesis::TkEventEnum;

#[derive(Debug)]
pub enum TkCommand {
    TkScan,
    TkVibrateAll(f64),
    TkVibrateAllDelayed(f64, std::time::Duration),
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

pub async fn cmd_vibrate_all(client: &ButtplugClient, speed: f64) -> i32 {
    let mut vibrated = 0;
    for device in client
        .devices()
        .iter()
        .filter(|d| d.message_attributes().scalar_cmd().is_some())
    {
        debug!("Vibrating device {} with speed {}", device.name(), speed);
        match device.vibrate(&VibrateCommand::Speed(speed)).await {
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
