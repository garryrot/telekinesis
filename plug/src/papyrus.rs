use crate::connection::{TkConnectionEvent, TkConnectionStatus, TkDeviceEvent};

/// Serialized types parsed/read in papyrus, if these change papyrus code has to change
impl TkDeviceEvent {
    pub fn serialize_papyrus(&self, is_error: bool) -> String {
        let device_list = self
            .devices
            .iter()
            .map(|d| d.name().clone())
            .collect::<Vec<String>>()
            .join(",");
        let event_list = self.events.join(",");
        let mut evt_name = "DeviceEvent";
        if is_error {
            evt_name = "DeviceError";
        }
        format!(
            "{}|Vibrator|{:.1}s|{}|{}%|{}|{}",
            evt_name, self.elapsed_sec, self.pattern, self.speed.value, device_list, event_list
        )
    }
}

// impl TkConnectionEvent {
//     pub fn serialize_papyrus(&self) -> String {
//         match self {
//             TkConnectionEvent::DeviceAdded(device) => format!("DeviceAdded|{}", device.name()),
//             TkConnectionEvent::DeviceRemoved(device) => format!("DeviceRemoved|{}", device.name()),
//             TkConnectionEvent::DeviceEvent(event) => event.serialize_papyrus(false),
//             TkConnectionEvent::DeviceError(event) => event.serialize_papyrus(true),
//             TkConnectionEvent::Connected(Error) => format!("Connected"),
//             TkConnectionEvent::ConnectionFailure(Error) => format!("ConnectionFailure"),
//         }
//     }
// }

impl TkConnectionStatus {
    pub fn serialize_papyrus(&self) -> String {
        match &self {
            TkConnectionStatus::Failed(err) => String::from(err),
            TkConnectionStatus::NotConnected => String::from("Not Connected"),
            TkConnectionStatus::Connected => String::from("Connected"),
        }
    }
}
