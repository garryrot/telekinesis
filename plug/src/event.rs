use std::{sync::Arc, fmt::Display, fmt};
use buttplug::{client::{ButtplugClientDevice, ButtplugClientEvent}, core::errors::ButtplugError};

use crate::Speed;

#[derive(Debug)]
pub enum TkEvent {
    ScanStarted,
    ScanStopped,
    DeviceAdded(Arc<ButtplugClientDevice>),
    DeviceRemoved(Arc<ButtplugClientDevice>),
    DeviceVibrated(i32, Speed),
    DeviceStopped(),
    TkError(ButtplugError),
    Other(ButtplugClientEvent),
}

impl Display for TkEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let _ = match self {
            TkEvent::DeviceAdded(device) => write!(f, "Device '{}' connected.", device.name()),
            TkEvent::DeviceRemoved(device) => write!(f, "Device '{}' Removed.", device.name()),
            TkEvent::DeviceVibrated(count, speed) => write!(f, "Vibrating {} device(s) {}%.", count, speed),
            TkEvent::DeviceStopped() => write!(f, "Stopping all devices."),
            TkEvent::TkError(err) => write!(f, "Error '{:?}'", err),
            TkEvent::Other(other) => write!(f, "{:?}", other),
            TkEvent::ScanStarted => write!(f, "Started scanning for devices"),
            TkEvent::ScanStopped => write!(f, "Stopped scanning for devices"),
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