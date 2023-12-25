use anyhow::anyhow;
use anyhow::Error;
use bp_scheduler::ButtplugScheduler;
use bp_scheduler::PlayerSettings;
use bp_scheduler::actuator::get_actuators;
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
use funscript::FScript;
use futures::Future;
use itertools::Itertools;

use std::time::Duration;
use std::{
    fmt::{self},
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Instant,
};
use tokio::{runtime::Runtime, sync::mpsc::channel};
use tokio::sync::mpsc::Sender;
use tracing::{debug, error, info};

use crate::connection::ActuatorList;
use crate::connection::Task;
use crate::{
    connection::{
        handle_connection, TkCommand, TkConnectionEvent, TkConnectionStatus, TkDeviceStatus,
    },
    input::TkParams,
    settings::{TkConnectionType, TkSettings},
    TkStatus,
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

pub struct Telekinesis {
    pub connection_status: Arc<Mutex<TkStatus>>,
    pub settings: TkSettings,
    pub connection_events: crossbeam_channel::Receiver<TkConnectionEvent>,
    runtime: Runtime,
    command_sender: Sender<TkCommand>,
    scheduler: ButtplugScheduler,
    event_sender: crossbeam_channel::Sender<TkConnectionEvent>,
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
        let runtime = Runtime::new()?;
        let (event_sender, event_receiver) = crossbeam_channel::unbounded();
        let (command_sender, command_receiver) = channel(256); // we handle them immediately
        let event_sender_clone = event_sender.clone();
        let connection_status = Arc::new(Mutex::new(TkStatus::default())); // ugly
        let status_clone = connection_status.clone();
        runtime.spawn(async move {
            debug!("starting connection handling thread");
            let client = with_connector(connector_factory().await).await;
            handle_connection(
                event_sender,
                command_receiver,
                client,
                connection_status,
                type_name,
            )
            .await;
            debug!("connection handling stopped");
        });

        let (scheduler, mut worker) = ButtplugScheduler::create(PlayerSettings {
            player_scalar_resolution_ms: 100,
        });

        runtime.spawn(async move {
            debug!("starting worker thread");
            worker.run_worker_thread().await;
            debug!("worked thread stopped");
        });

        Ok(Telekinesis {
            command_sender,
            connection_events: event_receiver,
            runtime,
            settings: provided_settings.unwrap_or_else(TkSettings::default),
            connection_status: status_clone,
            scheduler,
            event_sender: event_sender_clone,
        })
    }
}

impl fmt::Debug for Telekinesis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Telekinesis").finish()
    }
}

impl Telekinesis {
    pub fn get_device(&self, device_name: &str) -> Option<Arc<ButtplugClientDevice>> {
        if let Some(status) = self.get_device_status(device_name) {
            return Some(status.device);
        }
        None
    }

    pub fn get_devices(&self) -> ActuatorList {
        if let Ok(connection_status) = self.connection_status.try_lock() {
            let devices: ActuatorList = get_actuators(
                connection_status
                    .device_status
                    .values()
                    .map(|value| value.device.clone())
                    .collect(),
            );
            return devices;
        } else {
            error!("Error accessing device map");
        }
        vec![]
    }

    pub fn get_device_status(&self, device_name: &str) -> Option<TkDeviceStatus> {
        if let Ok(status) = self.connection_status.try_lock() {
            let devices = status
                .device_status
                .values()
                .find(|d| d.device.name() == device_name)
                .cloned();
            return devices;
        } else {
            error!("Error accessing device map");
        }
        None
    }

    pub fn get_known_device_names(&self) -> Vec<String> {
        debug!("Getting devices names");
        self.get_devices()
            .iter()
            .map(|d| d.device.name().clone())
            .chain(self.settings.devices.iter().map(|d| d.name.clone()))
            .unique()
            .collect()
    }

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
        info!("start scan for devices");
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

