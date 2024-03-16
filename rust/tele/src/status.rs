use std::{
    fmt::{self, Display},
    sync::Arc,
};
use crossbeam_channel::Receiver;
use itertools::Itertools;
use tracing::debug;

use buttplug::client::ButtplugClientDevice;

use bp_scheduler::actuator::{get_actuators, Actuator};

use crate::{connection::TkConnectionEvent, settings::TkSettings};

pub struct Status {
    status_events: Receiver<TkConnectionEvent>,
    connection: TkConnectionStatus,
    actuators: Vec<(Arc<Actuator>, TkConnectionStatus)>,
    known_actuators: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TkConnectionStatus {
    NotConnected,
    Connected,
    Failed(String),
}

impl Status {
    pub fn new(receiver: Receiver<TkConnectionEvent>, settings: &TkSettings) -> Self {
        Status {
            status_events: receiver,
            connection: TkConnectionStatus::NotConnected,
            actuators: vec![],
            known_actuators: settings
                .devices
                .iter()
                .map(|x| x.actuator_id.clone())
                .collect(),
        }
    }

    pub fn connection_status(&mut self) -> TkConnectionStatus {
        self.process_status_events();
        self.connection.clone()
    }

    pub fn actuators(&mut self) -> Vec<Arc<Actuator>> {
        self.process_status_events();
        self.actuators.iter().map(|x| x.0.clone()).collect()
    }

    pub fn connected_actuators(&mut self) -> Vec<Arc<Actuator>> {
        self.process_status_events();
        self.actuators
            .iter()
            .filter(|x| x.1 != TkConnectionStatus::NotConnected)
            .map(|x| x.0.clone())
            .collect()
    }

    pub fn actuator_status(&mut self) -> &Vec<(Arc<Actuator>, TkConnectionStatus)> {
        self.process_status_events();
        &self.actuators
    }

    pub fn get_actuator(&mut self, actuator_id: &str) -> Option<Arc<Actuator>> {
        self.actuators()
            .iter()
            .find(|x| x.identifier() == actuator_id)
            .cloned()
    }

    pub fn get_actuator_status(&mut self, actuator_id: &str) -> TkConnectionStatus {
        self.process_status_events();
        let entry: Option<&(Arc<Actuator>, TkConnectionStatus)> = self
            .actuator_status()
            .iter()
            .find(|x| x.0.identifier() == actuator_id);
        if let Some(status) = entry {
            return status.1.clone();
        }
        TkConnectionStatus::NotConnected
    }

    pub fn get_known_actuator_ids(&mut self) -> Vec<String> {
        let known_ids = self.known_actuators.clone();
        self.actuators()
            .iter()
            .map(|x| String::from(x.identifier()))
            .chain(known_ids)
            .unique()
            .collect()
    }

    pub fn process_status_events(&mut self) {
        while let Ok(evt) = self.status_events.try_recv() {
            debug!("processing status event {:?}", evt);
            match evt {
                TkConnectionEvent::Connected(_) => self.connection = TkConnectionStatus::Connected,
                TkConnectionEvent::ConnectionFailure(err) => {
                    self.connection = TkConnectionStatus::Failed(err)
                }
                TkConnectionEvent::DeviceAdded(device) => {
                    self.set_status(device.clone(), TkConnectionStatus::Connected);
                }
                TkConnectionEvent::DeviceRemoved(device) => {
                    self.set_status(device.clone(), TkConnectionStatus::NotConnected)
                }
                TkConnectionEvent::ActionError(actuator, err) => {
                    self.set_status(actuator.device.clone(), TkConnectionStatus::Failed(err))
                }
                TkConnectionEvent::ActionStarted(_, _, _, _) => {}
                TkConnectionEvent::ActionDone(_, _, _) => {}
            };
        }
    }

    fn set_status(&mut self, device: Arc<ButtplugClientDevice>, status: TkConnectionStatus) {
        let new_actuators = get_actuators(vec![device.clone()])
            .into_iter()
            .map(|x| (x, status.clone()));
        self.actuators = self
            .actuators
            .clone()
            .into_iter()
            .filter(|x| x.0.device.index() != device.index())
            .chain(new_actuators)
            .collect();
        debug!("device status updated: {:?}", self.actuators)
    }
}

impl Display for TkConnectionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            TkConnectionStatus::Failed(err) => write!(f, "{}", err),
            TkConnectionStatus::NotConnected => write!(f, "Not Connected"),
            TkConnectionStatus::Connected => write!(f, "Connected"),
        }
    }
}
