
use buttplug::client::ButtplugClientDevice;
use buttplug::core::message::ActuatorType;
use std::{fmt::{self, Display}, sync::Arc};

#[derive(Clone, Debug)]
pub struct Actuator {
    pub device: Arc<ButtplugClientDevice>,
    pub actuator: ActuatorType,
    pub index_in_device: u32,
}

impl Actuator {
    pub fn identifier(&self) -> String { // TODO return ref with lifetime of Actuator?
        self.to_string()
    }
}

impl Display for Actuator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}[{}].{}",
            self.device.name(),
            self.index_in_device,
            self.actuator
        )
    }
}

pub fn get_actuators(devices: Vec<Arc<ButtplugClientDevice>>) -> Vec<Arc<Actuator>> {
    let mut actuators = vec![];
    for device in devices {
        if let Some(scalar_cmd) = device.message_attributes().scalar_cmd() {
            for (idx, scalar_cmd) in scalar_cmd.iter().enumerate() {
                actuators.push(Arc::new(Actuator {
                    device: device.clone(),
                    actuator: *scalar_cmd.actuator_type(),
                    index_in_device: idx as u32,
                }))
            }
        }
        if let Some(linear_cmd) = device.message_attributes().linear_cmd() {
            for (idx, _) in linear_cmd.iter().enumerate() {
                actuators.push(Arc::new(Actuator {
                    device: device.clone(),
                    actuator: ActuatorType::Position,
                    index_in_device: idx as u32,
                }));
            }
        }
        if let Some(rotate_cmd) = device.message_attributes().rotate_cmd() {
            for (idx, _) in rotate_cmd.iter().enumerate() {
                actuators.push(Arc::new(Actuator {
                    device: device.clone(),
                    actuator: ActuatorType::Rotate,
                    index_in_device: idx as u32,
                }))
            }
        }
    }
    actuators
}