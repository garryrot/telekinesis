use std::{sync::Arc, fmt::Display, fmt};
use buttplug::{client::{ButtplugClientDevice, ButtplugClientEvent, ButtplugClientError}, core::errors::ButtplugError};

use crate::Speed;

#[derive(Debug)]
pub enum TkEvent {
    ScanStarted,
    ScanFailed(ButtplugClientError),
    ScanStopped,
    DeviceAdded(Arc<ButtplugClientDevice>),
    DeviceRemoved(Arc<ButtplugClientDevice>),
    DeviceVibrated(i32, Speed),
    DeviceStopped(),
    StopAll(),
    TkError(ButtplugError),
    Other(ButtplugClientEvent),
}

impl Display for TkEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let _ = match self {
            // TODO: absolutely disgusting
            TkEvent::DeviceAdded(device) => write!(f, "Device '{}' connected.", device.name()),
            TkEvent::DeviceRemoved(device) => write!(f, "Device '{}' removed.", device.name()),
            TkEvent::DeviceVibrated(count, speed) => write!(f, "Vibrated {} device(s) {}%.", count, speed),
            TkEvent::DeviceStopped() => write!(f, "Stopped device(s)"),
            TkEvent::TkError(err) => write!(f, "Error '{:?}'", err),
            TkEvent::Other(other) => write!(f, "{:?}", other),
            TkEvent::ScanStarted => write!(f, "Started scanning for devices"),
            TkEvent::ScanStopped => write!(f, "Stopped scanning for devices"),
            TkEvent::StopAll() => write!(f, "Stopping all devices."),
            TkEvent::ScanFailed(err) => write!(f, "Scan failed {:?}", err),
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