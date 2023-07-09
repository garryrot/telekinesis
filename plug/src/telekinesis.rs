use buttplug::{
    client::{ButtplugClient, ButtplugClientDevice, ButtplugClientEvent},
    core::{
        connector::{
            ButtplugConnector, ButtplugInProcessClientConnector,
            ButtplugInProcessClientConnectorBuilder,
        },
        message::{ButtplugCurrentSpecClientMessage, ButtplugCurrentSpecServerMessage},
    },
    server::{
        device::hardware::communication::btleplug::BtlePlugCommunicationManagerBuilder,
        ButtplugServerBuilder,
    },
};
use futures::{Future, StreamExt};
use std::time::Instant;
use std::{
    fmt::{self},
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{runtime::Runtime, sync::mpsc::channel, sync::mpsc::unbounded_channel};
use tracing::{debug, error, info, warn};

use crate::{
    commands::{create_cmd_thread, TkAction, TkControl, TkDeviceAction, TkDeviceSelector},
    Speed, Tk, TkEvent,
};

pub struct Telekinesis {
    pub event_receiver: tokio::sync::mpsc::UnboundedReceiver<TkEvent>,
    pub command_sender: tokio::sync::mpsc::Sender<TkAction>,
    pub devices: Arc<Mutex<Vec<Arc<ButtplugClientDevice>>>>,
    pub thread: Runtime,
}

pub fn in_process_connector() -> ButtplugInProcessClientConnector {
    ButtplugInProcessClientConnectorBuilder::default()
        .server(
            ButtplugServerBuilder::default()
                .comm_manager(BtlePlugCommunicationManagerBuilder::default())
                .finish()
                .expect("Could not create in-process-server."),
        )
        .finish()
}

impl Telekinesis {
    pub fn connect_with<T, Fn, Fut>(connector_factory: Fn) -> Result<Telekinesis, anyhow::Error>
    where
        Fn: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = T> + Send,
        T: ButtplugConnector<ButtplugCurrentSpecClientMessage, ButtplugCurrentSpecServerMessage>
            + 'static,
    {
        let (event_sender, event_receiver) = unbounded_channel();
        let (command_sender, command_receiver) = channel(256); // we handle them immediately
        let devices = Arc::new(Mutex::new(vec![]));
        let devices_clone = devices.clone();

        let runtime = Runtime::new()?;
        runtime.spawn(async move {
            info!("Main thread started");
            let buttplug = with_connector(connector_factory().await).await;
            let mut events = buttplug.event_stream();
            create_cmd_thread(buttplug, event_sender.clone(), command_receiver);
            while let Some(event) = events.next().await {
                match event.clone() {
                    ButtplugClientEvent::DeviceAdded(device) => {
                        let mut device_list = devices_clone.lock().unwrap();
                        device_list.push(device);
                    }
                    ButtplugClientEvent::DeviceRemoved(device) => {
                        let mut device_list = devices_clone.lock().unwrap();
                        if let Some(i) =
                            device_list.iter().position(|x| x.index() == device.index())
                        {
                            device_list.remove(i);
                        }
                    }
                    _ => {}
                };
                event_sender
                    .send(TkEvent::from_event(event))
                    .unwrap_or_else(|_| warn!("Dropped event cause queue is full."));
            }
        });
        Ok(Telekinesis {
            command_sender: command_sender,
            event_receiver: event_receiver,
            devices: devices,
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
        if let Err(_) = self.command_sender.blocking_send(TkAction::Scan) {
            error!("Failed to start scan");
            return false;
        }
        true
    }

    fn vibrate(&self, speed: Speed, duration: Duration, devices: Vec<String>) -> bool {
        info!("Sending Command: Vibrate");
        if let Err(_) = self.command_sender.try_send(TkAction::Control(TkControl {
            devices: TkDeviceSelector::ByNames(Box::new(devices)),
            duration: duration,
            action: TkDeviceAction::Vibrate(speed),
        })) {
            error!("Failed to send vibrate");
            return false;
        }
        true
    }

    fn vibrate_all(&self, speed: Speed, duration: Duration) -> bool {
        info!("Sending Command: Vibrate");
        if let Err(_) = self.command_sender.try_send(TkAction::Control(TkControl {
            devices: TkDeviceSelector::All,
            duration: duration,
            action: TkDeviceAction::Vibrate(speed),
        })) {
            error!("Failed to send vibrate");
            return false;
        }
        true
    }

    fn stop_all(&self) -> bool {
        info!("Sending Command: Stop all");
        if let Err(_) = self.command_sender.blocking_send(TkAction::StopAll) {
            error!("Failed to send stop_all");
            return false;
        }
        true
    }

    fn disconnect(&mut self) {
        info!("Sending Command: Disconnecting client");
        if let Err(_) = self.command_sender.blocking_send(TkAction::Disconect) {
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
}

async fn with_connector<T>(connector: T) -> ButtplugClient
where
    T: ButtplugConnector<ButtplugCurrentSpecClientMessage, ButtplugCurrentSpecServerMessage>
        + 'static,
{
    let buttplug = ButtplugClient::new("Telekinesis");
    let bp = buttplug.connect(connector).await;
    match bp {
        Ok(_) => {
            info!("Connected client.")
        }
        Err(err) => {
            error!("Could not connect client. Error: {}.", err);
        }
    }
    buttplug
}

#[cfg(test)]
mod tests {
    use std::{thread, time::Duration, vec};

    use crate::{
        fakes::{scalar, FakeConnectorCallRegistry, FakeDeviceConnector},
        util::assert_timeout,
    };
    use buttplug::core::message::{ActuatorType, DeviceAdded};
    use lazy_static::__Deref;

    use super::*;

    impl Telekinesis {
        pub fn await_connect(&self, devices: usize) {
            assert_timeout!(
                self.devices.deref().lock().unwrap().deref().len() == devices,
                "Awaiting connect"
            );
        }
    }

    #[test]
    fn test_connection_assert_devices_connect() {
        // arrange
        let (tk, _) = wait_for_connection(vec![
            scalar(1, "vib1", ActuatorType::Vibrate),
            scalar(2, "vib2", ActuatorType::Inflate),
        ]);

        // assert
        assert_timeout!(
            tk.devices.deref().lock().unwrap().deref().len() == 2,
            "Enough devices connected"
        );
    }

    #[test]
    fn vibrate_all_demo_only_vibrates_vibrators() {
        // arrange
        let (connector, call_registry) = FakeDeviceConnector::device_demo();
        let count = connector.devices.len();

        // act
        let tk = Telekinesis::connect_with(|| async move { connector }).unwrap();
        tk.await_connect(count);
        tk.vibrate_all(Speed::new(100), Duration::from_millis(1));

        // assert
        call_registry.assert_vibrated(1); // scalar
        call_registry.assert_not_vibrated(4); // linear 
        call_registry.assert_not_vibrated(7); // rotator
    }

    #[test]
    fn vibrate_all_only_vibrates_vibrators() {
        // arrange
        let (tk, call_registry) = wait_for_connection(vec![
            scalar(1, "vib1", ActuatorType::Vibrate),
            scalar(2, "vib2", ActuatorType::Inflate),
        ]);

        tk.vibrate_all(Speed::new(100), Duration::from_millis(1));

        // assert
        call_registry.assert_vibrated(1);
        call_registry.assert_not_vibrated(2);
    }

    #[test]
    fn vibrate_select_non_existing_devices() {
        // arrange
        let (tk, call_registry) =
            wait_for_connection(vec![scalar(1, "vib1", ActuatorType::Vibrate)]);

        // act
        tk.vibrate(
            Speed::max(),
            Duration::from_millis(1),
            vec![String::from("does not exist")],
        );
        thread::sleep(Duration::from_millis(50));

        // assert
        call_registry.assert_not_vibrated(1);
    }

    #[test]
    fn vibrate_select_single_device() {
        // arrange
        let (tk, call_registry) =
            wait_for_connection(vec![scalar(1, "vib1", ActuatorType::Vibrate)]);

        // act
        tk.vibrate(
            Speed::max(),
            Duration::from_millis(1),
            vec![String::from("vib1")],
        );
        thread::sleep(Duration::from_secs(1));

        // assert
        call_registry.assert_vibrated(1);
    }

    #[test]
    fn vibrate_multiple_existing_devices_vibrate() {
        // arrange
        let (tk, call_registry) = wait_for_connection(vec![
            scalar(1, "vib1", ActuatorType::Vibrate),
            scalar(2, "vib2", ActuatorType::Vibrate),
            scalar(3, "vib3", ActuatorType::Vibrate),
        ]);

        // act
        tk.vibrate(
            Speed::max(),
            Duration::from_millis(1),
            vec![String::from("vib1"), String::from("vib3")],
        );
        thread::sleep(Duration::from_secs(1));

        // assert
        call_registry.assert_vibrated(1);
        call_registry.assert_vibrated(3);
        call_registry.assert_not_vibrated(2);
    }

    fn wait_for_connection(devices: Vec<DeviceAdded>) -> (Telekinesis, FakeConnectorCallRegistry) {
        let (connector, call_registry) = FakeDeviceConnector::new(devices);
        let count = connector.devices.len();

        // act
        let tk = Telekinesis::connect_with(|| async move { connector }).unwrap();
        tk.await_connect(count);
        (tk, call_registry)
    }
}
