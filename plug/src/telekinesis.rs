use anyhow::Error;
use buttplug::{
    client::{ButtplugClient, ButtplugClientDevice},
    core::{
        connector::{
            new_json_ws_client_connector, ButtplugConnector,
            ButtplugInProcessClientConnectorBuilder,
        },
        message::{
            ActuatorType, ButtplugCurrentSpecClientMessage, ButtplugCurrentSpecServerMessage,
        },
    },
    server::{
        device::hardware::communication::btleplug::BtlePlugCommunicationManagerBuilder,
        ButtplugServerBuilder,
    },
};
use futures::Future;

use std::{
    fmt::{self},
    sync::{Arc, Mutex},
    time::Instant,
};
use tokio::{runtime::Runtime, sync::mpsc::channel, sync::mpsc::unbounded_channel};
use tracing::{debug, error, info};

use itertools::Itertools;

use crate::{
    connection::{
        handle_connection, TkAction, TkConnectionEvent, TkConnectionStatus, TkDeviceEvent, TkDeviceStatus,
    },
    input::TkParams,
    pattern::{TkButtplugScheduler, TkPlayerSettings},
    settings::{TkConnectionType, TkSettings},
    DeviceList, Speed, Telekinesis, Tk, TkDuration, TkPattern, TkStatus,
};

pub static ERROR_HANDLE: i32 = -1;

pub fn in_process_connector(
) -> impl ButtplugConnector<ButtplugCurrentSpecClientMessage, ButtplugCurrentSpecServerMessage> {
    ButtplugInProcessClientConnectorBuilder::default()
        .server(
            ButtplugServerBuilder::default()
                .comm_manager(BtlePlugCommunicationManagerBuilder::default())
                .finish()
                .expect("Could not create in-process-server."), // TODO log error instead of panic
        )
        .finish()
}

impl Telekinesis {
    pub fn connect_with<T, Fn, Fut>(
        connector_factory: Fn,
        provided_settings: Option<TkSettings>,
    ) -> Result<Telekinesis, anyhow::Error>
    where
        Fn: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = T> + Send,
        T: ButtplugConnector<ButtplugCurrentSpecClientMessage, ButtplugCurrentSpecServerMessage>
            + 'static,
    {
        let runtime = Runtime::new()?;

        let (event_sender, event_receiver) = unbounded_channel();
        let (command_sender, command_receiver) = channel(256); // we handle them immediately
        let event_sender_clone = event_sender.clone();

        let settings = provided_settings.or(Some(TkSettings::default())).unwrap();
        let pattern_path = settings.pattern_path.clone();

        let connection_status = Arc::new(Mutex::new(TkStatus::new())); // ugly
        let status_clone = connection_status.clone();
        runtime.spawn(async move {
            info!("Starting connection");
            let client = with_connector(connector_factory().await).await;
            handle_connection(event_sender, command_receiver, client, connection_status).await;
        });

        let (scheduler, action_receiver) = TkButtplugScheduler::create(TkPlayerSettings {
            player_resolution_ms: 100,
            pattern_path,
        });

        runtime.spawn(async move {
            TkButtplugScheduler::run_worker_thread(action_receiver).await;
            info!("Worker closed")
        });

        Ok(Telekinesis {
            command_sender: command_sender,
            connection_events: event_receiver,
            runtime: runtime,
            settings: settings,
            connection_status: status_clone,
            scheduler: scheduler,
            event_sender: event_sender_clone,
        })
    }
}

impl fmt::Debug for Telekinesis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Telekinesis").finish()
    }
}

impl Tk for Telekinesis {
    fn get_device(&self, device_name: &str) -> Option<Arc<ButtplugClientDevice>> {
        if let Some(status) = self.get_device_status(device_name) {
            return Some(status.device);
        }
        None
    }
   
    fn get_devices(&self) -> DeviceList {
        if let Ok(connection_status) = self.connection_status.try_lock() {
            let devices: DeviceList = connection_status
                .device_status
                .values()
                .into_iter()
                .map(|value| value.device.clone() )
                .collect();
            return devices;
        } else {
            error!("Error accessing device map");
        }
        vec![]
    }

