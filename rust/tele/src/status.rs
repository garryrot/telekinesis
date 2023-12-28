use std::{sync::Arc, fmt::{Display, self}};

use bp_scheduler::actuator::{Actuator, get_actuators};
use buttplug::client::ButtplugClientDevice;
use crossbeam_channel::Receiver;
use itertools::Itertools;
use tracing::error;

use crate::{connection::TkConnectionEvent, settings::TkSettings};

pub struct Status {
    status_events: Receiver<TkConnectionEvent>,
    connection: TkConnectionStatus,
    actuators: Vec<(Arc<Actuator>, TkConnectionStatus)>,
    known_actuators: Vec<String>
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
            known_actuators: settings.devices.iter().map(|x| x.actuator_id.clone()).collect()
        }
    }
    
    pub fn connection_status(&mut self) -> TkConnectionStatus {
        self.process_status_events();
        self.connection.clone()
    }
    
    pub fn actuators(&mut self) -> Vec<Arc<Actuator>> {
        self.process_status_events();
        self.actuators.iter().map( |x| x.0.clone() ).collect()
    }

    pub fn actuator_status(&mut self) -> &Vec<(Arc<Actuator>, TkConnectionStatus)> {
        self.process_status_events();
        &self.actuators
    }

    pub fn get_actuator(&mut self, actuator_id: &str) -> Option<Arc<Actuator>> {
        self.process_status_events();
        self.actuators().iter().find( |x| x.identifier() == actuator_id ).cloned()
    }

    pub fn get_actuator_status(&mut self, actuator_id: &str) -> TkConnectionStatus {
        let entry: Option<&(Arc<Actuator>, TkConnectionStatus)> = self.actuator_status().iter().find( |x| x.0.identifier() == actuator_id );
        if let Some(status) = entry {
            return status.1.clone();
        }
        TkConnectionStatus::NotConnected
    }

    pub fn get_known_actuator_ids(&mut self) -> Vec<String> {
        let known_ids = self.known_actuators.clone();
        self.actuators().iter().map( |x| String::from(x.identifier())).chain(known_ids).unique().collect()
    }

    pub fn process_status_events(&mut self) {
        error!("process status events");
        while let Ok(evt) = self.status_events.try_recv() {
            error!("got event {:?}", evt);
            match evt {
                TkConnectionEvent::Connected(_) => self.connection = TkConnectionStatus::Connected,
                TkConnectionEvent::ConnectionFailure(err) => self.connection = TkConnectionStatus::Failed(err),
                TkConnectionEvent::DeviceAdded(device) => {
                    self.set_status(device.clone(), TkConnectionStatus::Connected);
                    error!("New count: {:?}", self.actuators.len())
                },
                TkConnectionEvent::DeviceRemoved(device) =>  self.set_status(device.clone(), TkConnectionStatus::NotConnected),
                TkConnectionEvent::ActionError(actuator, err) => self.set_status(actuator.device.clone(), TkConnectionStatus::Failed(err)),
                TkConnectionEvent::ActionStarted(_, _, _, _) => {},
                TkConnectionEvent::ActionDone(_, _, _) => todo!(),
            }   
        }
    }
    
    fn set_status(&mut self, device: Arc<ButtplugClientDevice>, status: TkConnectionStatus) {
        let new_actuators = get_actuators(vec![device.clone()]).into_iter().map(|x| (x, status.clone()) );
        self.actuators = self.actuators.clone().into_iter().filter( |x| x.0.device.index() != device.index() ).chain(new_actuators).collect();
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
