use buttplug::client::ButtplugClientDevice;
use event::TkEvent;
use lazy_static::lazy_static;
use tracing_subscriber::field::debug;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use tracing::{error, info, instrument, debug};

use cxx::{CxxString, CxxVector};
use telekinesis::{in_process_connector, Telekinesis};

use crate::{
    inputs::{as_string_list, Speed},
    settings::{TkSettings, SETTINGS_FILE, SETTINGS_PATH},
};

mod commands;
mod event;
mod fakes;
mod inputs;
mod logging;
mod settings;
mod telekinesis;
mod tests;
mod util;

/// The ffi interfaces called as papyrus native functions. This is very thin glue code to
/// store the global singleton state in a mutex and handle error conditions, and then
/// acess the functionality in the main Telekinesis struct
///
/// - All ffi methods are non-blocking, triggering an async action somewhere in the future
/// - All all error conditions during the function call (i.e. mutex not available) will
///   be swallowed and logged to Telekinesis.log
#[cxx::bridge]
mod ffi {
    extern "Rust" {
        fn tk_connect() -> bool;
        fn tk_connect_and_scan() -> bool;
        fn tk_scan_for_devices() -> bool;
        fn tk_get_device_names() -> Vec<String>;
        fn tk_get_device_connected(name: &str) -> bool;
        fn tk_get_device_capabilities(name: &str) -> Vec<String>;
        fn tk_vibrate(speed: i64, duration_sec: u64) -> bool;
        fn tk_vibrate_events(speed: i64, duration_sec: u64, devices: &CxxVector<CxxString>)
            -> bool;
        fn tk_vibrate_all(speed: i64) -> bool;
        fn tk_vibrate_all_for(speed: i64, duration_sec: u64) -> bool;
        fn tk_stop_all() -> bool;
        fn tk_close() -> bool;
        fn tk_poll_events() -> Vec<String>;
        fn tk_settings_set_enabled(name: &str, enabled: bool);
        fn tk_settings_get_enabled(name: &str) -> bool;
        fn tk_settings_store() -> bool;
    }
}

/// access to Telekinesis struct from within foreign rust modules and tests
pub trait Tk {
    fn scan_for_devices(&self) -> bool;
    fn get_devices(&self) -> Vec<Arc<ButtplugClientDevice>>;
    fn get_device_names(&self) -> Vec<String>;
    fn get_device_connected(&self, name: &str) -> bool;
    fn get_device_capabilities(&self, name: &str) -> Vec<String>;
    fn vibrate(&self, speed: Speed, duration: Duration, device_names: Vec<String>) -> bool;
    fn vibrate_all(&self, speed: Speed, duration: Duration) -> bool;
    fn stop_all(&self) -> bool;
    fn disconnect(&mut self);
    fn get_next_event(&mut self) -> Option<TkEvent>;
    fn get_next_events(&mut self) -> Vec<TkEvent>;
    fn settings_set_enabled(&mut self, device_name: &str, enabled: bool);
    fn settings_get_enabled(&self, device_name: &str) -> bool;
}

pub fn new_with_default_settings() -> impl Tk {
    Telekinesis::connect_with(|| async move { in_process_connector() }, None).unwrap()
}

lazy_static! {
    static ref TK: Mutex<Option<Telekinesis>> = Mutex::new(None);
}

fn access_mutex<F, R>(func: F) -> Option<R>
where
    F: FnOnce(&mut Telekinesis) -> R,
{
    if let Ok(mut guard) = TK.try_lock() {
        match guard.take() {
            Some(mut tk) => {
                let result = Some(func(&mut tk));
                guard.replace(tk);
                return result;
            }
            None => error!("Trying to call method on non-initialized tk"),
        }
    }
    None
}

#[instrument]
pub fn tk_connect() -> bool {
    tk_connect_with_settings(Some(TkSettings::try_read_or_default(
        SETTINGS_PATH,
        SETTINGS_FILE,
    )))
}