    fn get_device_status(&self, device_name: &str) -> Option<TkDeviceStatus> {
        if let Ok(status) = self.connection_status.try_lock() {
            let devices = status
                .device_status
                .values()
                .into_iter()
                .filter(|d| d.device.name() == device_name )
                .map(|value| value.clone() )
                .next();
            return devices;
        } else {
            error!("Error accessing device map");
        }
        None
    }
    
    fn get_known_device_names(&self) -> Vec<String> {
        debug!("Getting devices names");
        self.get_devices()
            .iter()
            .map(|d| d.name().clone())
            .chain(self.settings.devices.iter().map(|d| d.name.clone()))
            .into_iter()
            .unique()
            .collect()
    }

    fn connect(settings: TkSettings) -> Result<Telekinesis, Error> {
        let settings_clone = settings.clone();
        match settings.connection {
            TkConnectionType::WebSocket(endpoint) => {
                let uri = format!("ws://{}", endpoint);
                info!("Connecting Websocket: {}", uri);
                Telekinesis::connect_with(
                    || async move { new_json_ws_client_connector(&uri) },
                    Some(settings_clone),
                )
            }
            _ => {
                info!("Connecting In-Process");
                Telekinesis::connect_with(|| async move { in_process_connector() }, Some(settings))
            }
        }
    }

    fn scan_for_devices(&self) -> bool {
        info!("Sending Command: Scan for devices");
        if let Err(_) = self.command_sender.try_send(TkAction::Scan) {
            error!("Failed to start scan");
            return false;
        }
        true
    }

    fn stop_scan(&self) -> bool {
        info!("Sending Command: Stop scan");
        if let Err(_) = self.command_sender.try_send(TkAction::StopScan) {
            error!("Failed to stop scan");
            return false;
        }
        true
    }

    fn get_device_capabilities(&self, name: &str) -> Vec<String> {
        debug!("Getting '{}' capabilities", name);
        // maybe just return all actuator + types + linear + rotate
        if self
            .get_devices()
            .iter()
            .filter(|d| d.name() == name)
            .any(|device| {
                if let Some(scalar) = device.message_attributes().scalar_cmd() {
                    if scalar
                        .iter()
                        .any(|a| *a.actuator_type() == ActuatorType::Vibrate)
                    {
                        return true;
                    }
                }
                false
            })
        {
            return vec![ActuatorType::Vibrate.to_string()];
        }
        vec![]
    }
    
    fn get_device_connection_status(&self, device_name: &str) -> TkConnectionStatus {
        debug!("Getting '{}' connected", device_name);
        if let Some(status) = self.get_device_status(device_name) {
            return status.status
        }
        TkConnectionStatus::NotConnected
    }

    fn vibrate(&mut self, speed: Speed, duration: TkDuration, events: Vec<String>) -> i32 {
        self.vibrate_pattern(TkPattern::Linear(duration, speed), events)
    }

    fn vibrate_pattern(&mut self, pattern: TkPattern, events: Vec<String>) -> i32 {
        info!("Received: Vibrate/Vibrate Pattern");
        let params = TkParams::from_input(events.clone(), pattern, &self.settings.devices);

        let mut player = self
            .scheduler
            .create_player(params.filter_devices(self.get_devices()));
        let handle = player.handle;

        let sender_clone = self.event_sender.clone();
        self.runtime.spawn(async move {
            let now = Instant::now();
            player.play(params.pattern.clone()).await;
            sender_clone
                .send(TkConnectionEvent::DeviceEvent(TkDeviceEvent::new(
                    now.elapsed(),
                    &player.devices,
                    params,
                )))
                .expect("queue full");
        });
        handle
    }

    fn stop(&mut self, handle: i32) -> bool {
        info!("Received: Stop");
        self.scheduler.stop_task(handle);
        true
    }

    fn stop_all(&mut self) -> bool {
        info!("Received: Stop All");
        self.scheduler.stop_all();
        if let Err(_) = self.command_sender.try_send(TkAction::StopAll) {
            error!("Failed to queue stop_all");
            return false;
        }
        true
    }

    fn disconnect(&mut self) {
        info!("Sending Command: Disconnecting client");
        if let Err(_) = self.command_sender.try_send(TkAction::Disconect) {
            error!("Failed to send disconnect");
        }
    }

