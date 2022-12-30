use std::ffi::c_float;

use buttplug::{
    client::{ButtplugClient, ButtplugClientError, ButtplugClientEvent, VibrateCommand},
    server::ButtplugServerError,
};
use futures::{Future, StreamExt};
use tokio::runtime::Runtime;
use tracing::{error, info};

pub struct Telekinesis {
    pub client: ButtplugClient,
    pub runtime: Runtime,
    pub event_receiver: tokio::sync::mpsc::Receiver<ButtplugClientEvent>,
}

pub enum TkError {
    ServerError(ButtplugServerError),
    ClientError(ButtplugClientError),
}

impl From<ButtplugServerError> for TkError {
    fn from(e: ButtplugServerError) -> Self {
        TkError::ServerError(e)
    }
}
impl From<ButtplugClientError> for TkError {
    fn from(e: ButtplugClientError) -> Self {
        TkError::ClientError(e)
    }
}

pub trait TkEvent {
    fn as_string(&self) -> String;
}

impl TkEvent for ButtplugClientEvent {
    fn as_string(&self) -> String {
        match self {
            ButtplugClientEvent::DeviceAdded(device) => {
                format!( "Device '{}' connected.", device.name() )
            }
            ButtplugClientEvent::DeviceRemoved(device) => {
                format!("Device '{}' Removed.", device.name())
            }
            ButtplugClientEvent::ScanningFinished => {
                String::from("Device scanning is finished!")
            }
            ButtplugClientEvent::PingTimeout => String::from("Ping Timeout"),
            ButtplugClientEvent::ServerConnect => String::from("Server Connect"),
            ButtplugClientEvent::ServerDisconnect => String::from("Server Disconnect"),
            ButtplugClientEvent::Error(err) => err.to_string(),
        }
    }
}

pub fn create_event_handling_thread( runtime: &Runtime, client: &ButtplugClient) -> tokio::sync::mpsc::Receiver<ButtplugClientEvent> {
    let (event_sender, event_receiver) = tokio::sync::mpsc::channel(4096);
    let mut events = client.event_stream();
    runtime.spawn(async move {
        info!("Event polling thread started");
        while let Some(event) = events.next().await {
            event_sender.send(event).await.expect("channel full")
        }
    });
    event_receiver
}

impl Telekinesis {
    pub fn new(
        fut: impl Future<Output = Result<ButtplugClient, TkError>>,
    ) -> Result<Telekinesis, TkError> {
        let runtime = Runtime::new().unwrap();
        let client = runtime.block_on(fut)?;
        let event_receiver = create_event_handling_thread(&runtime, &client);
        Ok(Telekinesis {
            client: client,
            runtime: runtime,
            event_receiver: event_receiver,
        })
    }

    pub fn scan_for_devices(&self) -> bool {
        info!("Scanning for devices.");
        if let Err(err) = self
            .runtime
            .block_on(async { self.client.start_scanning().await })
        {
            error!(error = err.to_string(), "Failed scanning for devices.");
            return false;
        }
        return true;
    }

    pub fn vibrate_all(&self, speed: c_float) -> i32 {
        self.runtime.block_on(async {
            let mut vibrated = 0;
            for device in self
                .client
                .devices()
                .iter()
                .filter(|d| d.message_attributes().scalar_cmd().is_some())
            {
                info!("Device can vibrate. Setting vibration speed to {}.", speed);
                match device.vibrate(&VibrateCommand::Speed(speed.into())).await {
                    Ok(_) => vibrated += 1,
                    Err(err) => error!(
                        dev = device.name(),
                        error = err.to_string(),
                        "Failed to set device vibration speed."
                    ),
                }
            }
            vibrated
        })
    }

    pub fn stop_all(&self) -> i32 {
        self.runtime.block_on(async {
            let mut stopped = 0;
            for device in self.client.devices() {
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
        })
    }

    pub fn get_next_event(&mut self) -> Vec<ButtplugClientEvent> {
        let mut strings: Vec<ButtplugClientEvent> = vec![];
        if let Ok(msg) = self.event_receiver.try_recv() {
            strings.push(msg);
        }
        strings
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

    pub fn tk_close(&self) {
        info!("Disconnecting client.");
        self.runtime.block_on(async {
            self.client.disconnect();
        });
    }
}