    pub fn get_device_capabilities(&self, name: &str) -> Vec<String> {
        debug!("Getting '{}' capabilities", name); // TODO print debug on each connect
                                                   // maybe just return all actuator + types + linear + rotate
        if self
            .get_devices()
            .iter()
            .filter(|a| a.device.name() == name)
            .any(|a| {
                if let Some(scalar) = a.device.message_attributes().scalar_cmd() {
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

    pub fn get_device_connection_status(&self, device_name: &str) -> TkConnectionStatus {
        debug!("Getting '{}' connected", device_name); // TODO print info on each connect
        if let Some(status) = self.get_device_status(device_name) {
            return status.status;
        }
        TkConnectionStatus::NotConnected
    }

    pub fn vibrate(
        &mut self,
        task: Task,
        duration: Duration,
        tags: Vec<String>,
        fscript: Option<FScript>
    ) -> i32 {
        info!("vibrate {:?}", task);
        let task_clone = task.clone();
        let params = TkParams::from_input(tags.clone(), task.clone(), &self.settings.devices);

        let actuators = self.get_devices();
        let player = self
            .scheduler
            .create_player(params.filter_devices(actuators.clone()));
        let handle = player.handle;
        let sender_clone = self.event_sender.clone();
        let connection_status = self.connection_status.clone();

        self.runtime.spawn(async move {
            let now = Instant::now();
            sender_clone
                .send(TkConnectionEvent::ActionStarted(
                    task_clone.clone(),
                    actuators.clone(),
                    tags,
                    player.handle,
                ))
                .expect("queue full");

            let result = match task {
                Task::Scalar(speed) => player.play_scalar(duration, speed).await,
                Task::Pattern(_, _) =>
                    player.play_scalar_pattern(duration, fscript.unwrap()).await
            };
            info!("done");
            if let Ok(mut connection_status) = connection_status.lock() {
                for actuator in &actuators {
                    let status = match &result {
                        Ok(_) => TkConnectionStatus::Connected,
                        Err(err) => TkConnectionStatus::Failed(err.to_string()),
                    };
                    connection_status
                        .device_status
                        .insert(actuator.device.index(), TkDeviceStatus::new(&actuator.device, status));
                }
            }
            sender_clone
                .send(match result {
                    Ok(_) => TkConnectionEvent::ActionDone(task_clone, now.elapsed(), handle),
                    Err(err) => {
                        TkConnectionEvent::ActionError(actuators[0].clone(), err.to_string())
                    }
                })
                .expect("queue full");
        });
        handle
    }

    pub fn stop(&mut self, handle: i32) -> bool {
        info!("stop handle {}", handle);
        self.scheduler.stop_task(handle);
        true
    }

    pub fn stop_all(&mut self) -> bool {
        info!("stop all");
        self.scheduler.stop_all();
        if self.command_sender.try_send(TkCommand::StopAll).is_err() {
            error!("Failed to queue stop_all");
            return false;
        }
        true
    }

    pub fn disconnect(&mut self) {
        info!("disconnect");
        if self.command_sender.try_send(TkCommand::Disconect).is_err() {
            error!("Failed to send disconnect");
        }
    }

    pub fn settings_set_enabled(&mut self, device_name: &str, enabled: bool) {
        debug!("Setting '{}'.enabled={}", device_name, enabled);
        let mut settings = self.settings.clone();
        settings.set_enabled(device_name, enabled);
        self.settings = settings;
    }

    pub fn settings_set_events(&mut self, device_name: &str, events: &[String]) {
        debug!("Setting '{}'.events={:?}", device_name, events);
        let settings = self.settings.clone();
        self.settings = settings.set_events(device_name, events);
    }

    pub fn settings_get_events(&self, device_name: &str) -> Vec<String> {
        self.settings.get_events(device_name)
    }

    pub fn settings_get_enabled(&self, device_name: &str) -> bool {
        let enabled = self.settings.is_enabled(device_name);
        debug!("Getting setting '{}'.enabled={}", device_name, enabled);
        enabled
    }

    pub fn get_connection_status(&self) -> TkConnectionStatus {
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
    if let Err(err) = buttplug.connect(connector).await {
        error!("Could not connect client. Error: {}.", err);
    }
    buttplug
}

pub fn get_pattern_names(pattern_path: &str, vibration_patterns: bool) -> Vec<String> {
    match get_pattern_paths(pattern_path) {
        Ok(patterns) => patterns
            .iter()
            .filter(|p| p.is_vibration == vibration_patterns)
            .map(|p| p.name.clone())
            .collect::<Vec<String>>(),
        Err(err) => {
            error!("Failed reading patterns {}", err);
            vec![]
        }
    }
}

struct TkPatternFile {
    path: PathBuf,
    is_vibration: bool,
    name: String,
}

fn get_pattern_paths(pattern_path: &str) -> Result<Vec<TkPatternFile>, anyhow::Error> {
    let mut patterns = vec![];
    let pattern_dir = fs::read_dir(pattern_path)?;
    for entry in pattern_dir {
        let file = entry?;

        let path = file.path();
        let path_clone = path.clone();
        let file_name = path
            .file_name()
            .ok_or_else(|| anyhow!("No file name"))?
            .to_str()
            .ok_or_else(|| anyhow!("Invalid unicode"))?;
        if !file_name.to_lowercase().ends_with(".funscript") {
            continue;
        }

        let is_vibration = file_name.to_lowercase().ends_with(".vibrator.funscript");
        let removal = if is_vibration {
            file_name.len() - ".vibrator.funscript".len()
        } else {
            file_name.len() - ".funscript".len()
        };

        patterns.push(TkPatternFile {
            path: path_clone,
            is_vibration,
            name: String::from(&file_name[0..removal]),
        })
    }
    Ok(patterns)
}

pub fn read_pattern(
    pattern_path: &str,
    pattern_name: &str,
    vibration_pattern: bool,
) -> Option<FScript> {
    match read_pattern_name(pattern_path, pattern_name, vibration_pattern) {
        Ok(funscript) => Some(funscript),
        Err(err) => {
            error!(
                "Error loading funscript vibration pattern={} err={}",
                pattern_name, err
            );
            None
        }
    }
}

pub fn read_pattern_name(
    pattern_path: &str,
    pattern_name: &str,
    vibration_pattern: bool,
) -> Result<FScript, anyhow::Error> {
    let now = Instant::now();
    let patterns: Vec<TkPatternFile> = get_pattern_paths(pattern_path)?;
    let pattern = patterns
        .iter()
        .find(|d| {
            d.is_vibration == vibration_pattern
                && d.name.to_lowercase() == pattern_name.to_lowercase()
        })
        .ok_or_else(|| anyhow!("Pattern '{}' not found", pattern_name))?;

    let fs = funscript::load_funscript(pattern.path.to_str().unwrap())?;
    debug!("Read pattern {} in {:?}", pattern_name, now.elapsed());
    Ok(fs)
}

#[cfg(test)]
mod tests {
    use crate::connection::TkConnectionStatus;
    use bp_fakes::{scalar, FakeDeviceConnector, linear, FakeConnectorCallRegistry};
    use bp_scheduler::speed::Speed;
    use crate::telekinesis::in_process_connector;
    use crate::*;
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
            tk.get_known_device_names()
                .contains(&String::from("foreign")),
            "Contains additional device from settings"
        );
    }

    #[test]
    fn vibrate_all_demo_only_vibrates_vibrators() {
        // arrange
        let (connector, call_registry) = FakeDeviceConnector::device_demo();
        let count = connector.devices.len();

        // act
        let mut tk =
            Telekinesis::connect_with(|| async move { connector }, None, TkConnectionType::Test)
                .unwrap();
        tk.await_connect(count);
        for device_name in tk.get_known_device_names() {
            tk.settings_set_enabled(&device_name, true);
        }
        tk.vibrate(Task::Scalar(Speed::new(100)), Duration::from_millis(1), vec![], None);

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
            wait_for_connection(vec![scalar(1, "vib1", ActuatorType::Vibrate)]);

        // act
        tk.vibrate(
            Task::Scalar(Speed::max()),
            Duration::from_millis(1),
            vec![String::from("does not exist")],
            None
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
        ]);
        tk.settings_set_enabled("vib2", false);

        // act
        tk.vibrate(Task::Scalar(Speed::max()), Duration::from_millis(1), vec![], None);
        thread::sleep(Duration::from_secs(1));

        // assert
        call_registry.get_device(1)[0].assert_strenth(1.0);
        call_registry.get_device(1)[1].assert_strenth(0.0);
        call_registry.get_device(3)[0].assert_strenth(1.0);
        call_registry.get_device(3)[1].assert_strenth(0.0);
        call_registry.assert_unused(2);
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
        ]);

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
        ]);
        tk.settings_set_events("vib1", &[String::from("selected_event")]);
        tk.settings_set_events("vib2", &[String::from("bogus")]);

        tk.vibrate(
            Task::Scalar(Speed::max()),
            Duration::from_millis(1),
            vec![String::from("selected_event")],
            None
        );
        thread::sleep(Duration::from_secs(1));

        call_registry.get_device(1)[0].assert_strenth(1.0);
        call_registry.get_device(1)[1].assert_strenth(0.0);

        call_registry.assert_unused(2);
    }

    #[test]
    fn event_is_trimmed_and_ignores_casing() {
        let (mut tk, call_registry) =
            wait_for_connection(vec![scalar(1, "vib1", ActuatorType::Vibrate)]);
        tk.settings_set_enabled("vib1", true);
        tk.settings_set_events("vib1", &[String::from("some event")]);
        tk.vibrate(
            Task::Scalar(Speed::max()),
            Duration::from_millis(1),
            vec![String::from(" SoMe EvEnT    ")],
            None
        );

        thread::sleep(Duration::from_millis(500));
        call_registry.get_device(1)[0].assert_strenth(1.0);
        call_registry.get_device(1)[1].assert_strenth(0.0);
    }

    #[test]
    fn settings_are_trimmed_and_lowercased() {
        let (mut tk, call_registry) =
            wait_for_connection(vec![scalar(1, "vib1", ActuatorType::Vibrate)]);
        tk.settings_set_enabled("vib1", true);
        tk.settings_set_events("vib1", &[String::from(" SoMe EvEnT    ")]);
        tk.vibrate(
            Task::Scalar(Speed::max()),
            Duration::from_millis(1),
            vec![String::from("some event")],
            None
        );

        thread::sleep(Duration::from_millis(500));
        call_registry.get_device(1)[0].assert_strenth(1.0);
        call_registry.get_device(1)[1].assert_strenth(0.0);
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
        let handle = tk.vibrate( Task::Scalar(Speed::max()), Duration::MAX, vec![], None);

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
            wait_for_connection(vec![scalar(1, "vib1", ActuatorType::Vibrate)]);

        // act
        thread::sleep(Duration::from_secs(1));
        tk.vibrate(Task::Scalar(Speed::max()), Duration::from_secs(1), vec![], None);
        thread::sleep(Duration::from_secs(2));
        call_registry.get_device(1)[0].assert_strenth(1.0);
        tk.stop_all();

        thread::sleep(Duration::from_secs(1));
        call_registry.get_device(1)[1].assert_strenth(0.0);
    }

    // TODO: Scheduler test
    #[test]
    #[ignore = "Requires one (1) vibrator to be connected via BTLE (vibrates it)"]
    fn vibrate_pattern_then_cancel() {
        let (mut tk, handle) = test_pattern("02_Cruel-Tease", Duration::from_secs(10));
        thread::sleep(Duration::from_secs(2)); // dont disconnect
        tk.stop(handle);
        thread::sleep(Duration::from_secs(10));
    }

    // TODO: Scheduler test
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
        tk.settings
            .set_enabled(tk.get_known_device_names().first().unwrap(), true);

            let fscript = read_pattern(&pattern_path, pattern_name, true).unwrap();
            let handle = tk.vibrate(Task::Pattern(ActuatorType::Vibrate, pattern_name.into()), duration, vec![], Some(fscript));
        (tk, handle)
    }

    #[test]
    #[ignore = "Requires intiface to be connected, with a connected device (vibrates it)"]
    fn intiface_test_vibration() {
        let mut settings = TkSettings::default();
        settings.connection = TkConnectionType::WebSocket(String::from("127.0.0.1:12345"));

        let mut tk = Telekinesis::connect(settings).unwrap();
        tk.scan_for_devices();

        thread::sleep(Duration::from_secs(5));
        assert!(matches!(
            tk.connection_status.lock().unwrap().connection_status,
            TkConnectionStatus::Connected
        ));

        for actuator in tk.get_devices() {
            tk.settings.set_enabled(actuator.device.name(), true);
        }
        tk.vibrate(Task::Scalar(Speed::max()), Duration::MAX, vec![], None);
        thread::sleep(Duration::from_secs(5));
    }

    #[test]
    fn intiface_not_available_connection_status_error() {
        let mut settings = TkSettings::default();
        settings.connection = TkConnectionType::WebSocket(String::from("bogushost:6572"));

        let tk = Telekinesis::connect(settings).unwrap();
        tk.scan_for_devices();
        thread::sleep(Duration::from_secs(5));
        match &tk.connection_status.lock().unwrap().connection_status {
            TkConnectionStatus::Failed(err) => {
                assert!(!err.is_empty());
            }
            _ => todo!(),
        };
    }

    fn wait_for_connection(devices: Vec<DeviceAdded>) -> (Telekinesis, FakeConnectorCallRegistry) {
        let devices_names: Vec<String> = devices.iter().map(|d| d.device_name().clone()).collect();
        let (connector, call_registry) = FakeDeviceConnector::new(devices);
        let count = connector.devices.len();

        // act
        let mut settings = TkSettings::default();
        settings.pattern_path =
            String::from("../contrib/Distribution/SKSE/Plugins/Telekinesis/Patterns");
        let mut tk = Telekinesis::connect_with(
            || async move { connector },
            Some(settings),
            TkConnectionType::Test,
        )
        .unwrap();
        tk.await_connect(count);

        for name in devices_names {
            tk.settings_set_enabled(&name, true);
        }

        (tk, call_registry)
    }

    #[test]
    fn process_next_events_after_action_returns_1() {
        let mut tk = Telekinesis::connect_with(
            || async move { in_process_connector() },
            None,
            TkConnectionType::Test,
        )
        .unwrap();
        tk.vibrate(Task::Scalar(Speed::new(22)), Duration::from_millis(1), vec![], None);
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
        tk.vibrate(Task::Scalar(Speed::new(10)), Duration::from_millis(100), vec![], None);
        tk.vibrate(Task::Scalar(Speed::new(20)), Duration::from_millis(200), vec![], None);
        get_next_events_blocking(&tk.connection_events);
        get_next_events_blocking(&tk.connection_events);
    }
}