    fn get_next_event(&mut self) -> Option<TkConnectionEvent> {
        if let Ok(msg) = self.connection_events.try_recv() {
            debug!("Get event {:?}", msg);
            return Some(msg);
        }
        None
    }

    fn process_next_events(&mut self) -> Vec<TkConnectionEvent> {
        debug!("Polling all events");
        let mut events = vec![];
        while let Some(event) = self.get_next_event() {
            events.push(event);
            if events.len() >= 128 {
                break;
            }
        }
        events
    }

    fn settings_set_enabled(&mut self, device_name: &str, enabled: bool) {
        info!("Setting '{}'.enabled={}", device_name, enabled);

        let mut settings = self.settings.clone();
        settings.set_enabled(device_name, enabled);
        self.settings = settings;
    }

    fn settings_set_events(&mut self, device_name: &str, events: Vec<String>) {
        info!("Setting '{}'.events={:?}", device_name, events);

        let settings = self.settings.clone();
        self.settings = settings.set_events(device_name, events);
    }

    fn settings_get_events(&self, device_name: &str) -> Vec<String> {
        self.settings.get_events(device_name)
    }

    fn settings_get_enabled(&self, device_name: &str) -> bool {
        let enabled = self.settings.is_enabled(device_name);
        debug!("Getting setting '{}'.enabled={}", device_name, enabled);
        enabled
    }

