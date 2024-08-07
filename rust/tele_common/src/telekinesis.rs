use anyhow::Error;
use anyhow::anyhow;
use settings::linear::LinearRange;

use std::{
    fmt::{self},
    time::Instant,
};

use futures::Future;
use tracing::{debug, error, info};

use tokio::sync::mpsc::Sender;
use tokio::{runtime::Runtime, sync::mpsc::channel};

use buttplug::{
    client::ButtplugClient,
    core::{
        connector::{
            new_json_ws_client_connector, ButtplugConnector,
            ButtplugInProcessClientConnectorBuilder,
        },
        message::{ButtplugCurrentSpecClientMessage, ButtplugCurrentSpecServerMessage},
    },
    server::{
        device::hardware::communication::btleplug::BtlePlugCommunicationManagerBuilder,
        ButtplugServerBuilder,
    },
};

use bp_scheduler::speed::*;
use bp_scheduler::*;

use crate::input::DeviceCommand;
use crate::input::TkParams;
use crate::settings::TkSettings;
use crate::status::Status;
use crate::connection::*;

#[cfg(feature = "testing")]
use bp_fakes::FakeDeviceConnector;

pub static ERROR_HANDLE: i32 = -1;

pub struct Telekinesis {
    pub settings: TkSettings,
    pub connection_events: crossbeam_channel::Receiver<TkConnectionEvent>,
    pub status: Status,
    runtime: Runtime,
    command_sender: Sender<ConnectionCommand>,
    scheduler: ButtplugScheduler,
    client_event_sender: crossbeam_channel::Sender<TkConnectionEvent>,
    status_event_sender: crossbeam_channel::Sender<TkConnectionEvent>,
}

