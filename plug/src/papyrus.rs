use crate::connection::{TkConnectionStatus, TkDeviceEvent, TkDeviceEventType};

/// Serialized types displayed in debug as a papyrus array
impl TkDeviceEvent {
    pub fn serialize_papyrus(&self) -> String {
        let device_list = self
            .devices
            .iter()
            .map(|d| d.name().clone())
            .collect::<Vec<String>>()
            .join(",");
        let tag_list = self.tags.join(",");
        let event_type = match self.event_type {
            TkDeviceEventType::Scalar => "Scalar",
            TkDeviceEventType::Linear => "Linear",
        };
        format!(
            "{}|{:.1}s|{}|{}%|{}|{}",
            event_type, self.elapsed_sec, self.pattern, self.speed.value, device_list, tag_list
        )
    }
}

impl TkConnectionStatus {
    pub fn serialize_papyrus(&self) -> String {
        match &self {
            TkConnectionStatus::Failed(err) => String::from(err),
            TkConnectionStatus::NotConnected => String::from("Not Connected"),
            TkConnectionStatus::Connected => String::from("Connected"),
        }
    }
}
