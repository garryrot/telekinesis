use crate::connection::{TkDeviceEvent, TkConnectionEvent, TkConnectionStatus};

/// Serialized types parsed/read in papyrus

impl TkDeviceEvent {
    pub fn serialize_papyrus(&self) -> String {
        let device_list = self.devices.iter().map(|d| d.name().clone()).collect::<Vec<String>>().join(",");
        let event_list = self.events.join(",");
        format!("DeviceEvent|Vibrator|{:.1}s|{}|{}%|{}|{}", self.elapsed_sec, self.pattern, self.speed.value, device_list, event_list)
    }
}

impl TkConnectionEvent {
    pub fn serialize_papyrus(&self) -> String {
        match self {
            TkConnectionEvent::DeviceAdded(device) => format!("DeviceAdded|{}", device.name()),
            TkConnectionEvent::DeviceRemoved(device) => format!("DeviceRemoved|{}", device.name()),
            TkConnectionEvent::DeviceEvent(event) => event.serialize_papyrus(),
            TkConnectionEvent::Connected => format!("Connected"),
            TkConnectionEvent::ConnectionFailure =>  format!("ConnectionFailure"),
        }
    }
}

impl TkConnectionStatus {
    pub fn serialize_papyrus(&self) -> String {
        match &self {
            TkConnectionStatus::Failed(err) => format!("Failed|{:?}", err),
            TkConnectionStatus::NotConnected => String::from("Not Connected"),
            TkConnectionStatus::Connected => String::from("Connected"),
        }
    }
}
