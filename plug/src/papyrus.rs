use crate::{connection::{TkConnectionStatus}, pattern::Speed};


impl TkConnectionStatus {
    pub fn serialize_papyrus(&self) -> String {
        match &self {
            TkConnectionStatus::Failed(err) => String::from(err),
            TkConnectionStatus::NotConnected => String::from("Not Connected"),
            TkConnectionStatus::Connected => String::from("Connected"),
        }
    }
}