    fn get_connection_status(&self) -> TkConnectionStatus {
        if let Ok(status) = self.connection_status.try_lock() {
            return status.connection_status.clone();
        }
        TkConnectionStatus::NotConnected
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
    use std::time::Instant;
    use std::{thread, time::Duration, vec};

    use crate::connection::TkConnectionStatus;
    use crate::util::enable_log;
    use crate::{
        fakes::{linear, scalar, FakeConnectorCallRegistry, FakeDeviceConnector},
        util::assert_timeout,
    };
    use buttplug::core::message::{ActuatorType, DeviceAdded};

    use super::*;

    impl Telekinesis {
        /// should only be used by tests or fake backends
        pub fn await_connect(&self, devices: usize) {
            assert_timeout!(
                self.connection_status.lock().unwrap().device_status.len() == devices,
                "Awaiting connect"
            );
        }
    }

    #[test]
    fn get_devices_contains_connected_devices() {
        // arrange
        let (tk, _) = wait_for_connection(vec![
            scalar(1, "vib1", ActuatorType::Vibrate),
            scalar(2, "vib2", ActuatorType::Inflate),
        ]);

        // assert
        assert_timeout!(
            tk.connection_status.lock().unwrap().device_status.len() == 2,
            "Enough devices connected"
        );
        assert!(
            tk.get_known_device_names().contains(&String::from("vib1")),
            "Contains name vib1"
        );
        assert!(
            tk.get_known_device_names().contains(&String::from("vib2")),
            "Contains name vib2"
        );
    }

    #[test]
    fn get_devices_contains_devices_from_settings() {
        let (mut tk, _) = wait_for_connection(vec![]);
        tk.settings_set_enabled("foreign", true);
        assert!(
            tk.get_known_device_names().contains(&String::from("foreign")),
            "Contains additional device from settings"
        );
    }

    #[test]
    fn vibrate_all_demo_only_vibrates_vibrators() {
        // arrange
        let (connector, call_registry) = FakeDeviceConnector::device_demo();
        let count = connector.devices.len();

        // act
        let mut tk = Telekinesis::connect_with(|| async move { connector }, None).unwrap();
        tk.await_connect(count);
        for device_name in tk.get_known_device_names() {
            tk.settings_set_enabled(&device_name, true);
        }
        tk.vibrate(Speed::new(100), TkDuration::from_millis(1), vec![]);

        // assert
        call_registry.assert_vibrated(1); // scalar
        call_registry.assert_not_vibrated(4); // linear
        call_registry.assert_not_vibrated(7); // rotator
    }

    #[test]
    fn vibrate_all_only_vibrates_vibrators() {
        // arrange
        let (mut tk, call_registry) = wait_for_connection(vec![
            scalar(1, "vib1", ActuatorType::Vibrate),
            scalar(2, "vib2", ActuatorType::Inflate),
        ]);

        tk.vibrate(Speed::new(100), TkDuration::from_millis(1), vec![]);

        // assert
        call_registry.assert_vibrated(1);
        call_registry.assert_not_vibrated(2);
    }

    #[test]
    fn vibrate_non_existing_device() {
        // arrange
        let (mut tk, call_registry) =
            wait_for_connection(vec![scalar(1, "vib1", ActuatorType::Vibrate)]);

        // act
        tk.vibrate(
            Speed::max(),
            TkDuration::from_millis(1),
            vec![String::from("does not exist")],
        );
        thread::sleep(Duration::from_millis(50));

        // assert
        call_registry.assert_not_vibrated(1);
    }

    #[test]
    fn vibrate_two_devices_simultaneously_both_are_started_and_stopped() {
        let (mut tk, call_registry) = wait_for_connection(vec![
            scalar(1, "vib1", ActuatorType::Vibrate),
            scalar(2, "vib2", ActuatorType::Vibrate),
        ]);
        tk.settings_set_events("vib1", vec![String::from("device 1")]);
        tk.settings_set_events("vib2", vec![String::from("device 2")]);

        // act
        tk.vibrate(
            Speed::new(99),
            TkDuration::from_millis(3000),
            vec![String::from("device 1")],
        );
        tk.vibrate(
            Speed::new(88),
            TkDuration::from_millis(3000),
            vec![String::from("device 2")],
        );
        thread::sleep(Duration::from_secs(5));

        // assert
        call_registry.assert_vibrated(1);
        call_registry.assert_vibrated(2);
    }

    #[test]
    fn linear_correct_priority_2() {
        // call1  |111111111111111111111-->|
        // call2         |2222->|
        // result |111111122222211111111-->|

        // arrange
        let start = Instant::now();
        let (mut tk, call_registry) =
            wait_for_connection(vec![scalar(1, "vib1", ActuatorType::Vibrate)]);

        // act
        tk.vibrate(Speed::new(50), TkDuration::from_secs(1), vec![]);
        thread::sleep(Duration::from_millis(500));
        tk.vibrate(Speed::new(100), TkDuration::from_millis(10), vec![]);
        thread::sleep(Duration::from_secs(1));

        // assert
        print_device_calls(&call_registry, 1, start);

        assert!(call_registry.get_device(1)[0].vibration_started_strength(0.5));
        assert!(call_registry.get_device(1)[1].vibration_started_strength(1.0));
        assert!(call_registry.get_device(1)[2].vibration_started_strength(0.5));
        assert!(call_registry.get_device(1)[3].vibration_stopped());
        assert_eq!(call_registry.get_device(1).len(), 4);
    }

    #[test]
    fn linear_correct_priority_3() {
        // call1  |111111111111111111111111111-->|
        // call2       |22222222222222->|
        // call3            |333->|
        // result |111122222333332222222111111-->|

        // arrange
        let start = Instant::now();
        let (mut tk, call_registry) =
            wait_for_connection(vec![scalar(1, "vib1", ActuatorType::Vibrate)]);

        // act
        tk.vibrate(Speed::new(20), TkDuration::from_secs(3), vec![]);
        thread::sleep(Duration::from_millis(250));
        tk.vibrate(Speed::new(40), TkDuration::from_secs(2), vec![]);
        thread::sleep(Duration::from_millis(250));
        tk.vibrate(Speed::new(80), TkDuration::from_secs(1), vec![]);

        thread::sleep(Duration::from_secs(3));

        // assert
        print_device_calls(&call_registry, 1, start);

        assert!(call_registry.get_device(1)[0].vibration_started_strength(0.2));
        assert!(call_registry.get_device(1)[1].vibration_started_strength(0.4));
        assert!(call_registry.get_device(1)[2].vibration_started_strength(0.8));
        assert!(call_registry.get_device(1)[3].vibration_started_strength(0.4));
        assert!(call_registry.get_device(1)[4].vibration_started_strength(0.2));
        assert!(call_registry.get_device(1)[5].vibration_stopped());
        assert_eq!(call_registry.get_device(1).len(), 6);
    }

    #[test]
    fn linear_correct_priority_4() {
        // call1  |111111111111111111111111111-->|
        // call2       |22222222222->|
        // call3            |333333333-->|
        // result |111122222222222233333331111-->|

        // arrange
        let start = Instant::now();
        let (mut tk, call_registry) =
            wait_for_connection(vec![scalar(1, "vib1", ActuatorType::Vibrate)]);

        // act
        tk.vibrate(Speed::new(20), TkDuration::from_secs(3), vec![]);
        thread::sleep(Duration::from_millis(250));
        tk.vibrate(Speed::new(40), TkDuration::from_secs(1), vec![]);
        thread::sleep(Duration::from_millis(250));
        tk.vibrate(Speed::new(80), TkDuration::from_secs(2), vec![]);
        thread::sleep(Duration::from_secs(3));

        // assert
        print_device_calls(&call_registry, 1, start);

        assert!(call_registry.get_device(1)[0].vibration_started_strength(0.2));
        assert!(call_registry.get_device(1)[1].vibration_started_strength(0.4));
        assert!(call_registry.get_device(1)[2].vibration_started_strength(0.8));
        assert!(call_registry.get_device(1)[3].vibration_started_strength(0.8));
        assert!(call_registry.get_device(1)[4].vibration_started_strength(0.2));
        assert!(call_registry.get_device(1)[5].vibration_stopped());
        assert_eq!(call_registry.get_device(1).len(), 6);
    }

    #[test]
    fn linear_overrides_pattern() {
        // lin1   |11111111111111111-->|
        // pat1       |23452345234523452345234-->|
        // result |1111111111111111111123452345234-->|

        // arrange
        let start = Instant::now();
        let (mut tk, call_registry) =
            wait_for_connection(vec![scalar(1, "vib1", ActuatorType::Vibrate)]);

        // act
        let _lin1 = tk.vibrate(Speed::new(99), TkDuration::Infinite, vec![]);
        thread::sleep(Duration::from_millis(10));
        let _pat1 = tk.vibrate_pattern(
            TkPattern::Funscript(TkDuration::Infinite, String::from("31_Sawtooth-Fast")),
            vec![],
        );

        // assert
        print_device_calls(&call_registry, 1, start);

        thread::sleep(Duration::from_secs(2));
        tk.stop(_lin1);
        thread::sleep(Duration::from_secs(2));
        assert!(call_registry.get_device(1).len() > 3);
    }

    #[test]
    fn settings_only_vibrate_enabled_devices() {
        // arrange
        let (mut tk, call_registry) = wait_for_connection(vec![
            scalar(1, "vib1", ActuatorType::Vibrate),
            scalar(2, "vib2", ActuatorType::Vibrate),
            scalar(3, "vib3", ActuatorType::Vibrate),
        ]);
        tk.settings_set_enabled("vib2", false);

        // act
        tk.vibrate(Speed::max(), TkDuration::from_millis(1), vec![]);
        thread::sleep(Duration::from_secs(1));

        // assert
        call_registry.assert_vibrated(1);
        call_registry.assert_vibrated(3);
        call_registry.assert_not_vibrated(2);
    }

    #[test]
    fn events_get() {
        let empty: Vec<String> = vec![];
        let one_event: Vec<String> = vec![String::from("evt2")];
        let two_events: Vec<String> = vec![String::from("evt2"), String::from("evt3")];

        let (mut tk, _) = wait_for_connection(vec![
            scalar(1, "vib1", ActuatorType::Vibrate),
            scalar(2, "vib2", ActuatorType::Vibrate),
            scalar(3, "vib3", ActuatorType::Vibrate),
        ]);

        tk.settings_set_events("vib2", one_event.clone());
        tk.settings_set_events("vib3", two_events.clone());

        assert_eq!(tk.settings_get_events("vib1"), empty);
        assert_eq!(tk.settings_get_events("vib2"), one_event);
        assert_eq!(tk.settings_get_events("vib3"), two_events);
    }

    #[test]
    fn event_only_vibrate_selected_devices() {
        let (mut tk, call_registry) = wait_for_connection(vec![
            scalar(1, "vib1", ActuatorType::Vibrate),
            scalar(2, "vib2", ActuatorType::Vibrate),
        ]);
        tk.settings_set_events("vib1", vec![String::from("selected_event")]);
        tk.settings_set_events("vib2", vec![String::from("bogus")]);

        tk.vibrate(
            Speed::max(),
            TkDuration::from_millis(1),
            vec![String::from("selected_event")],
        );
        thread::sleep(Duration::from_secs(1));

        call_registry.assert_vibrated(1);
        call_registry.assert_not_vibrated(2);
    }

    #[test]
    fn event_is_trimmed_and_ignores_casing() {
        let (mut tk, call_registry) =
            wait_for_connection(vec![scalar(1, "vib1", ActuatorType::Vibrate)]);
        tk.settings_set_enabled("vib1", true);
        tk.settings_set_events("vib1", vec![String::from("some event")]);
        tk.vibrate(
            Speed::max(),
            TkDuration::from_millis(1),
            vec![String::from(" SoMe EvEnT    ")],
        );

        call_registry.assert_vibrated(1);
    }

    #[test]
    fn settings_are_trimmed_and_lowercased() {
        let (mut tk, call_registry) =
            wait_for_connection(vec![scalar(1, "vib1", ActuatorType::Vibrate)]);
        tk.settings_set_enabled("vib1", true);
        tk.settings_set_events("vib1", vec![String::from(" SoMe EvEnT    ")]);
        tk.vibrate(
            Speed::max(),
            TkDuration::from_millis(1),
            vec![String::from("some event")],
        );

        call_registry.assert_vibrated(1);
    }

    #[test]
    fn get_device_capabilities() {
        // arrange
        let (tk, _) = wait_for_connection(vec![
            scalar(1, "vib1", ActuatorType::Vibrate),
            scalar(2, "vib2", ActuatorType::Constrict),
            linear(3, "lin2"),
        ]);

        // assert
        assert!(
            tk.get_device_capabilities("not exist").is_empty(),
            "Non existing device returns empty list"
        );
        assert!(
            tk.get_device_capabilities("vib2").is_empty(),
            "Unsupported capability is not returned"
        );
        assert!(
            tk.get_device_capabilities("lin2").is_empty(),
            "Unsupported capability is not returned"
        );
        assert_eq!(
            tk.get_device_capabilities("vib1").first().unwrap(),
            &String::from("Vibrate"),
            "vibrator returns vibrate"
        );
    }

    #[test]
    fn get_device_connected() {
        let (tk, _) = wait_for_connection(vec![scalar(1, "existing", ActuatorType::Vibrate)]);
        assert_eq!(
            tk.get_device_connection_status("existing"),
            TkConnectionStatus::Connected,
            "Existing device returns connected"
        );
        assert_eq!(
            tk.get_device_connection_status("not existing"),
            TkConnectionStatus::NotConnected,
            "Non-existing device returns not connected"
        );
    }

    #[test]
    fn vibrate_infinitely_and_then_stop() {
        // arrange
        let (mut tk, call_registry) =
            wait_for_connection(vec![scalar(1, "vib1", ActuatorType::Vibrate)]);

        // act
        let handle = tk.vibrate(Speed::max(), TkDuration::Infinite, vec![]);
        thread::sleep(Duration::from_secs(1));
        call_registry.assert_started(1);

        tk.stop(handle);
        call_registry.assert_vibrated(1);
    }

    #[test]
    fn vibrate_linear_then_cancel() {
        // arrange
        let (mut tk, call_registry) =
            wait_for_connection(vec![scalar(1, "vib1", ActuatorType::Vibrate)]);

        // act
        tk.vibrate(
            Speed::max(),
            TkDuration::Timed(Duration::from_secs(1)),
            vec![],
        );
        thread::sleep(Duration::from_secs(1));
        call_registry.assert_started(1);

        tk.stop_all();
        call_registry.assert_vibrated(1);
    }

    fn wait_for_connection(devices: Vec<DeviceAdded>) -> (Telekinesis, FakeConnectorCallRegistry) {
        let devices_names: Vec<String> = devices.iter().map(|d| d.device_name().clone()).collect();
        let (connector, call_registry) = FakeDeviceConnector::new(devices);
        let count = connector.devices.len();

        // act
        let mut settings = TkSettings::default();
        settings.pattern_path =
            String::from("../contrib/Distribution/SKSE/Plugins/Telekinesis/Patterns");
        let mut tk =
            Telekinesis::connect_with(|| async move { connector }, Some(settings)).unwrap();
        tk.await_connect(count);

        for name in devices_names {
            tk.settings_set_enabled(&name, true);
        }

        (tk, call_registry)
    }

    #[test]
    #[ignore = "Requires one (1) vibrator to be connected via BTLE (vibrates it)"]
    fn vibrate_pattern_then_cancel() {
        let mut settings = TkSettings::default();
        settings.pattern_path =
            String::from("../contrib/Distribution/SKSE/Plugins/Telekinesis/Patterns");

        let mut tk =
            Telekinesis::connect_with(|| async move { in_process_connector() }, Some(settings))
                .unwrap();
        tk.scan_for_devices();
        tk.await_connect(1);
        thread::sleep(Duration::from_secs(2));
        tk.settings
            .set_enabled(tk.get_known_device_names().first().unwrap(), true);

        enable_log();
        let handle = tk.vibrate_pattern(
            TkPattern::Funscript(TkDuration::from_secs(10), String::from("02_Cruel-Tease")),
            vec![],
        );
        thread::sleep(Duration::from_secs(2)); // dont disconnect
        tk.stop(handle);
        thread::sleep(Duration::from_secs(10));
    }

    #[test]
    #[ignore = "Requires one (1) vibrator to be connected via BTLE (vibrates it)"]
    fn test_funscript_vibrate_10s() {
        // TODO: Does not assert if the vibration actually happened
        enable_log();

        let mut tk =
            Telekinesis::connect_with(|| async move { in_process_connector() }, None).unwrap();
        tk.scan_for_devices();
        tk.await_connect(1);
        thread::sleep(Duration::from_secs(2));
        let _ = tk.process_next_events();
        assert!(matches!(
            tk.connection_status.lock().unwrap().connection_status,
            TkConnectionStatus::Connected
        ));

        tk.settings
            .set_enabled(tk.get_known_device_names().first().unwrap(), true);

        tk.vibrate_pattern(
            TkPattern::Funscript(TkDuration::from_secs(10), String::from("01_Tease")),
            vec![],
        );
        thread::sleep(Duration::from_secs(15)); // dont disconnect
        tk.stop_all();
    }

    #[test]
    #[ignore = "Requires intiface to be connected, with a connected device (vibrates it)"]
    fn intiface_test_vibration() {
        enable_log();

        let mut settings = TkSettings::default();
        settings.connection = TkConnectionType::WebSocket(String::from("127.0.0.1:12345"));

        let mut tk = Telekinesis::connect(settings).unwrap();
        tk.scan_for_devices();

        thread::sleep(Duration::from_secs(5));
        let _ = tk.process_next_events();
        assert!(matches!(
            tk.connection_status.lock().unwrap().connection_status,
            TkConnectionStatus::Connected
        ));

        for device in tk.get_devices() {
            tk.settings.set_enabled(device.name(), true);
        }
        tk.vibrate(Speed::max(), TkDuration::Infinite, vec![]);
        thread::sleep(Duration::from_secs(5));
    }

    #[test]
    fn intiface_not_available_connection_status_error() {
        let mut settings = TkSettings::default();
        settings.connection = TkConnectionType::WebSocket(String::from("bogushost:6572"));

        let mut tk = Telekinesis::connect(settings).unwrap();
        tk.scan_for_devices();
        thread::sleep(Duration::from_secs(5));
        let _ = tk.process_next_events();

        match &tk.connection_status.lock().unwrap().connection_status {
            TkConnectionStatus::Failed(err) => {
                assert!(err.len() > 0);
            }
            _ => todo!(),
        };
    }

    fn print_device_calls(
        call_registry: &FakeConnectorCallRegistry,
        index: u32,
        test_start: Instant,
    ) {
        for i in 0..call_registry.get_device(index).len() {
            let fake_call = call_registry.get_device(1)[i].clone();
            let s = fake_call.get_scalar_strength();
            let t = (test_start.elapsed() - fake_call.time.elapsed()).as_millis();
            let perc = (s * 100.0).round();
            println!(
                "{:02} @{:04} ms {percent:>3}% {empty:=>width$}",
                i,
                t,
                percent = perc as i32,
                empty = "",
                width = (perc / 5.0).floor() as usize
            );
        }
    }
}
