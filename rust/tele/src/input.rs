use std::{sync::Arc, time::Duration};

use bp_scheduler::{actuator::Actuator, settings::ActuatorSettings};
use buttplug::core::message::ActuatorType;
use cxx::{CxxString, CxxVector};
use tracing::debug;

use crate::settings::TkDeviceSettings;

pub fn sanitize_name_list(list: &[String]) -> Vec<String> {
    list.iter()
        .map(|e| e.to_lowercase().trim().to_owned())
        .collect()
}

pub fn parse_csv(input: &str) -> Vec<String> {
    let mut list = vec![];
    for part in input.split(',') {
        if !part.is_empty() {
            list.push(part.trim().to_lowercase());
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
        .filter(|d| !d.is_empty())
        .map(|d| d.to_string_lossy().into_owned())
        .collect()
}

pub struct TkParams {}

impl TkParams {
    pub fn filter_devices(
        actuators: &[Arc<Actuator>],
        input_body_parts: &[String],
        actuator_types: &[ActuatorType],
        device_settings: &[TkDeviceSettings]
        ) -> Vec<Arc<Actuator>> {
        let body_parts = sanitize_name_list(input_body_parts);
        let selected_settings = device_settings.iter().filter( |setting| { 
            if ! setting.enabled {
                return false;
            }
            if body_parts.is_empty() {
                return true;
            }
            setting.events.iter().any( |y| body_parts.contains(y) )
        }).cloned().collect::<Vec<TkDeviceSettings>>();

        let selected = selected_settings.iter().map(|x| x.actuator_id.clone()).collect::<Vec<String>>();
        
        let used = actuators
                .iter()
                .filter( |x| actuator_types.iter().any(|y| y == &x.actuator) )
                .filter( |x| selected.contains( & x.identifier().to_owned() ) )
                .cloned()
                .collect::<Vec<Arc<Actuator>>>();

        let actuator_settings = used.iter().map(|x| device_settings.iter().find( |y| y.actuator_id == x.identifier() ) );

        debug!("connected: {:?}", actuators.iter().map( |x| x.identifier() ).collect::<Vec<&str>>());
        debug!(?used);
        used
    }

}