pub fn tk_connect_with_settings(settings: Option<TkSettings>) -> bool {
    info!("Creating new connection");
    match Telekinesis::connect_with(|| async move { in_process_connector() }, settings) {
        Ok(tk) => {
            match TK.try_lock() {
                Ok(mut guard) => {
                    guard.replace(tk);
                }
                Err(err) => error!("Failed locking mutex: {}", err),
            }
            true
        }
        Err(err) => {
            error!("tk_connect error {:?}", err);
            false
        }
    }
}

#[instrument]
pub fn tk_close() -> bool {
    info!("Closing connection");
    match TK.try_lock() {
        Ok(mut guard) => {
            if let Some(mut tk) = guard.take() {
                tk.disconnect();
                return true;
            }
        }
        Err(err) => error!("Failed locking mutex: {}", err),
    }
    false
}

#[instrument]
pub fn tk_connect_and_scan() -> bool {
    tk_connect() && tk_scan_for_devices()
}

#[instrument]
pub fn tk_scan_for_devices() -> bool {
    access_mutex(|tk| tk.scan_for_devices()).is_some()
}

#[instrument]
pub fn tk_vibrate(speed: i64, secs: u64) -> bool {
    access_mutex(|tk| tk.vibrate(Speed::new(speed), Duration::from_secs(secs), vec![])).is_some()
}

#[instrument]
pub fn tk_vibrate_events(speed: i64, secs: u64, events: &CxxVector<CxxString>) -> bool {
    access_mutex(|tk| {
        tk.vibrate(
            Speed::new(speed),
            Duration::from_secs(secs),
            as_string_list(&events),
        )
    })
    .is_some()
}

// deprecated
#[instrument]
pub fn tk_vibrate_all(speed: i64) -> bool {
    access_mutex(|tk| tk.vibrate_all(Speed::new(speed), Duration::from_secs(30))).is_some()
}

#[instrument]
pub fn tk_vibrate_all_for(speed: i64, secs: u64) -> bool {
    access_mutex(|tk| tk.vibrate_all(Speed::new(speed), Duration::from_secs(secs))).is_some()
}

#[instrument]
pub fn tk_get_device_names() -> Vec<String> {
    if let Some(value) = access_mutex(|tk| tk.get_device_names()) {
        return value;
    }
    vec![]
}

#[instrument]
pub fn tk_get_device_connected(name: &str) -> bool {
    if let Some(value) = access_mutex(|tk| tk.get_device_connected(name)) {
        return value;
    }
    false
}

#[instrument]
pub fn tk_get_device_capabilities(name: &str) -> Vec<String> {
    if let Some(value) = access_mutex(|tk| tk.get_device_capabilities(name)) {
        return value;
    }
    vec![]
}

#[instrument]
pub fn tk_stop_all() -> bool {
    access_mutex(|tk| tk.stop_all()).is_some()
}

#[instrument]
pub fn tk_poll_events() -> Vec<String> {
    match access_mutex(|tk| {
        let events = tk
            .get_next_events()
            .iter()
            .map(|evt| evt.to_string())
            .collect::<Vec<String>>();
        return events;
    }) {
        Some(events) => events,
        None => vec![],
    }
}

#[instrument]
pub fn tk_settings_set_enabled(device_name: &str, enabled: bool) {
    access_mutex(|tk| tk.settings_set_enabled(device_name, enabled));
}

#[instrument]
pub fn tk_settings_get_enabled(device_name: &str) -> bool {
    match access_mutex(|tk| tk.settings_get_enabled(device_name)) {
        Some(enabled) => {
            enabled
        },
        None => false,
    }
}

#[instrument]
pub fn tk_settings_store() -> bool {
    access_mutex(|tk| tk.settings.try_write(SETTINGS_PATH, SETTINGS_FILE)).is_some()
}
