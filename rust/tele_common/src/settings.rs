use std::{
    fs::{self},
    path::PathBuf, vec,
};
use serde::{Deserialize, Serialize};
use tracing::{error, event, info, Level};

use bp_scheduler::settings::devices::BpSettings;

use crate::connection::TkConnectionType;

pub static DEFAULT_PATTERN_PATH: &str = "Data\\SKSE\\Plugins\\Telekinesis\\Patterns";
pub static SETTINGS_PATH: &str = "Data\\SKSE\\Plugins";
pub static SETTINGS_FILE: &str = "Telekinesis.v2.json";

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
            TkLogLevel::Trace => Level::TRACE,
            TkLogLevel::Debug => Level::DEBUG,
            TkLogLevel::Info => Level::INFO,
            TkLogLevel::Warn => Level::WARN,
            TkLogLevel::Error => Level::ERROR,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TkSettings {
    pub version: u32,
    pub log_level: TkLogLevel,
    pub connection: TkConnectionType,
    pub device_settings: BpSettings,
    #[serde(skip)]
    pub pattern_path: String,
}

impl TkSettings {
    pub fn default() -> Self {
        TkSettings {
            version: 2,
            log_level: TkLogLevel::Debug,
            connection: TkConnectionType::InProcess,
            device_settings: BpSettings {
                devices: vec![]
            },
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use bp_scheduler::settings::devices::*;
    use tempfile::{tempdir, TempDir};
    use tokio_test::assert_ok;

    #[test]
    fn serialize_deserialize_works() {
        // Arrange
        let mut setting = TkSettings::default();

        // Act
        setting.device_settings.devices.push(BpDeviceSettings::from_identifier("value"));

        let serialized = serde_json::to_string_pretty(&setting).unwrap();
        let deserialized: TkSettings = serde_json::from_str(&serialized).unwrap();
        println!("{}", serialized);
        assert_eq!(
            deserialized.device_settings.devices[0].actuator_id,
            setting.device_settings.devices[0].actuator_id
        );
    }

    #[test]
    fn file_existing_returns_parsed_content() {
        // Arrange
        let mut setting = TkSettings::default();
        setting.device_settings.devices.push(BpDeviceSettings::from_identifier("a"));
        setting.device_settings.devices.push(BpDeviceSettings::from_identifier("b"));
        setting.device_settings.devices.push(BpDeviceSettings::from_identifier("c"));

        let file = "test_config.json";
        let (path, _tmp_dir) = create_temp_file(file, &serde_json::to_string(&setting).unwrap());

        // Act
        println!("{}", path);
        let settings = TkSettings::try_read_or_default(_tmp_dir.path().to_str().unwrap(), file);
        assert_eq!(settings.device_settings.devices.len(), 3);
    }

    #[test]
    fn file_not_existing_returns_default() {
        let settings = TkSettings::try_read_or_default("Path that does not exist", "some.json");
        assert_eq!(settings.device_settings.devices.len(), settings.device_settings.devices.len());
    }

    #[test]
    fn file_unreadable_returns_default() {
        // File
        let (_, tmp_dir) = create_temp_file("bogus.json", "Some stuff that is not valid json");

        // Act
        let settings =
            TkSettings::try_read_or_default(tmp_dir.path().to_str().unwrap(), "bogus.json");

        // Assert
        assert_eq!(settings.device_settings.devices.len(), settings.device_settings.devices.len());
    }

    #[test]
    fn adds_every_device_only_once() {
        let mut settings = TkSettings::default();
        settings.device_settings.get_or_create("a");
        settings.device_settings.get_or_create("a");
        assert_eq!(settings.device_settings.devices.len(), 1);
    }

    #[test]
    fn enable_and_disable_devices() {
        let mut settings = TkSettings::default();
        settings.device_settings.get_or_create("a");
        settings.device_settings.get_or_create("b");
        settings.device_settings.set_enabled("a", true);
        let enabled_devices = settings.device_settings.get_enabled_devices();
        assert_eq!(enabled_devices.len(), 1);
        assert_eq!(enabled_devices[0].actuator_id, "a");

        settings.device_settings.set_enabled("a", false);
        assert_eq!(settings.device_settings.get_enabled_devices().len(), 0);
    }

    #[test]
    fn enable_multiple_devices() {
        let mut settings = TkSettings::default();
        settings.device_settings.get_or_create("a");
        settings.device_settings.get_or_create("b");
        settings.device_settings.set_enabled("a", true);
        settings.device_settings.set_enabled("b", true);
        assert_eq!(settings.device_settings.get_enabled_devices().len(), 2);
    }

    #[test]
    fn enable_unknown_device() {
        let mut settings = TkSettings::default();
        settings.device_settings.set_enabled("foobar", true);
        assert_eq!(settings.device_settings.get_enabled_devices()[0].actuator_id, "foobar");
    }

    #[test]
    fn is_enabled_false() {
        let mut settings = TkSettings::default();
        settings.device_settings.get_or_create("a");
        assert!(!settings.device_settings.get_enabled("a"));
    }

    #[test]
    fn is_enabled_true() {
        let mut settings = TkSettings::default();
        settings.device_settings.get_or_create("a");
        settings.device_settings.set_enabled("a", true);
        assert!(settings.device_settings.get_enabled("a"));
    }

    #[test]
    fn write_to_temp_file() {
        let mut settings = TkSettings::default();
        settings.device_settings.get_or_create("foobar");

        // act
        let target_file = "some_target_file.json";
        let (_, tmpdir) = create_temp_file(target_file, "");
        settings.try_write(tmpdir.path().to_str().unwrap(), target_file);

        // assert
        let settings2 =
            TkSettings::try_read_or_default(tmpdir.path().to_str().unwrap(), target_file);
        assert_eq!(settings2.device_settings.devices[0].actuator_id, "foobar");
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
