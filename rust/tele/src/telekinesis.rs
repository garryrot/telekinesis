
use anyhow::Error;
use bp_scheduler::ButtplugScheduler;
use bp_scheduler::PlayerSettings;
use bp_scheduler::speed::Speed;
use buttplug::{
    client::ButtplugClient,
    core::{
        connector::{
            new_json_ws_client_connector, ButtplugConnector,
            ButtplugInProcessClientConnectorBuilder,
        },
        message::{
            ButtplugCurrentSpecClientMessage, ButtplugCurrentSpecServerMessage,
        },
    },
    server::{
        device::hardware::communication::btleplug::BtlePlugCommunicationManagerBuilder,
        ButtplugServerBuilder,
    },
};
use funscript::FScript;
use futures::Future;
use tracing::instrument;

use std::time::Duration;
use std::{
    fmt::{self},
    time::Instant,
};
use tokio::sync::mpsc::Sender;
use tokio::{runtime::Runtime, sync::mpsc::channel};
use tracing::{debug, error, info};

use crate::connection::Task;
use crate::status::Status;
use crate::{
    connection::{
        handle_connection, TkCommand, TkConnectionEvent,
    },
    input::TkParams,
    settings::{TkConnectionType, TkSettings}
};

pub static ERROR_HANDLE: i32 = -1;

pub struct Telekinesis {
    pub settings: TkSettings,
    pub connection_events: crossbeam_channel::Receiver<TkConnectionEvent>,
    pub status: Status,
    runtime: Runtime,
    command_sender: Sender<TkCommand>,
    scheduler: ButtplugScheduler,
    client_event_sender: crossbeam_channel::Sender<TkConnectionEvent>,
    status_event_sender: crossbeam_channel::Sender<TkConnectionEvent>,
}

impl Telekinesis {
    pub fn connect_with<T, Fn, Fut>(
        connector_factory: Fn,
        provided_settings: Option<TkSettings>,
        type_name: TkConnectionType,
    ) -> Result<Telekinesis, anyhow::Error>
    where
        Fn: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = T> + Send,
        T: ButtplugConnector<ButtplugCurrentSpecClientMessage, ButtplugCurrentSpecServerMessage>
            + 'static,
    {
        let settings = provided_settings.unwrap_or_else(TkSettings::default);
        let (event_sender_client, event_receiver) = crossbeam_channel::unbounded();
        let (event_sender_internal, event_receiver_internal) = crossbeam_channel::unbounded();
        let (command_sender, command_receiver) = channel(256);
        let (scheduler, mut worker) = ButtplugScheduler::create(PlayerSettings {
            scalar_resolution_ms: 100,
        });

        let telekinesis = Telekinesis {
            command_sender,
            connection_events: event_receiver,
            runtime: Runtime::new()?,
            settings: settings.clone(),
            scheduler,
            client_event_sender: event_sender_client.clone(),
            status_event_sender: event_sender_internal.clone(),
            status: Status::new(event_receiver_internal, &settings)
        };
        info!(?telekinesis, "connecting...");
        telekinesis.runtime.spawn(async move {
            debug!("starting connection handling thread");
            let client = with_connector(connector_factory().await).await;
            handle_connection(
                event_sender_client,
                event_sender_internal,
                command_receiver,
                client,
                type_name,
            )
            .await;
            debug!("connection handling stopped");
        });
        telekinesis.runtime.spawn(async move {
            debug!("starting worker thread");
            worker.run_worker_thread().await;
            debug!("worked thread stopped");
        });
        Ok(telekinesis)
    }
}

impl Telekinesis {
    pub fn connect(settings: TkSettings) -> Result<Telekinesis, Error> {
        let settings_clone = settings.clone();
        match settings.connection {
            TkConnectionType::WebSocket(endpoint) => {
                let uri = format!("ws://{}", endpoint);
                Telekinesis::connect_with(
                    || async move { new_json_ws_client_connector(&uri) },
                    Some(settings_clone),
                    TkConnectionType::WebSocket(endpoint),
                )
            }
            _ => Telekinesis::connect_with(
                || async move { in_process_connector() },
                Some(settings),
                TkConnectionType::InProcess,
            ),
        }
    }

