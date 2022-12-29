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
    messages: tokio::sync::mpsc::Receiver<String>,
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

impl Telekinesis {
    pub fn connect_with(
        fut: impl Future<Output = Result<ButtplugClient, TkError>>,
    ) -> Result<Telekinesis, TkError> {
        let runtime = Runtime::new().unwrap();
        let (tx, rx) = tokio::sync::mpsc::channel(1024);

        let client = runtime.block_on(fut)?;
        let mut events = client.event_stream();
        let recv = rx;
        runtime.spawn(async move {
            while let Some(event) = events.next().await {
                match event {
                    ButtplugClientEvent::DeviceAdded(device) => {
                        info!("Device {} Connected!", device.name());
                        tx.send(format!("Device {} Connected!", device.name()))
                            .await
                            .unwrap();
                    }
                    ButtplugClientEvent::DeviceRemoved(device) => {
                        info!("Device {} Removed!", device.name());
                        tx.send(format!("Device {} Removed!", device.name()))
                            .await
                            .unwrap();
                    }
                    ButtplugClientEvent::ScanningFinished => {
                        info!("Device scanning is finished!");
                        tx.send(format!("Device scanning is finished!"))
                            .await
                            .unwrap();
                    }
                    _ => {}
                }
            }
        });

        Ok(Telekinesis {
            client: client,
            runtime: runtime,
            messages: recv,
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

    pub fn get_next_event(&mut self) -> Vec<String> {
        let mut strings: Vec<String> = vec![];
        if let Ok(msg) = self.messages.try_recv() {
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