impl Telekinesis {
    pub fn connect_with<T, Fn, Fut>(
        connect_action: Fn,
        provided_settings: Option<TkSettings>,
        type_name: TkConnectionType,
    ) -> Result<Telekinesis, anyhow::Error>
    where
        Fn: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = T> + Send,
        T: ButtplugConnector<ButtplugCurrentSpecClientMessage, ButtplugCurrentSpecServerMessage>
            + 'static,
    {
        let settings = provided_settings.unwrap_or_else(TkSettings::new);
        let (event_sender_client, event_receiver) = crossbeam_channel::unbounded();
        let (event_sender_internal, event_receiver_internal) = crossbeam_channel::unbounded();
        let (command_sender, command_receiver) = channel(256);
        let (scheduler, mut worker) = ButtplugScheduler::create(PlayerSettings {
            scalar_resolution_ms: 100,
        });

        let telekinesis = Telekinesis {
            command_sender: command_sender.clone(),
            connection_events: event_receiver,
            runtime: Runtime::new()?,
            settings: settings.clone(),
            scheduler,
            client_event_sender: event_sender_client.clone(),
            status_event_sender: event_sender_internal.clone(),
            status: Status::new(event_receiver_internal, &settings),
        };
        info!(?telekinesis, "connecting...");    
        telekinesis.runtime.spawn(async move {
            let client = with_connector(connect_action().await).await;
            handle_connection(
                event_sender_client,
                event_sender_internal,
                command_sender,
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

#[cfg(feature = "testing")]
pub fn get_test_connection(settings: TkSettings) -> Result<Telekinesis, Error> {
    Telekinesis::connect_with(
        || async move { FakeDeviceConnector::device_demo().0 },
        Some(options),
        TkConnectionType::Test,
    )
}

#[cfg(not(feature = "testing"))]
pub fn get_test_connection(_: TkSettings) -> Result<Telekinesis, Error> {
    Err(anyhow!("Compiled without testing support"))
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
            TkConnectionType::InProcess => Telekinesis::connect_with(
                || async move { in_process_connector() },
                Some(settings),
                TkConnectionType::InProcess,
            ),
            TkConnectionType::Test => {
                get_test_connection(settings)
            },
        }
    }

    pub fn scan_for_devices(&self) -> bool {
        info!("start scan");
        if self.command_sender.try_send(ConnectionCommand::Scan).is_err() {
            error!("Failed to start scan");
            return false;
        }
        true
    }

    pub fn stop_scan(&self) -> bool {
        info!("stop scan");
        if self.command_sender.try_send(ConnectionCommand::StopScan).is_err() {
            error!("Failed to stop scan");
            return false;
        }
        true
    }

    pub fn update(&mut self, handle: i32, speed: Speed) -> bool {
        info!("update");
        self.scheduler.clean_finished_tasks();
        self.scheduler.update_task(handle, speed)
    }

    pub fn stop(&mut self, handle: i32) -> bool {
        info!("stop");
        self.scheduler.stop_task(handle);
        true
    }

    pub fn stop_all(&mut self) -> bool {
        info!("stop all");
        self.scheduler.stop_all();
        if self.command_sender.try_send(ConnectionCommand::StopAll).is_err() {
            error!("Failed to queue stop_all");
            return false;
        }
        true
    }

    pub fn disconnect(&mut self) {
        info!("disconnect");
        if self.command_sender.try_send(ConnectionCommand::Disconect).is_err() {
            error!("Failed to send disconnect");
        }
    }

    pub fn dispatch_cmd(&mut self, cmd: DeviceCommand) -> i32 {
        self.scheduler.clean_finished_tasks();
        let task_clone = cmd.task.clone();
        let actuators = self.status.connected_actuators();
        let devices = TkParams::filter_devices(
            &actuators,
            &cmd.body_parts,
            &cmd.actuator_types,
            &self.settings.device_settings.devices,
        );
        let settings = devices.iter().map(|x| self.settings.device_settings.get_or_create(x.identifier()).actuator_settings ).collect();
        let player = self.scheduler.create_player_with_settings(devices, settings);
        let handle = player.handle;

        info!(handle, "dispatching {:?}", cmd.task);
        let client_sender_clone = self.client_event_sender.clone();
        let status_sender_clone = self.status_event_sender.clone();
        self.runtime.spawn(async move {
            let now = Instant::now();
            client_sender_clone
                .send(TkConnectionEvent::ActionStarted(
                    task_clone.clone(),
                    player.actuators.clone(),
                    cmd.body_parts,
                    player.handle,
                ))
                .expect("never full");
            let result = match cmd.task {
                Task::Scalar(speed) => player.play_scalar(cmd.duration, speed).await,
                Task::Pattern(speed, _, _) => {
                    player
                        .play_scalar_pattern(cmd.duration, cmd.fscript.unwrap(), speed)
                        .await
                }
                Task::Linear(_, _) => player.play_linear(cmd.duration, cmd.fscript.unwrap()).await,
                Task::LinearStroke(speed, _) => player.play_linear_stroke(cmd.duration, speed, LinearRange::max()).await,
            };
            info!(handle, "done");
            let event = match result {
                Ok(()) => TkConnectionEvent::ActionDone(task_clone, now.elapsed(), handle),
                Err(err) => TkConnectionEvent::ActionError(err.actuator, err.bp_error.to_string()),
            };
            client_sender_clone.send(event.clone()).expect("never full");
            status_sender_clone.send(event.clone()).expect("never full");
        });
        handle
    }
}

pub fn in_process_connector() 
    -> impl ButtplugConnector<ButtplugCurrentSpecClientMessage, ButtplugCurrentSpecServerMessage> {
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
    use buttplug::core::message::{ActuatorType, DeviceAdded};
    use connection::{Task, TkConnectionType};
    use input::DeviceCommand;
    use settings::TkSettings;
    use std::time::Instant;
    use std::{thread, time::Duration, vec};
    use funscript::FScript;
    
    use crate::*;
    use bp_fakes::*;
    use bp_scheduler::speed::Speed;
    use crate::pattern::read_pattern;
    use crate::status::TkConnectionStatus;
    use crate::telekinesis::in_process_connector;
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
            assert_timeout!(self.status.actuators().len() >= devices, "Awaiting connect");
        }
    }

    /// Vibrate
    pub fn test_cmd(
        tk: &mut Telekinesis, 
        task: Task, 
        duration: Duration,
        body_parts: Vec<String>,
        fscript: Option<FScript>,
        actuator_types: &[ActuatorType]) -> i32 {
            tk.dispatch_cmd(DeviceCommand {
                task,
                duration,
                fscript,
                body_parts,
                actuator_types: actuator_types.to_vec(),
            })
    } 

    #[test]
    fn test_vibrate_and_stop() {
        // arrange
        let (mut tk, call_registry) =
            wait_for_connection(vec![scalar(1, "vib1", ActuatorType::Vibrate)], None);

        // act
        let handle = test_cmd(
            &mut tk,
            Task::Scalar(Speed::max()),
            Duration::MAX,
            vec![],
            None,
            &[ActuatorType::Vibrate],
        );
        thread::sleep(Duration::from_secs(1));
        call_registry.get_device(1)[0].assert_strenth(1.0);

        tk.stop(handle);
        thread::sleep(Duration::from_secs(1));
        call_registry.get_device(1)[1].assert_strenth(0.0);
    }

    #[test]
    fn test_vibrate_and_stop_all() {
        // arrange
        let (mut tk, call_registry) =
            wait_for_connection(vec![scalar(1, "vib1", ActuatorType::Vibrate)], None);

        // act
        thread::sleep(Duration::from_secs(1));
        test_cmd(
            &mut tk,
            Task::Scalar(Speed::max()),
            Duration::from_secs(1),
            vec![],
            None,
            &[ActuatorType::Vibrate],
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
            tk.settings.device_settings.set_enabled(&actuator_id, true);
        }
        test_cmd(
            &mut tk,
            Task::Scalar(Speed::new(100)),
            Duration::from_millis(1),
            vec![],
            None,
            &[ActuatorType::Vibrate],
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
        test_cmd(
            &mut tk,
            Task::Scalar(Speed::max()),
            Duration::from_millis(1),
            vec![String::from("does not exist")],
            None,
            &[ActuatorType::Vibrate],
        );
        thread::sleep(Duration::from_millis(50));

        // assert
        call_registry.assert_unused(1);
    }

    #[test]
    fn settings_only_vibrate_enabled_devices() {
        // arrange
        let (mut tk, call_registry) = wait_for_connection(
            vec![
                scalar(1, "vib1", ActuatorType::Vibrate),
                scalar(2, "vib2", ActuatorType::Vibrate),
                scalar(3, "vib3", ActuatorType::Vibrate),
            ],
            None,
        );
        tk.settings.device_settings.set_enabled("vib2 (Vibrate)", false);

        // act
        test_cmd(
            &mut tk,
            Task::Scalar(Speed::max()),
            Duration::from_millis(1),
            vec![],
            None,
            &[ActuatorType::Vibrate],
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
    fn vibrate_pattern() {
        let (mut tk, handle) = test_pattern("02_Cruel-Tease", Duration::from_secs(10), true);
        thread::sleep(Duration::from_secs(2)); // dont disconnect
        tk.stop(handle);
        thread::sleep(Duration::from_secs(10));
    }

    fn test_pattern(
        pattern_name: &str,
        duration: Duration,
        vibration_pattern: bool,
    ) -> (Telekinesis, i32) {
        let settings = TkSettings::new();
        let pattern_path =
            String::from("../deploy/Data/SKSE/Plugins/Telekinesis/Patterns");
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
        tk.settings.device_settings
            .set_enabled(known_actuator_ids.first().unwrap(), true);

        let fscript = read_pattern(&pattern_path, pattern_name, vibration_pattern).unwrap();
        let handle = test_cmd(
            &mut tk,
            Task::Pattern(Speed::max(), ActuatorType::Vibrate, pattern_name.into()),
            duration,
            vec![],
            Some(fscript),
            &[ActuatorType::Vibrate],
        );
        (tk, handle)
    }

    /// Intiface (E2E)

    #[test]
    #[ignore = "Requires intiface to be connected, with a connected device (vibrates it)"]
    fn intiface_test_vibration() {
        let mut settings = TkSettings::new();
        settings.connection = TkConnectionType::WebSocket(String::from("127.0.0.1:12345"));

        let mut tk = Telekinesis::connect(settings).unwrap();
        tk.scan_for_devices();

        thread::sleep(Duration::from_secs(5));
        assert!(matches!(
            tk.status.connection_status(),
            TkConnectionStatus::Connected
        ));

        for actuator in tk.status.actuators() {
            tk.settings.device_settings.set_enabled(actuator.device.name(), true);
        }
        test_cmd(
            &mut tk,
            Task::Scalar(Speed::max()),
            Duration::MAX,
            vec![],
            None,
            &[ActuatorType::Vibrate],
        );
        thread::sleep(Duration::from_secs(5));
    }

    #[test]
    fn intiface_not_available_connection_status_error() {
        let mut settings = TkSettings::new();
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
        tk.settings.device_settings.set_enabled("vib1 (Vibrate)", true);
        tk.settings.device_settings.set_events("vib1 (Vibrate)", &[String::from(" SoMe EvEnT    ")]);
        test_cmd(
            &mut tk,
            Task::Scalar(Speed::max()),
            Duration::from_millis(1),
            vec![String::from("some event")],
            None,
            &[ActuatorType::Vibrate],
        );

        thread::sleep(Duration::from_millis(500));
        call_registry.get_device(1)[0].assert_strenth(1.0);
        call_registry.get_device(1)[1].assert_strenth(0.0);
    }

    #[test]
    fn get_devices_contains_connected_devices() {
        // arrange
        let (mut tk, _) = wait_for_connection(
            vec![
                scalar(1, "vib1", ActuatorType::Vibrate),
                scalar(2, "vib2", ActuatorType::Inflate),
            ],
            None,
        );

        // assert
        assert_timeout!(tk.status.actuators().len() == 2, "Enough devices connected");
        assert!(
            tk.status
                .get_known_actuator_ids()
                .contains(&String::from("vib1 (Vibrate)")),
            "Contains name vib1"
        );
        assert!(
            tk.status
                .get_known_actuator_ids()
                .contains(&String::from("vib2 (Inflate)")),
            "Contains name vib2"
        );
    }

    #[test]
    fn get_devices_contains_devices_from_settings() {
        let mut settings = TkSettings::new();
        settings.device_settings.set_enabled("foreign", true);

        let (mut tk, _) = wait_for_connection(vec![], Some(settings));
        assert!(
            tk.status
                .get_known_actuator_ids()
                .contains(&String::from("foreign")),
            "Contains additional device from settings"
        );
    }

    #[test]
    fn events_get() {
        let empty: Vec<String> = vec![];
        let one_event = &[String::from("evt2")];
        let two_events = &[String::from("evt2"), String::from("evt3")];

        let (mut tk, _) = wait_for_connection(
            vec![
                scalar(1, "vib1", ActuatorType::Vibrate),
                scalar(2, "vib2", ActuatorType::Vibrate),
                scalar(3, "vib3", ActuatorType::Vibrate),
            ],
            None,
        );

        tk.settings.device_settings.set_events("vib2", one_event);
        tk.settings.device_settings.set_events("vib3", two_events);

        assert_eq!(tk.settings.device_settings.get_events("vib1"), empty);
        assert_eq!(tk.settings.device_settings.get_events("vib2"), one_event);
        assert_eq!(tk.settings.device_settings.get_events("vib3"), two_events);
    }

    #[test]
    fn event_only_vibrate_selected_devices() {
        let (mut tk, call_registry) = wait_for_connection(
            vec![
                scalar(1, "vib1", ActuatorType::Vibrate),
                scalar(2, "vib2", ActuatorType::Vibrate),
            ],
            None,
        );
        tk.settings.device_settings.set_events("vib1 (Vibrate)", &[String::from("selected_event")]);
        tk.settings.device_settings.set_events("vib2 (Vibrate)", &[String::from("bogus")]);

        test_cmd(
            &mut tk,
            Task::Scalar(Speed::max()),
            Duration::from_millis(1),
            vec![String::from("selected_event")],
            None,
            &[ActuatorType::Vibrate],
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
        tk.settings.device_settings.set_enabled("vib1 (Vibrate)", true);
        tk.settings.device_settings.set_events("vib1 (Vibrate)", &[String::from("some event")]);
        test_cmd(
            &mut tk,
            Task::Scalar(Speed::max()),
            Duration::from_millis(1),
            vec![String::from(" SoMe EvEnT    ")],
            None,
            &[ActuatorType::Vibrate],
        );

        thread::sleep(Duration::from_millis(500));
        call_registry.get_device(1)[0].assert_strenth(1.0);
        call_registry.get_device(1)[1].assert_strenth(0.0);
    }

    /// Device Status
    #[test]
    fn get_device_connected() {
        let (mut tk, _) =
            wait_for_connection(vec![scalar(1, "existing", ActuatorType::Vibrate)], None);
        assert_eq!(
            tk.status.get_actuator_connection_status("existing (Vibrate)"),
            TkConnectionStatus::Connected,
            "Existing device returns connected"
        );
        assert_eq!(
            tk.status.get_actuator_connection_status("not existing (Vibrate)"),
            TkConnectionStatus::NotConnected,
            "Non-existing device returns not connected"
        );
    }

    fn wait_for_connection(
        devices: Vec<DeviceAdded>,
        settings: Option<TkSettings>,
    ) -> (Telekinesis, FakeConnectorCallRegistry) {
        let (connector, call_registry) = FakeDeviceConnector::new(devices);
        let count = connector.devices.len();

        // act
        let mut settings = settings.unwrap_or(TkSettings::new());
        settings.pattern_path =
            String::from("../deploy/Data/SKSE/Plugins/Telekinesis/Patterns");
        let mut tk = Telekinesis::connect_with(
            || async move { connector },
            Some(settings),
            TkConnectionType::Test,
        )
        .unwrap();
        tk.await_connect(count);

        for actuator in tk.status.actuators() {
            tk.settings.device_settings.set_enabled(actuator.identifier(), true);
        }
        (tk, call_registry)
    }
}