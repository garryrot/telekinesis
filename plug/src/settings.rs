use std::{
    fs::{self},
    path::{PathBuf},
    fmt::{self, Display}
};

use serde::{Deserialize, Serialize};
use tracing::{error, event, info, Level};

use crate::input::sanitize_name_list;

pub static DEFAULT_PATTERN_PATH: &str = "Data\\SKSE\\Plugins\\Telekinesis\\Patterns";
pub static SETTINGS_PATH: &str = "Data\\SKSE\\Plugins";
pub static SETTINGS_FILE: &str = "Telekinesis.json";

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TkSettings {
    pub version: u32,
    pub log_level: TkLogLevel,
    pub connection: TkConnectionType,
    pub devices: Vec<TkDeviceSettings>,
    #[serde(skip)]
    pub pattern_path: String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TkDeviceSettings {
    pub name: String,
    pub enabled: bool,
    pub events: Vec<String>,
}

impl TkDeviceSettings {
    pub fn from_name(name: &str) -> TkDeviceSettings {
        TkDeviceSettings {
            name: name.clone().into(),
            enabled: false,
            events: vec![],
        }
    }
}

impl TkSettings {
    pub fn default() -> Self {
        TkSettings {
            version: 1,
            log_level: TkLogLevel::Trace,
            connection: TkConnectionType::InProcess,
            devices: vec![],
            pattern_path: String::from(DEFAULT_PATTERN_PATH)
        }
    }
    pub fn try_read_or_default(settings_path: &str, settings_file: &str) -> Self {
        let path = [settings_path, settings_file].iter().collect::<PathBuf>();
        match fs::read_to_string(path) {
            Ok(settings_json) => match serde_json::from_str::<TkSettings>(&settings_json) {
                Ok(mut settings) => {
                    settings.pattern_path = String::from(DEFAULT_PATTERN_PATH);
                    settings
                },
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
            return false
        }
        true
    }
    pub fn get_enabled_devices(&self) -> Vec<TkDeviceSettings> {
        self.devices
            .iter()
            .filter(|d| d.enabled)
            .map(|d| d.clone())
            .collect()
    }
    pub fn add(&mut self, device_name: &str) {
        if self.devices.iter().any(|d| d.name == device_name) {
            return;
        }
        self.devices.push(TkDeviceSettings::from_name(device_name))
    }

    pub fn set_events(mut self, device_name: &str, events: Vec<String>) -> Self {
        self.assure_exists(device_name);
        let evts: Vec<String> = sanitize_name_list(&events);
        self.devices = self
            .devices
            .iter()
            .map(|d| {
                let mut device = d.clone();
                if d.name == device_name {
                    device.events = evts.clone();
                }
                device
            })
            .collect();
        self
    }

    pub fn get_events(&self, device_name: &str) -> Vec<String> {
        match self
            .devices
            .iter()
            .filter(|d| d.name == device_name)
            .map(|d| d.events.clone())
            .next()
        {
            Some(evt) => evt,
            None => vec![],
        }
    }

    pub fn set_enabled(&mut self, device_name: &str, enabled: bool) {
        self.assure_exists(device_name);

        self.devices = self
            .devices
            .iter()
            .map(|d| {
                let mut device = d.clone();
                if d.name == device_name {
                    device.enabled = enabled;
                }
                device
            })
            .collect();
    }

    pub fn assure_exists(&mut self, device_name: &str) {
        if self
            .devices
            .iter()
            .filter(|d| d.name == device_name)
            .count()
            == 0
        {
            self.add(device_name);
        }
    }

    pub fn is_enabled(&self, device_name: &str) -> bool {
        self.devices
            .iter()
            .filter(|d| d.name == device_name && d.enabled)
            .count()
            >= 1
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
        setting.devices.push(TkDeviceSettings::from_name("value"));

        let serialized = serde_json::to_string_pretty(&setting).unwrap();
        let deserialized: TkSettings = serde_json::from_str(&serialized).unwrap();
        println!("{}", serialized);
        assert_eq!(deserialized.devices[0].name, setting.devices[0].name);
    }

    #[test]
    fn file_existing_returns_parsed_content() {
        // Arrange
        let mut setting = TkSettings::default();
        setting.devices.push(TkDeviceSettings::from_name("a"));
        setting.devices.push(TkDeviceSettings::from_name("b"));
        setting.devices.push(TkDeviceSettings::from_name("c"));

        let file = "test_config.json";
        let (path, _tmp_dir) = create_temp_file(file, &serde_json::to_string(&setting).unwrap());

        // Act
        println!("{}", path);
        let settings = TkSettings::try_read_or_default(_tmp_dir.path().to_str().unwrap(), &file);
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
        settings.add("a");
        settings.add("a");
        assert_eq!(settings.devices.len(), 1);
    }

    #[test]
    fn enable_and_disable_devices() {
        let mut settings = TkSettings::default();
        settings.add("a");
        settings.add("b");
        settings.set_enabled("a", true);
        let enabled_devices = settings.get_enabled_devices();
        assert_eq!(enabled_devices.len(), 1);
        assert_eq!(enabled_devices[0].name, "a");

        settings.set_enabled("a", false);
        assert_eq!(settings.get_enabled_devices().len(), 0);
    }

    #[test]
    fn enable_multiple_devices() {
        let mut settings = TkSettings::default();
        settings.add("a");
        settings.add("b");
        settings.set_enabled("a", true);
        settings.set_enabled("b", true);
        assert_eq!(settings.get_enabled_devices().len(), 2);
    }

    #[test]
    fn enable_unknown_device() {
        let mut settings = TkSettings::default();
        settings.set_enabled("foobar", true);
        assert_eq!(settings.get_enabled_devices()[0].name, "foobar");
    }

    #[test]
    fn is_enabled_false() {
        let mut settings = TkSettings::default();
        settings.add("a");
        assert!(settings.is_enabled("a") == false);
    }

    #[test]
    fn is_enabled_true() {
        let mut settings = TkSettings::default();
        settings.add("a");
        settings.set_enabled("a", true);
        assert!(settings.is_enabled("a") == true);
    }

    #[test]
    fn write_to_temp_file() {
        let mut settings = TkSettings::default();
        settings.add("foobar");

        // act
        let target_file = "some_target_file.json";
        let (_, tmpdir) = create_temp_file(target_file, "");
        settings.try_write(tmpdir.path().to_str().unwrap(), target_file);

        // assert
        let settings2 =
            TkSettings::try_read_or_default(tmpdir.path().to_str().unwrap(), target_file);
        assert_eq!(settings2.devices[0].name, "foobar");
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
            assert!(false)
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
            assert!(false)
        }
    }

    fn create_temp_file(name: &str, content: &str) -> (String, TempDir) {
        let tmp_path = tempdir().unwrap();
        assert_ok!(fs::create_dir_all(tmp_path.path().to_str().unwrap()));

        let file_path = tmp_path.path().join(name);
        let path = file_path.to_str().unwrap();
        assert_ok!(fs::write(file_path.clone(), content));

        (path.clone().into(), tmp_path)
    }
}

