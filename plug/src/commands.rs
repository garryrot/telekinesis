use std::time::Duration;

use buttplug::{client::{ButtplugClient, ScalarValueCommand}};
use tokio::{runtime::Handle, select, time::sleep};
use tracing::{debug, error, info, span, Level};

use crate::{event::TkEvent, Speed};

type DeviceNameList = Box<Vec<String>>;


#[derive(Clone, Debug)]
pub enum TkAction {
    Scan,
    Control(TkControl),
    StopAll,
    Disconect
}

#[derive(Clone, Debug)]
pub struct TkControl 
{
    pub duration: Duration,
    pub devices: TkDeviceSelector,
    pub action: TkDeviceAction,
}

#[derive(Clone, Debug)]
pub enum TkDeviceSelector {
    All,
    ByNames(DeviceNameList)
}

#[derive(Clone, Debug)]
pub enum TkDeviceAction
{
    Vibrate(Speed),
    VibratePattern(String)
}

pub async fn cmd_scan_for_devices(client: &ButtplugClient) -> bool {
    if let Err(err) = client.start_scanning().await {
        error!(error = err.to_string(), "Failed scanning for devices.");
        return false;
    }
    true
}

pub async fn cmd_vibrate_all(client: &ButtplugClient, command: ScalarValueCommand) -> i32 {
    let mut vibrated = 0;
    for device in client
        .devices()
        .iter()
        .filter(|d| d.message_attributes().scalar_cmd().is_some())
    {
        match command {
            ScalarValueCommand::ScalarValue(speed) => {
                debug!("Vibrating device {} with speed {}", device.name(), speed);
                match device.vibrate(&command).await {
                    Ok(_) => vibrated += 1,
                    Err(err) => error!(
                        dev = device.name(),
                        error = err.to_string(),
                        "Failed to set device vibration speed."
                    ),
                }
            },
            _ => todo!(),
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
        let mut delayed_timer: Duration = Duration::ZERO;
        loop {
            let recv_fut = command_receiver.recv();
            let cmd = if let Some(TkAction::Control(control)) = delayed_cmd {
                debug!("Select delayed command");
                select! {
                    () = sleep(delayed_timer) => Some(TkAction::Control(control)),
                    cmd = recv_fut => cmd
                }
            } else {
                recv_fut.await
            };
            delayed_cmd = None; // always overwrite delayed with new command

            if let Some(cmd) = cmd {
                info!("Executing command {:?}", cmd);
                match cmd {
                    TkAction::Scan => { 
                        cmd_scan_for_devices(&client).await; 
                    },
                    TkAction::StopAll => {
                        let stopped = cmd_stop_all(&client).await;
                        event_sender
                            .send(TkEvent::DeviceStopped(stopped))
                            .expect("Open");
                    },
                    TkAction::Disconect => {
                        client
                            .disconnect()
                            .await
                            .unwrap_or_else(|_| error!("Failed to disconnect."));
                    },
                    TkAction::Control(control) => {
                        match control.action {
                            TkDeviceAction::Vibrate(speed) => {
                                let vibrated = cmd_vibrate_all(&client, ScalarValueCommand::ScalarValue(speed.as_0_to_1_f64())).await;
                                event_sender
                                    .send(TkEvent::DeviceVibrated(vibrated, speed))
                                    .expect("Open");
                                if ! control.duration.is_zero() {
                                    delayed_timer = control.duration;
                                    delayed_cmd = Some( TkAction::Control(
                                        TkControl { 
                                            duration: Duration::ZERO, // control.duration, 
                                            devices: control.devices.clone(), 
                                            action: TkDeviceAction::Vibrate(Speed::min())
                                        }) );
                                }
                            }
                            _ => todo!(),
                        }
                    }
                }
            } else {
                info!("Command stream closed");
                break;
            }
        }
    });
}
