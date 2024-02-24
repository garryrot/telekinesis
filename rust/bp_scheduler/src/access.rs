use buttplug::client::{ButtplugClientError, ScalarCommand};
use std::collections::HashMap;

use std::sync::Arc;
use tracing::{debug, error, trace, instrument};

use crate::{actuator::Actuator, speed::Speed};

/// Stores information about concurrent accesses to a buttplug actuator
/// to calculate the actual vibration speed or linear movement
pub struct DeviceEntry {
    /// The amount of tasks that currently access this device,
    pub task_count: usize,
    /// Priority calculation work like a stack with the top of the stack
    /// task being the used vibration speed
    pub linear_tasks: Vec<(i32, Speed)>,
}

pub struct DeviceAccess {
    device_actions: HashMap<String, DeviceEntry>,
}

impl DeviceAccess {
    pub fn default() -> Self {
        DeviceAccess {
            device_actions: HashMap::new(),
        }
    }

    pub async fn start_scalar(
        &mut self,
        actuator: &Arc<Actuator>,
        speed: Speed,
        is_pattern: bool,
        handle: i32,
    ) {
        trace!("start scalar {:?} {} {}", speed, actuator, handle);
        self.device_actions
            .entry(actuator.identifier().into())
            .and_modify(|entry| {
                entry.task_count += 1;
                if ! is_pattern {
                    entry.linear_tasks.push((handle, speed))
                }
            })
            .or_insert_with(|| DeviceEntry {
                task_count: 1,
                linear_tasks: if is_pattern {
                    vec![]
                } else {
                    vec![(handle, speed)]
                },
            });
        let _ = self.set_scalar(actuator, speed).await;
    }

    #[instrument(skip(self))]
    pub async fn stop_scalar(
        &mut self,
        actuator: &Arc<Actuator>,
        is_pattern: bool,
        handle: i32,
    ) -> Result<(), ButtplugClientError> {
        trace!("stop scalar");
        if let Some(mut entry) = self.device_actions.remove(actuator.identifier()) {
            if ! is_pattern {
                entry.linear_tasks.retain(|t| t.0 != handle);
            }
            let mut count = entry.task_count;
            count = count.saturating_sub(1);
            entry.task_count = count;
            self.device_actions.insert(actuator.identifier().into(), entry);
            if count == 0 {
                // nothing else is controlling the device, stop it
                return self.set_scalar(actuator, Speed::min()).await;
            } else if let Some(last_speed) = self.get_priority_speed(actuator) {
                let _ = self.set_scalar(actuator, last_speed).await;
            }
        }
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn update_scalar(&mut self, actuator: &Arc<Actuator>, new_speed: Speed, is_pattern: bool, handle: i32) {
        trace!("update scalar scalar");
        if ! is_pattern {
            self.device_actions.entry(actuator.identifier().into()).and_modify(|entry| {
                entry.linear_tasks = entry.linear_tasks.iter().map(|t| {
                    if t.0 == handle {
                        return (handle, new_speed);
                    }
                    *t
                }).collect()
            });
        }
        let speed = self.get_priority_speed(actuator).unwrap_or(new_speed);
        debug!("updating {} speed to {}", actuator, speed);
        let _ = self.set_scalar(actuator, speed).await;
    }

    #[instrument(skip(self))]
    async fn set_scalar(
        &self,
        actuator: &Arc<Actuator>,
        speed: Speed,
    ) -> Result<(), ButtplugClientError> {
        let cmd = ScalarCommand::ScalarMap(HashMap::from([(
            actuator.index_in_device,
            (speed.as_float(), actuator.actuator),
        )]));

        if let Err(err) = actuator.device.scalar(&cmd).await {
            error!("failed to set scalar speed {:?}", err);
            return Err(err);
        }
        Ok(())
    }

    fn get_priority_speed(&self, actuator: &Arc<Actuator>) -> Option<Speed> {
        if let Some(entry) = self.device_actions.get(actuator.identifier()) {
            let mut sorted: Vec<(i32, Speed)> = entry.linear_tasks.clone();
            sorted.sort_by_key(|b| b.0);
            if let Some(tuple) = sorted.last() {
                return Some(tuple.1);
            }
        }
        None
    }

    pub fn clear_all(&mut self) {
        self.device_actions.clear();
    }
}