    pub fn scan_for_devices(&self) -> bool {
        info!("start scan");
        if self.command_sender.try_send(TkCommand::Scan).is_err() {
            error!("Failed to start scan");
            return false;
        }
        true
    }

    pub fn stop_scan(&self) -> bool {
        info!("stop scan");
        if self.command_sender.try_send(TkCommand::StopScan).is_err() {
            error!("Failed to stop scan");
            return false;
        }
        true
    }

    #[instrument(skip(self, fscript))]
    pub fn vibrate(
        &mut self,
        task: Task,
        duration: Duration,
        tags: Vec<String>,
        fscript: Option<FScript>,
    ) -> i32 {
        info!("vibrate");
        self.scheduler.clean_finished_tasks();

        let task_clone = task.clone();
        let params = TkParams::from_input(tags.clone(), &task, &self.settings.devices);
        let actuators = self.status.actuators();
        let player = self.scheduler.create_player(params.filter_devices(&actuators));
        let handle = player.handle;
        let client_sender_clone = self.client_event_sender.clone();
        let status_sender_clone = self.status_event_sender.clone();
        self.runtime.spawn(async move {
            let now = Instant::now();
            client_sender_clone
                .send(TkConnectionEvent::ActionStarted(
                    task_clone.clone(),
                    player.actuators.clone(),
                    tags,
                    player.handle,
                ))
                .expect("never full");
            let result = match task {
                Task::Scalar(speed) => player.play_scalar(duration, speed).await,
                Task::Pattern(speed, _, _) => {
                    player
                        .play_scalar_pattern(duration, fscript.unwrap(), speed)
                        .await
                }
            };
            let event = match result {
                Ok(_) => TkConnectionEvent::ActionDone(task_clone, now.elapsed(), handle),
                Err(err) => {
                    TkConnectionEvent::ActionError(actuators[0].clone(), err.to_string())
                }
            };
            client_sender_clone.send(event.clone()).expect("never full");
            status_sender_clone.send(event.clone()).expect("never full");
        });
        handle
    }

    #[instrument(skip(self))]
    pub fn update(&mut self, handle: i32, speed: Speed) -> bool {
        info!("update");
        self.scheduler.clean_finished_tasks();
        self.scheduler.update_task(handle, speed)
    }

    #[instrument(skip(self))]
    pub fn stop(&mut self, handle: i32) -> bool {
        info!("stop");
        self.scheduler.stop_task(handle);
        true
    }

    #[instrument(skip(self))]
    pub fn stop_all(&mut self) -> bool {
        info!("stop all");
        self.scheduler.stop_all();
        if self.command_sender.try_send(TkCommand::StopAll).is_err() {
            error!("Failed to queue stop_all");
            return false;
        }
        true
    }

    #[instrument(skip(self))]
    pub fn disconnect(&mut self) {
        info!("disconnect");
        if self.command_sender.try_send(TkCommand::Disconect).is_err() {
            error!("Failed to send disconnect");
        }
    }

    #[instrument(skip(self))]
    pub fn settings_set_enabled(&mut self, actuator_id: &str, enabled: bool) {
        debug!("settings={:?}", self.settings);
        let mut settings = self.settings.clone();
        settings.set_enabled(actuator_id, enabled);
        self.settings = settings;
    }

    #[instrument(skip(self))]
    pub fn settings_set_events(&mut self, actuator_id: &str, events: &[String]) {
        debug!("settings={:?}", self.settings);
        let settings = self.settings.clone();
        self.settings = settings.set_events(actuator_id, events);
    }

    pub fn settings_get_events(&self, actuator_id: &str) -> Vec<String> {
        self.settings.get_events(actuator_id)
    }

