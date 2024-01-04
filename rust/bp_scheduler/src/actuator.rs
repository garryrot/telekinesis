use buttplug::client::ButtplugClientDevice;
use buttplug::core::message::ActuatorType;
use std::{
    fmt::{self, Display},
    sync::Arc,
};

#[derive(Clone)]
pub struct Actuator {
    pub device: Arc<ButtplugClientDevice>,
    pub actuator: ActuatorType,
    pub index_in_device: u32,
    identifier: String,
}

impl Actuator {
    pub fn new(
        device: &Arc<ButtplugClientDevice>,
        actuator: ActuatorType,
        index_in_device: usize,
    ) -> Self {
        let identifier = Actuator::get_identifier(device, actuator, index_in_device);
        Actuator {
            device: device.clone(),
            actuator,
            index_in_device: index_in_device as u32,
            identifier,
        }
    }

    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    fn get_identifier(
        device: &Arc<ButtplugClientDevice>,
        actuator: ActuatorType,
        index_in_device: usize,
    ) -> String {
        if index_in_device > 0 {
            return format!("{} ({} #{})", device.name(), actuator, index_in_device);
        }
        format!("{} ({})", device.name(), actuator)
    }
}

pub fn get_actuators(devices: Vec<Arc<ButtplugClientDevice>>) -> Vec<Arc<Actuator>> {
    let mut actuators = vec![];
    for device in devices {
        if let Some(scalar_cmd) = device.message_attributes().scalar_cmd() {
            for (idx, scalar_cmd) in scalar_cmd.iter().enumerate() {
                actuators.push(Actuator::new(&device, *scalar_cmd.actuator_type(), idx))
            }
        }
        if let Some(linear_cmd) = device.message_attributes().linear_cmd() {
            for (idx, _) in linear_cmd.iter().enumerate() {
                actuators.push(Actuator::new(&device, ActuatorType::Position, idx));
            }
        }
        if let Some(rotate_cmd) = device.message_attributes().rotate_cmd() {
            for (idx, _) in rotate_cmd.iter().enumerate() {
                actuators.push(Actuator::new(&device, ActuatorType::Rotate, idx))
            }
        }
    }
    actuators.into_iter().map(Arc::new).collect()
}

impl Display for Actuator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.identifier)
    }
}

impl fmt::Debug for Actuator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Actuator({})", self.identifier)
    }
}
