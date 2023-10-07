use cxx::{CxxString, CxxVector};
use funscript::FSPoint;
use std::fmt::{self, Display};
use util::Narrow;

use crate::{settings::TkDeviceSettings, util, Speed, TkPattern, connection::{TkConnectionEvent, TkConnectionStatus, TkDeviceEvent}, DeviceList};

impl Speed {
    pub fn new(percentage: i64) -> Speed {
        Speed {
            value: percentage.narrow(0, 100) as u16,
        }
    }
    pub fn from_fs(point: &FSPoint) -> Speed {
        Speed::new(point.pos.into())
    }
    pub fn min() -> Speed {
        Speed { value: 0 }
    }
    pub fn max() -> Speed {
        Speed { value: 100 }
    }
    pub fn as_float(self) -> f64 {
        self.value as f64 / 100.0
    }
}

impl Display for Speed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

pub fn sanitize_name_list(list: &Vec<String>) -> Vec<String> {
    list.iter()
        .map(|e| String::from(e.to_lowercase().trim()))
        .collect()
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
        devices: DeviceList,
    ) -> DeviceList {
        devices
            .iter()
            .filter(|d| {
                self.selector.iter().any(|x| x == d.name())
                    && d.message_attributes().scalar_cmd().is_some()
            })
            .map(|d| d.clone())
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
