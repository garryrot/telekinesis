use std::{
    fmt::{self, Display},
    fs::{self},
    path::PathBuf,
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, event, info, instrument, Level};

use buttplug::core::message::ActuatorType;

use bp_scheduler::{actuator::Actuator, settings::{ActuatorSettings, LinearRange, LinearSpeedScaling, ScalarRange}};

use crate::input::sanitize_name_list;

pub static DEFAULT_PATTERN_PATH: &str = "Data\\SKSE\\Plugins\\Telekinesis\\Patterns";
pub static SETTINGS_PATH: &str = "Data\\SKSE\\Plugins";
pub static SETTINGS_FILE: &str = "Telekinesis.v2.json";

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum TkConnectionType {
    InProcess,
    WebSocket(String),
    Test,
}

impl Display for TkConnectionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TkConnectionType::InProcess => write!(f, "In-Process"),
            TkConnectionType::WebSocket(host) => write!(f, "WebSocket {}", host),
            TkConnectionType::Test => write!(f, "Test"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TkLogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
}

impl From<TkLogLevel> for Level {
    fn from(val: TkLogLevel) -> Self {
        match val {
            crate::settings::TkLogLevel::Trace => Level::TRACE,
            crate::settings::TkLogLevel::Debug => Level::DEBUG,
            crate::settings::TkLogLevel::Info => Level::INFO,
            crate::settings::TkLogLevel::Warn => Level::WARN,
            crate::settings::TkLogLevel::Error => Level::ERROR,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TkSettings {
    pub version: u32,
    pub log_level: TkLogLevel,
    pub connection: TkConnectionType,
    pub devices: Vec<TkDeviceSettings>,
    #[serde(skip)]
    pub pattern_path: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TkDeviceSettings {
    pub actuator_id: String,
    pub enabled: bool,
    pub events: Vec<String>,
    #[serde(default = "ActuatorSettings::default")]
    pub actuator_settings: ActuatorSettings,
}

impl TkDeviceSettings {
    pub fn from_identifier(actuator_id: &str) -> TkDeviceSettings {
        TkDeviceSettings {
            actuator_id: actuator_id.into(),
            enabled: false,
            events: vec![],
            actuator_settings: ActuatorSettings::None,
        }
    }
    pub fn from_actuator(actuator: &Actuator) -> TkDeviceSettings {
        TkDeviceSettings {
            actuator_id: actuator.identifier().into(),
            enabled: false,
            events: vec![],
            actuator_settings: match actuator.actuator {
                ActuatorType::Vibrate
                | ActuatorType::Rotate
                | ActuatorType::Oscillate
                | ActuatorType::Constrict
                | ActuatorType::Inflate => ActuatorSettings::Scalar(ScalarRange::default()),
                ActuatorType::Position => ActuatorSettings::Linear(LinearRange::default()),
                _ => ActuatorSettings::None,
            },
        }
    }
}

impl TkSettings {
    pub fn default() -> Self {
        TkSettings {
            version: 2,
            log_level: TkLogLevel::Debug,
            connection: TkConnectionType::InProcess,
            devices: vec![],
            pattern_path: String::from(DEFAULT_PATTERN_PATH),
        }
    }
    pub fn try_read_or_default(settings_path: &str, settings_file: &str) -> Self {
        let path = [settings_path, settings_file].iter().collect::<PathBuf>();
        match fs::read_to_string(path) {
            Ok(settings_json) => match serde_json::from_str::<TkSettings>(&settings_json) {
                Ok(mut settings) => {
                    settings.pattern_path = String::from(DEFAULT_PATTERN_PATH);
                    settings
                }
                Err(err) => {
                    error!("Settings path '{}' could not be parsed. Error: {}. Using default configuration.", settings_path, err);
                    TkSettings::default()
                }
            },
            Err(err) => {
                info!("Settings path '{}' could not be opened. Error: {}. Using default configuration.", settings_path, err);
                TkSettings::default()
            }
        }
    }
    pub fn try_write(&self, settings_path: &str, settings_file: &str) -> bool {
        let json = serde_json::to_string_pretty(self).expect("Always serializable");
        let _ = fs::create_dir_all(settings_path);
        let filename = [settings_path, settings_file].iter().collect::<PathBuf>();

        event!(Level::INFO, filename=?filename, settings=?self, "Storing settings");
        if let Err(err) = fs::write(filename, json) {
            error!("Writing to file failed. Error: {}.", err);
            return false;
        }
        true
    }
    pub fn get_enabled_devices(&self) -> Vec<TkDeviceSettings> {
        self.devices.iter().filter(|d| d.enabled).cloned().collect()
    }

    pub fn get_or_create(&mut self, actuator_id: &str) -> TkDeviceSettings {
        let device = self.get_device(actuator_id);
        match device {
            Some(setting) => setting,
            None => {
                let device = TkDeviceSettings::from_identifier(actuator_id);
                self.update_device(device.clone());
                device
            },
        }
    }

    pub fn try_get_actuator_settings(&mut self, actuator_id: &str) -> ActuatorSettings {
        if let Some(setting) = self.get_device(actuator_id) {
            return setting.actuator_settings;
        }
        ActuatorSettings::None
    }

    pub fn get_or_create_linear(&mut self, actuator_id: &str) -> (TkDeviceSettings, LinearRange) {
        let mut device = self.get_or_create(actuator_id);
        if let ActuatorSettings::Scalar(ref scalar) = device.actuator_settings {
            error!("actuator {:?} is scalar but assumed linear... dropping all {:?}", actuator_id, scalar)
        }
        if let ActuatorSettings::Linear(ref linear) = device.actuator_settings {
            return (device.clone(), linear.clone());
        }
        let default = LinearRange { scaling: LinearSpeedScaling::Parabolic(2), ..Default::default() };
        device.actuator_settings = ActuatorSettings::Linear(default.clone());
        self.update_device(device.clone());
        (device, default)
    }

    pub fn get_or_create_scalar(&mut self, actuator_id: &str) -> (TkDeviceSettings, ScalarRange) {
        let mut device = self.get_or_create(actuator_id);
        if let ActuatorSettings::Linear(ref linear) = device.actuator_settings {
            error!("actuator {:?} is linear but assumed scalar... dropping all {:?}", actuator_id, linear)
        }
        if let ActuatorSettings::Scalar(ref scalar) = device.actuator_settings {
            return (device.clone(), scalar.clone());
        }
        let default = ScalarRange::default();
        device.actuator_settings = ActuatorSettings::Scalar(default.clone());
        self.update_device(device.clone());
        (device, default)
    }

    pub fn access_linear<F, R>(&mut self, actuator_id: &str, accessor: F) -> R
        where F: FnOnce(&mut LinearRange) -> R
    {
        let (mut settings, mut linear) = self.get_or_create_linear(actuator_id);
        let result = accessor(&mut linear);
        settings.actuator_settings = ActuatorSettings::Linear(linear);
        self.update_device(settings);
        result
    }

    pub fn access_scalar<F, R>(&mut self, actuator_id: &str, accessor: F) -> R
        where F: FnOnce(&mut ScalarRange) -> R
    {
        let (mut settings, mut scalar) = self.get_or_create_scalar(actuator_id);
        let result = accessor(&mut scalar);
        settings.actuator_settings = ActuatorSettings::Scalar(scalar);
        self.update_device(settings);

        result
    }
   
    pub fn update_device(&mut self, setting: TkDeviceSettings)
    {
        let insert_pos = self.devices.iter().find_position(|x| x.actuator_id == setting.actuator_id);
        if let Some((pos, _)) = insert_pos {
            self.devices[ pos ] = setting;
        } else {
            self.devices.push(setting);
        }
    }

    pub fn get_device(&self, actuator_id: &str) -> Option<TkDeviceSettings> {
         self.devices
                .iter()
                .find(|d| d.actuator_id == actuator_id)
                .cloned()
    }

    #[instrument]
    pub fn set_enabled(&mut self, actuator_id: &str, enabled: bool) {
        debug!("set_enabled");

        let mut device =  self.get_or_create(actuator_id);
        device.enabled = enabled;
        self.update_device(device)
    }

    #[instrument]
    pub fn set_events(&mut self, actuator_id: &str, events: &[String]) {
        debug!("set_events");

        let mut device = self.get_or_create(actuator_id);
        device.events = sanitize_name_list(events);
        self.update_device(device);
    }

    pub fn get_events(&mut self, actuator_id: &str) -> Vec<String> {
        self.get_or_create(actuator_id).events
    }

    pub fn get_enabled(&mut self, actuator_id: &str) -> bool {
        self.get_or_create(actuator_id).enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{tempdir, TempDir};
    use tokio_test::assert_ok;

    #[test]
    fn serialize_deserialize_works() {
        // Arrange
        let mut setting = TkSettings::default();

        // Act
        setting.devices.push(TkDeviceSettings::from_identifier("value"));

        let serialized = serde_json::to_string_pretty(&setting).unwrap();
        let deserialized: TkSettings = serde_json::from_str(&serialized).unwrap();
        println!("{}", serialized);
        assert_eq!(
            deserialized.devices[0].actuator_id,
            setting.devices[0].actuator_id
        );
    }

    #[test]
    fn file_existing_returns_parsed_content() {
        // Arrange
        let mut setting = TkSettings::default();
        setting.devices.push(TkDeviceSettings::from_identifier("a"));
        setting.devices.push(TkDeviceSettings::from_identifier("b"));
        setting.devices.push(TkDeviceSettings::from_identifier("c"));

        let file = "test_config.json";
        let (path, _tmp_dir) = create_temp_file(file, &serde_json::to_string(&setting).unwrap());

        // Act
        println!("{}", path);
        let settings = TkSettings::try_read_or_default(_tmp_dir.path().to_str().unwrap(), file);
        assert_eq!(settings.devices.len(), 3);
    }

    #[test]
    fn file_not_existing_returns_default() {
        let settings = TkSettings::try_read_or_default("Path that does not exist", "some.json");
        assert_eq!(settings.devices.len(), settings.devices.len());
    }

    #[test]
    fn file_unreadable_returns_default() {
        // File
        let (_, tmp_dir) = create_temp_file("bogus.json", "Some stuff that is not valid json");

        // Act
        let settings =
            TkSettings::try_read_or_default(tmp_dir.path().to_str().unwrap(), "bogus.json");

        // Assert
        assert_eq!(settings.devices.len(), settings.devices.len());
    }

    #[test]
    fn adds_every_device_only_once() {
        let mut settings = TkSettings::default();
        settings.get_or_create("a");
        settings.get_or_create("a");
        assert_eq!(settings.devices.len(), 1);
    }

    #[test]
    fn enable_and_disable_devices() {
        let mut settings = TkSettings::default();
        settings.get_or_create("a");
        settings.get_or_create("b");
        settings.set_enabled("a", true);
        let enabled_devices = settings.get_enabled_devices();
        assert_eq!(enabled_devices.len(), 1);
        assert_eq!(enabled_devices[0].actuator_id, "a");

        settings.set_enabled("a", false);
        assert_eq!(settings.get_enabled_devices().len(), 0);
    }

    #[test]
    fn enable_multiple_devices() {
        let mut settings = TkSettings::default();
        settings.get_or_create("a");
        settings.get_or_create("b");
        settings.set_enabled("a", true);
        settings.set_enabled("b", true);
        assert_eq!(settings.get_enabled_devices().len(), 2);
    }

    #[test]
    fn enable_unknown_device() {
        let mut settings = TkSettings::default();
        settings.set_enabled("foobar", true);
        assert_eq!(settings.get_enabled_devices()[0].actuator_id, "foobar");
    }

    #[test]
    fn is_enabled_false() {
        let mut settings = TkSettings::default();
        settings.get_or_create("a");
        assert!(!settings.get_enabled("a"));
    }

    #[test]
    fn is_enabled_true() {
        let mut settings = TkSettings::default();
        settings.get_or_create("a");
        settings.set_enabled("a", true);
        assert!(settings.get_enabled("a"));
    }

    #[test]
    fn write_to_temp_file() {
        let mut settings = TkSettings::default();
        settings.get_or_create("foobar");

        // act
        let target_file = "some_target_file.json";
        let (_, tmpdir) = create_temp_file(target_file, "");
        settings.try_write(tmpdir.path().to_str().unwrap(), target_file);

        // assert
        let settings2 =
            TkSettings::try_read_or_default(tmpdir.path().to_str().unwrap(), target_file);
        assert_eq!(settings2.devices[0].actuator_id, "foobar");
        assert_ok!(tmpdir.close());
    }

    #[test]
    fn set_valid_websocket_endpoint() {
        let mut settings = TkSettings::default();
        let endpoint = String::from("3.44.33.6:12345");
        settings.connection = TkConnectionType::WebSocket(endpoint);
        if let TkConnectionType::WebSocket(endpoint) = settings.connection {
            assert_eq!(endpoint, "3.44.33.6:12345")
        } else {
            panic!()
        }
    }

    #[test]
    fn set_valid_websocket_endpoint_hostname() {
        let mut settings = TkSettings::default();
        let endpoint = String::from("localhost:12345");
        settings.connection = TkConnectionType::WebSocket(endpoint);
        if let TkConnectionType::WebSocket(endpoint) = settings.connection {
            assert_eq!(endpoint, "localhost:12345")
        } else {
            panic!()
        }
    }

    fn create_temp_file(name: &str, content: &str) -> (String, TempDir) {
        let tmp_path = tempdir().unwrap();
        assert_ok!(fs::create_dir_all(tmp_path.path().to_str().unwrap()));

        let file_path = tmp_path.path().join(name);
        let path = file_path.to_str().unwrap();
        assert_ok!(fs::write(file_path.clone(), content));

        (path.into(), tmp_path)
    }
}