    pub fn settings_get_enabled(&self, actuator_id: &str) -> bool {
        self.settings.is_enabled(actuator_id)
    }
}

pub fn in_process_connector() -> impl ButtplugConnector<ButtplugCurrentSpecClientMessage, ButtplugCurrentSpecServerMessage> {
    ButtplugInProcessClientConnectorBuilder::default()
        .server(
            ButtplugServerBuilder::default()
                .comm_manager(BtlePlugCommunicationManagerBuilder::default())
                .finish()
                .expect("Could not create in-process-server."),
        )
        .finish()
}

async fn with_connector<T>(connector: T) -> ButtplugClient
where
    T: ButtplugConnector<ButtplugCurrentSpecClientMessage, ButtplugCurrentSpecServerMessage>
        + 'static,
{
    let buttplug = ButtplugClient::new("Telekinesis");
    if let Err(err) = buttplug.connect(connector).await {
        error!("Could not connect client. Error: {}.", err);
    }
    buttplug
}

impl fmt::Debug for Telekinesis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Telekinesis")
            .field("settings", &self.settings)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::pattern::read_pattern;
    use crate::status::TkConnectionStatus;
    use crate::telekinesis::in_process_connector;
    use crate::*;
    use bp_fakes::{scalar, FakeConnectorCallRegistry, FakeDeviceConnector};
    use bp_scheduler::speed::Speed;
    use buttplug::core::message::{ActuatorType, DeviceAdded};
    use std::time::Instant;
    use std::{thread, time::Duration, vec};

    use super::Telekinesis;

    macro_rules! assert_timeout {
        ($cond:expr, $arg:tt) => {
            // starting time
            let start: Instant = Instant::now();
            while !$cond {
                thread::sleep(Duration::from_millis(10));
                if start.elapsed().as_secs() > 5 {
                    panic!($arg);
                }
            }
        };
    }

    impl Telekinesis {
        pub fn await_connect(&mut self, devices: usize) {
            assert_timeout!(
                self.status.actuators().len() == devices,
                "Awaiting connect"
            );
        }
    }

    /// Vibrate

    #[test]
    fn vibrate_infinitely_and_then_stop() {
        // arrange
        let (mut tk, call_registry) =
            wait_for_connection(vec![scalar(1, "vib1", ActuatorType::Vibrate)], None);

        // act
        let handle = tk.vibrate(Task::Scalar(Speed::max()), Duration::MAX, vec![], None);

        thread::sleep(Duration::from_secs(1));
        call_registry.get_device(1)[0].assert_strenth(1.0);

        tk.stop(handle);
        thread::sleep(Duration::from_secs(1));
        call_registry.get_device(1)[1].assert_strenth(0.0);
    }

    #[test]
    fn vibrate_linear_then_cancel() {
        // arrange
        let (mut tk, call_registry) =
            wait_for_connection(vec![scalar(1, "vib1", ActuatorType::Vibrate)], None);

        // act
        thread::sleep(Duration::from_secs(1));
        tk.vibrate(
            Task::Scalar(Speed::max()),
            Duration::from_secs(1),
            vec![],
            None,
        );
        thread::sleep(Duration::from_secs(2));
        call_registry.get_device(1)[0].assert_strenth(1.0);
        tk.stop_all();

        thread::sleep(Duration::from_secs(1));
        call_registry.get_device(1)[1].assert_strenth(0.0);
    }

    #[test]
    fn vibrate_all_demo_vibrators() {
        // arrange
        let (connector, call_registry) = FakeDeviceConnector::device_demo();
        let count = connector.devices.len();

        // act
        let mut tk =
            Telekinesis::connect_with(|| async move { connector }, None, TkConnectionType::Test)
                .unwrap();
        tk.await_connect(count);
        for actuator_id in tk.status.get_known_actuator_ids() {
            tk.settings_set_enabled(&actuator_id, true);
        }
        tk.vibrate(
            Task::Scalar(Speed::new(100)),
            Duration::from_millis(1),
            vec![],
            None,
        );

        // assert
        thread::sleep(Duration::from_millis(500));
        call_registry.get_device(1)[0].assert_strenth(1.0);
        call_registry.get_device(1)[1].assert_strenth(0.0);
        call_registry.assert_unused(4); // linear
        call_registry.assert_unused(7); // rotator
    }

    #[test]
    fn vibrate_non_existing_device() {
        // arrange
        let (mut tk, call_registry) =
            wait_for_connection(vec![scalar(1, "vib1", ActuatorType::Vibrate)], None);

        // act
        tk.vibrate(
            Task::Scalar(Speed::max()),
            Duration::from_millis(1),
            vec![String::from("does not exist")],
            None,
        );
        thread::sleep(Duration::from_millis(50));

        // assert
        call_registry.assert_unused(1);
    }

    #[test]
    fn settings_only_vibrate_enabled_devices() {
        // arrange
        let (mut tk, call_registry) = wait_for_connection(vec![
            scalar(1, "vib1", ActuatorType::Vibrate),
            scalar(2, "vib2", ActuatorType::Vibrate),
            scalar(3, "vib3", ActuatorType::Vibrate),
        ], None);
        tk.settings_set_enabled("vib2 (Vibrate)", false);

        // act
        tk.vibrate(
            Task::Scalar(Speed::max()),
            Duration::from_millis(1),
            vec![],
            None,
        );
        thread::sleep(Duration::from_secs(1));

        // assert
        call_registry.get_device(1)[0].assert_strenth(1.0);
        call_registry.get_device(1)[1].assert_strenth(0.0);
        call_registry.get_device(3)[0].assert_strenth(1.0);
        call_registry.get_device(3)[1].assert_strenth(0.0);
        call_registry.assert_unused(2);
    }

    /// Vibrate (E2E)

    #[test]
    #[ignore = "Requires one (1) vibrator to be connected via BTLE (vibrates it)"]
    fn vibrate_pattern_then_cancel() {
        let (mut tk, handle) = test_pattern("02_Cruel-Tease", Duration::from_secs(10));
        thread::sleep(Duration::from_secs(2)); // dont disconnect
        tk.stop(handle);
        thread::sleep(Duration::from_secs(10));
    }

    #[test]
    #[ignore = "Requires one (1) vibrator to be connected via BTLE (vibrates it)"]
    fn vibrate_pattern_loops() {
        let (mut tk, handle) = test_pattern("03_Wub-Wub-Wub", Duration::from_secs(20));
        thread::sleep(Duration::from_secs(20));
        tk.stop(handle);
        thread::sleep(Duration::from_secs(2));
    }

    fn test_pattern(pattern_name: &str, duration: Duration) -> (Telekinesis, i32) {
        let settings = TkSettings::default();
        let pattern_path =
            String::from("../contrib/Distribution/SKSE/Plugins/Telekinesis/Patterns");
        let mut tk = Telekinesis::connect_with(
            || async move { in_process_connector() },
            Some(settings),
            TkConnectionType::Test,
        )
        .unwrap();
        tk.scan_for_devices();
        tk.await_connect(1);
        thread::sleep(Duration::from_secs(2));
        let known_actuator_ids = tk.status.get_known_actuator_ids();
        tk.settings.set_enabled(known_actuator_ids.first().unwrap(), true);

        let fscript = read_pattern(&pattern_path, pattern_name, true).unwrap();
        let handle = tk.vibrate(
            Task::Pattern(Speed::max(), ActuatorType::Vibrate, pattern_name.into()),
            duration,
            vec![],
            Some(fscript),
        );
        (tk, handle)
    }

    /// Intiface (E2E)

    #[test]
    #[ignore = "Requires intiface to be connected, with a connected device (vibrates it)"]
    fn intiface_test_vibration() {
        let mut settings = TkSettings::default();
        settings.connection = TkConnectionType::WebSocket(String::from("127.0.0.1:12345"));

        let mut tk = Telekinesis::connect(settings).unwrap();
        tk.scan_for_devices();

        thread::sleep(Duration::from_secs(5));
        assert!(matches!(
            tk.status.connection_status(),
            TkConnectionStatus::Connected
        ));

        for actuator in tk.status.actuators() {
            tk.settings.set_enabled(actuator.device.name(), true);
        }
        tk.vibrate(Task::Scalar(Speed::max()), Duration::MAX, vec![], None);
        thread::sleep(Duration::from_secs(5));
    }

    #[test]
    fn intiface_not_available_connection_status_error() {
        let mut settings = TkSettings::default();
        settings.connection = TkConnectionType::WebSocket(String::from("bogushost:6572"));

        let mut tk = Telekinesis::connect(settings).unwrap();
        tk.scan_for_devices();
        thread::sleep(Duration::from_secs(5));
        match tk.status.connection_status() {
            TkConnectionStatus::Failed(err) => {
                assert!(!err.is_empty());
            }
            _ => todo!(),
        };
    }

    /// Settings

    #[test]
    fn settings_are_trimmed_and_lowercased() {
        let (mut tk, call_registry) =
            wait_for_connection(vec![scalar(1, "vib1", ActuatorType::Vibrate)], None);
        tk.settings_set_enabled("vib1 (Vibrate)", true);
        tk.settings_set_events("vib1 (Vibrate)", &[String::from(" SoMe EvEnT    ")]);
        tk.vibrate(
            Task::Scalar(Speed::max()),
            Duration::from_millis(1),
            vec![String::from("some event")],
            None,
        );

        thread::sleep(Duration::from_millis(500));
        call_registry.get_device(1)[0].assert_strenth(1.0);
        call_registry.get_device(1)[1].assert_strenth(0.0);
    }

    #[test]
    fn get_devices_contains_connected_devices() {
        // arrange
        let (mut tk, _) = wait_for_connection(vec![
            scalar(1, "vib1", ActuatorType::Vibrate),
            scalar(2, "vib2", ActuatorType::Inflate),
        ], None);

        // assert
        assert_timeout!(
            tk.status.actuators().len() == 2,
            "Enough devices connected"
        );
        assert!(
            tk.status.get_known_actuator_ids().contains(&String::from("vib1 (Vibrate)")),
            "Contains name vib1"
        );
        assert!(
            tk.status.get_known_actuator_ids().contains(&String::from("vib2 (Inflate)")),
            "Contains name vib2"
        );
    }

    #[test]
    fn get_devices_contains_devices_from_settings() {
        let mut settings = TkSettings::default();
        settings.set_enabled("foreign", true);

        let (mut tk, _) = wait_for_connection(vec![], Some(settings));
        assert!(
            tk.status.get_known_actuator_ids()
                .contains(&String::from("foreign")),
            "Contains additional device from settings"
        );
    }

    #[test]
    fn events_get() {
        let empty: Vec<String> = vec![];
        let one_event = &[String::from("evt2")];
        let two_events = &[String::from("evt2"), String::from("evt3")];

        let (mut tk, _) = wait_for_connection(vec![
            scalar(1, "vib1", ActuatorType::Vibrate),
            scalar(2, "vib2", ActuatorType::Vibrate),
            scalar(3, "vib3", ActuatorType::Vibrate),
        ], None);

        tk.settings_set_events("vib2", one_event);
        tk.settings_set_events("vib3", two_events);

        assert_eq!(tk.settings_get_events("vib1"), empty);
        assert_eq!(tk.settings_get_events("vib2"), one_event);
        assert_eq!(tk.settings_get_events("vib3"), two_events);
    }

    #[test]
    fn event_only_vibrate_selected_devices() {
        let (mut tk, call_registry) = wait_for_connection(vec![
            scalar(1, "vib1", ActuatorType::Vibrate),
            scalar(2, "vib2", ActuatorType::Vibrate),
        ], None);
        tk.settings_set_events("vib1 (Vibrate)", &[String::from("selected_event")]);
        tk.settings_set_events("vib2 (Vibrate)", &[String::from("bogus")]);

        tk.vibrate(
            Task::Scalar(Speed::max()),
            Duration::from_millis(1),
            vec![String::from("selected_event")],
            None,
        );
        thread::sleep(Duration::from_secs(1));

        call_registry.get_device(1)[0].assert_strenth(1.0);
        call_registry.get_device(1)[1].assert_strenth(0.0);
        call_registry.assert_unused(2);
    }

    #[test]
    fn event_is_trimmed_and_ignores_casing() {
        let (mut tk, call_registry) =
            wait_for_connection(vec![scalar(1, "vib1", ActuatorType::Vibrate)], None);
        tk.settings_set_enabled("vib1 (Vibrate)", true);
        tk.settings_set_events("vib1 (Vibrate)", &[String::from("some event")]);
        tk.vibrate(
            Task::Scalar(Speed::max()),
            Duration::from_millis(1),
            vec![String::from(" SoMe EvEnT    ")],
            None,
        );

        thread::sleep(Duration::from_millis(500));
        call_registry.get_device(1)[0].assert_strenth(1.0);
        call_registry.get_device(1)[1].assert_strenth(0.0);
    }
    
    /// Device Status
    #[test]
    fn get_device_connected() {
        let (mut tk, _) = wait_for_connection(vec![scalar(1, "existing", ActuatorType::Vibrate)], None);
        assert_eq!(
            tk.status.get_actuator_status("existing (Vibrate)"),
            TkConnectionStatus::Connected,
            "Existing device returns connected"
        );
        assert_eq!(
            tk.status.get_actuator_status("not existing (Vibrate)"),
            TkConnectionStatus::NotConnected,
            "Non-existing device returns not connected"
        );
    }

    /// Events

    #[test]
    fn process_next_events_after_action_returns_1() {
        let mut tk = Telekinesis::connect_with(
            || async move { in_process_connector() },
            None,
            TkConnectionType::Test,
        )
        .unwrap();
        tk.vibrate(
            Task::Scalar(Speed::new(22)),
            Duration::from_millis(1),
            vec![],
            None,
        );
        get_next_events_blocking(&tk.connection_events);
    }

    #[test]
    fn process_next_events_works() {
        let mut tk = Telekinesis::connect_with(
            || async move { in_process_connector() },
            None,
            TkConnectionType::Test,
        )
        .unwrap();
        tk.vibrate(
            Task::Scalar(Speed::new(10)),
            Duration::from_millis(100),
            vec![],
            None,
        );
        tk.vibrate(
            Task::Scalar(Speed::new(20)),
            Duration::from_millis(200),
            vec![],
            None,
        );
        get_next_events_blocking(&tk.connection_events);
        get_next_events_blocking(&tk.connection_events);
    }

    fn wait_for_connection(devices: Vec<DeviceAdded>, settings: Option<TkSettings>) -> (Telekinesis, FakeConnectorCallRegistry) {
        let (connector, call_registry) = FakeDeviceConnector::new(devices);
        let count = connector.devices.len();

        // act
        let mut settings = settings.unwrap_or(TkSettings::default());
        settings.pattern_path =
            String::from("../contrib/Distribution/SKSE/Plugins/Telekinesis/Patterns");
        let mut tk = Telekinesis::connect_with(
            || async move { connector },
            Some(settings),
            TkConnectionType::Test,
        )
        .unwrap();
        tk.await_connect(count);

        for actuator in tk.status.actuators() {   
            tk.settings_set_enabled(actuator.identifier(), true);
        }
        (tk, call_registry)
    }
}
