use buttplug::client::ButtplugClientDevice;
use event::TkEvent;
use lazy_static::lazy_static;
use std::{
    sync::RwLock,
    sync::{Arc, RwLockWriteGuard},
    time::Duration,
};
use tracing::{error, info, instrument};

use cxx::{CxxString, CxxVector};
use telekinesis::{in_process_connector, Telekinesis};

use crate::inputs::{as_string_list, Speed};

mod commands;
mod event;
mod fakes;
mod inputs;
mod logging;
mod telekinesis;
mod tests;
mod util;

#[cxx::bridge]
mod ffi {
    extern "Rust" {
        fn tk_connect() -> bool;
        fn tk_connect_and_scan() -> bool;
        fn tk_scan_for_devices() -> bool;
        fn tk_get_device_names() -> Vec<String>;
        fn tk_get_device_connected(name: &str) -> bool;
        fn tk_get_device_capabilities(name: &str) -> Vec<String>;
        fn tk_vibrate(speed: i64, duration_sec: u64, devices: &CxxVector<CxxString>) -> bool;
        fn tk_vibrate_all(speed: i64) -> bool;
        fn tk_vibrate_all_for(speed: i64, duration_sec: u64) -> bool;
        fn tk_stop_all() -> bool;
        fn tk_close() -> bool;
        fn tk_poll_events() -> Vec<String>;
    }
}

// Rust Library
pub fn new_with_default_settings() -> impl Tk {
    Telekinesis::connect_with(|| async move { in_process_connector() }).unwrap()
}

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
}

lazy_static! {
    static ref TK: RwLock<Option<Telekinesis>> = RwLock::new(None);
}

macro_rules! tk_ffi (
    ($call:ident, $( $arg:tt ),* ) => {
        match TK.read().unwrap().as_ref() {
            None => {
                error!("FFI call on missing TK instance {}()", stringify!($call));
                false
            },
            Some(tk) => {
                info!("FFI");
                tk.$call( $( $arg ),* )
            }
        }
    };
);

#[instrument]
pub fn tk_connect_and_scan() -> bool {
    tk_connect() && tk_scan_for_devices()
}

#[instrument]
pub fn tk_connect() -> bool {
    info!("Creating new connection");
    match Telekinesis::connect_with(|| async move { in_process_connector() }) {
        Ok(tk) => {
            TK.write().unwrap().replace(tk);
            true
        }
        Err(e) => {
            error!("tk_connect error {:?}", e);
            false
        }
    }
}

#[instrument]
pub fn tk_scan_for_devices() -> bool {
    tk_ffi!(scan_for_devices,)
}

#[instrument]
pub fn tk_vibrate(speed: i64, secs: u64, device_names: &CxxVector<CxxString>) -> bool {
    let speed = Speed::new(speed);
    let duration = Duration::from_secs(secs as u64);
    let devices = as_string_list(device_names);
    tk_ffi!(vibrate, speed, duration, devices)
}

// deprecated
#[instrument]
pub fn tk_vibrate_all(speed: i64) -> bool {
    let speed = Speed::new(speed);
    let duration = Duration::from_secs(30);
    tk_ffi!(vibrate_all, speed, duration)
}

#[instrument]
pub fn tk_vibrate_all_for(speed: i64, secs: u64) -> bool {
    let speed = Speed::new(speed);
    let duration = Duration::from_secs(secs as u64);
    tk_ffi!(vibrate_all, speed, duration)
}

#[instrument]
pub fn tk_get_device_names() -> Vec<String> {
    if let Some(tk) = TK.write().unwrap().as_ref() {
        return tk.get_device_names()
    }
    vec![]
}

#[instrument]
pub fn tk_get_device_connected(name: &str) -> bool {
    if let Some(tk) = TK.write().unwrap().as_ref() {
        return tk.get_device_connected(name);
    }
    false
}

#[instrument]
pub fn tk_get_device_capabilities(name: &str) -> Vec<String> {
    if let Some(tk) = TK.write().unwrap().as_ref() {
        return tk.get_device_capabilities(name);
    }
    vec![]
}

#[instrument]
pub fn tk_stop_all() -> bool {
    tk_ffi!(stop_all,)
}

#[instrument]
pub fn tk_close() -> bool {
    info!("Closing connection");
    let tk = TK.write().unwrap().take();
    if let None = tk {
        return false;
    }
    tk.unwrap().disconnect();
    return true;
}

// PollEvents
#[instrument]
pub fn tk_poll_event() -> Option<String> {
    info!("Polling event");
    let mut evt = None;
    let mut guard: RwLockWriteGuard<'_, Option<Telekinesis>> = TK.write().unwrap();
    if let Some(mut tk) = guard.take() {
        if let Some(ok) = tk.get_next_event() {
            evt = Some(ok.to_string());
        }
        guard.replace(tk);
    }
    evt
}

#[instrument]
pub fn tk_poll_events() -> Vec<String> {
    info!("Polling all events");
    let mut guard: RwLockWriteGuard<'_, Option<Telekinesis>> = TK.write().unwrap();
    if let Some(mut tk) = guard.take() {
        let events = tk
            .get_next_events()
            .iter()
            .map(|evt| evt.to_string())
            .collect::<Vec<String>>();
        guard.replace(tk);
        return events;
    }
    vec![]
}
