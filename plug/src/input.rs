use std::time::Duration;

use cxx::{CxxString, CxxVector};

use crate::{settings::TkDeviceSettings, TkPattern, connection::ActuatorList};

pub fn sanitize_name_list(list: &Vec<String>) -> Vec<String> {
    list.iter()
        .map(|e| String::from(e.to_lowercase().trim()))
        .collect()
}

pub fn parse_list_string(input: &str) -> Vec<String> {
    let mut list = vec![];
    for part in input.split(",") {
        if part.len() > 0 {
            list.push(String::from(part.trim().to_lowercase()));
        }
    }
    list
}

pub fn get_duration_from_secs(secs: f32) -> Duration {
    if secs > 0.0 {
        Duration::from_millis((secs * 1000.0) as u64)
    } else {
        Duration::MAX
    }
}

pub fn read_input_string(list: &CxxVector<CxxString>) -> Vec<String> {
    // automatically discards any empty strings to account for papyrus
    // inability to do dynamic array sizes
    list.iter()
        .filter(|d| d.len() > 0)
        .map(|d| d.to_string_lossy().into_owned())
        .collect()
}

#[derive(Clone, Debug)]
pub struct TkParams {
    pub selector: Vec<String>,
    pub pattern: TkPattern,
    pub events: Vec<String>
}

impl TkParams {
    pub fn filter_devices(
        &self,
        actuators: ActuatorList,
    ) -> ActuatorList {
        actuators
            .iter()
            .filter(|a| {
                self.selector.iter().any(|x| x == a.device.name())
                    && a.device.message_attributes().scalar_cmd().is_some()
            })
            .map(|a| a.clone())
            .collect()
    }

    pub fn from_input(
        events: Vec<String>,
        pattern: TkPattern,
        devices: &Vec<TkDeviceSettings>,
    ) -> Self {
        let event_names = sanitize_name_list(&events);
        let device_names = devices
            .iter()
            .filter(|d| {
                d.enabled
                    && (event_names.len() == 0 || d.events.iter().any(|e| event_names.contains(e)))
            })
            .map(|d| d.name.clone())
            .collect();
        TkParams {
            selector: device_names,
            pattern: pattern,
            events: events
        }
    }
}
