use buttplug::client::ButtplugClientDevice;
use event::TkEvent;
use lazy_static::lazy_static;
use pattern::get_pattern_names;
use settings::PATTERN_PATH;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use tracing::{error, info, instrument};

use cxx::{CxxString, CxxVector};
use telekinesis::{in_process_connector, Telekinesis, ERROR_HANDLE};

use crate::{
    inputs::{read_input_string, Speed},
    settings::{TkSettings, SETTINGS_FILE, SETTINGS_PATH},
};

mod commands;
mod event;
mod fakes;
mod inputs;
mod logging;
mod pattern;
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
        fn tk_scan_for_devices() -> bool;
        fn tk_stop_scan() -> bool;
        fn tk_get_devices() -> Vec<String>;
        fn tk_get_device_connected(device_name: &str) -> bool;
        fn tk_get_device_capabilities(device_name: &str) -> Vec<String>;
        fn tk_get_pattern_names(vibration_devices: bool) -> Vec<String>;
        fn tk_vibrate(speed: i64, secs: f32, events: &CxxVector<CxxString>) -> i32;
        fn tk_vibrate_pattern(pattern_name: &str, secs: f32, events: &CxxVector<CxxString>) -> i32;
        fn tk_stop(handle: i32) -> bool;
        fn tk_stop_all() -> bool;
        fn tk_close() -> bool;
        fn tk_poll_events() -> Vec<String>;
        fn tk_settings_set_enabled(device_name: &str, enabled: bool);
        fn tk_settings_get_enabled(device_name: &str) -> bool;
        fn tk_settings_get_events(device_name: &str) -> Vec<String>;
        fn tk_settings_set_events(device_name: &str, events: &CxxVector<CxxString>);
        fn tk_settings_store() -> bool;
    }
}

/// access to Telekinesis struct from within foreign rust modules and tests
pub trait Tk {
    fn scan_for_devices(&self) -> bool;
    fn stop_scan(&self) -> bool;
    fn disconnect(&mut self);
    fn get_devices(&self) -> Vec<Arc<ButtplugClientDevice>>;
    fn get_device_names(&self) -> Vec<String>;
    fn get_device_connected(&self, device_name: &str) -> bool;
    fn get_device_capabilities(&self, device_name: &str) -> Vec<String>;
    fn vibrate(&mut self, speed: Speed, duration: TkDuration, events: Vec<String>) -> i32;
    fn vibrate_pattern(&mut self, pattern: TkPattern, events: Vec<String>) -> i32;
    fn stop(&self, handle: i32) -> bool;
    fn stop_all(&self) -> bool;
    fn vibrate_all(&mut self, speed: Speed, duration: TkDuration) -> i32; // obsolete
    fn get_next_event(&mut self) -> Option<TkEvent>;
    fn get_next_events(&mut self) -> Vec<TkEvent>;
    fn settings_set_enabled(&mut self, device_name: &str, enabled: bool);
    fn settings_set_events(&mut self, device_name: &str, events: Vec<String>);
    fn settings_get_events(&self, device_name: &str) -> Vec<String>;
    fn settings_get_enabled(&self, device_name: &str) -> bool;
}

#[derive(Clone, Debug)]
pub enum TkDuration {
    Infinite,
    Timed(Duration)
}

impl TkDuration {
    pub fn from_input_float(secs: f32) -> TkDuration {
        if secs > 0.0 {
            return TkDuration::Timed(Duration::from_millis((secs * 1000.0) as u64));
        }
        else {
            return TkDuration::Infinite;
        }
    }
    pub fn from_millis(ms: u64) -> TkDuration {
        TkDuration::Timed(Duration::from_millis(ms))
    }
    pub fn from_secs(s: u64) -> TkDuration {
        TkDuration::Timed(Duration::from_secs(s))
    }
}

#[derive(Clone, Debug)]
pub enum TkPattern {
    Linear(TkDuration, Speed),
    Funscript(TkDuration, String)
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
pub fn tk_scan_for_devices() -> bool {
    access_mutex(|tk| tk.scan_for_devices()).is_some()
}

#[instrument]
pub fn tk_stop_scan() -> bool {
    access_mutex(|tk| tk.stop_scan()).is_some()
}

#[instrument]
pub fn tk_vibrate(speed: i64, secs: f32, events: &CxxVector<CxxString>) -> i32 {
    access_mutex(|tk| {
        tk.vibrate(
            Speed::new(speed),
            TkDuration::from_input_float(secs),
            read_input_string(&events),
        )
    }).unwrap_or(ERROR_HANDLE)
}

#[instrument]
pub fn tk_vibrate_pattern(pattern_name: &str, secs: f32, events: &CxxVector<CxxString>) -> i32 {
    access_mutex(|tk| {
        tk.vibrate_pattern(
            TkPattern::Funscript(TkDuration::from_input_float(secs), String::from(pattern_name)),
            read_input_string(&events),
        )
    }).unwrap_or(ERROR_HANDLE)
}

#[instrument]
pub fn tk_stop(handle: i32) -> bool {
    access_mutex(|tk| {
        tk.stop(handle)
    })
    .is_some()
}

#[instrument]
pub fn tk_get_devices() -> Vec<String> {
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
pub fn tk_get_pattern_names(vibration_patterns: bool) -> Vec<String> {
    get_pattern_names(PATTERN_PATH, vibration_patterns)
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
pub fn tk_settings_get_events(device_name: &str) -> Vec<String> {
    match access_mutex(|tk| tk.settings_get_events(device_name)) {
        Some(events) => events,
        None => vec![],
    }
}

#[instrument]
pub fn tk_settings_set_events(device_name: &str, events: &CxxVector<CxxString>) {
    access_mutex(|tk| tk.settings_set_events(device_name, read_input_string(events)));
}

#[instrument]
pub fn tk_settings_get_enabled(device_name: &str) -> bool {
    match access_mutex(|tk| tk.settings_get_enabled(device_name)) {
        Some(enabled) => enabled,
        None => false,
    }
}

#[instrument]
pub fn tk_settings_store() -> bool {
    access_mutex(|tk| tk.settings.try_write(SETTINGS_PATH, SETTINGS_FILE)).is_some()
}
