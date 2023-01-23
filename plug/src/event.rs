use std::{sync::Arc, fmt::Display, fmt};
use buttplug::{client::{ButtplugClientDevice, ButtplugClientEvent}, core::errors::ButtplugError};

pub enum TkEvent {
    DeviceAdded(Arc<ButtplugClientDevice>),
    DeviceRemoved(Arc<ButtplugClientDevice>),
    DeviceVibrated(i32),
    DeviceStopped(i32),
    TkError(ButtplugError),
    Other(ButtplugClientEvent),
}

impl Display for TkEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let _ = match self {
            TkEvent::DeviceAdded(device) => write!(f, "Device '{}' connected.", device.name()),
            TkEvent::DeviceRemoved(device) => write!(f, "Device '{}' Removed.", device.name()),
            TkEvent::DeviceVibrated(speed) => write!(f, "Vibrating '{}' devices.", speed),
            TkEvent::DeviceStopped(speed) => write!(f, "Stopping '{}' devices.", speed),
            TkEvent::TkError(err) => write!(f, "Error '{:?}'", err),
            TkEvent::Other(other) => write!(f, "{:?}", other),
        };
        Ok(())
    }
}

impl TkEvent {
    pub fn from_event(event: ButtplugClientEvent) -> TkEvent {
        match event {
            ButtplugClientEvent::DeviceAdded(device) => TkEvent::DeviceAdded(device),
            ButtplugClientEvent::DeviceRemoved(device) => TkEvent::DeviceRemoved(device),
            ButtplugClientEvent::Error(err) => TkEvent::TkError(err),
            other => TkEvent::Other(other),
        }
    }
}