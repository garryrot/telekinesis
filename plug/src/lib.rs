use event::TkEvent;
use lazy_static::lazy_static;
use tracing::{
    error,
    info, instrument
};
use std::{
    sync::RwLock,
    sync::RwLockWriteGuard,
    time::Duration,
};

use telekinesis::{Telekinesis, Speed};

mod commands;
mod event;
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
        fn tk_vibrate_all(speed: i64) -> bool;
        fn tk_vibrate_all_for(speed: i64, duration_sec: u64) -> bool;
        fn tk_stop_all() -> bool;
        fn tk_close() -> bool;
        fn tk_poll_events() -> Vec<String>;
    }
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

// FFI Library
#[instrument]
pub fn tk_connect_and_scan() -> bool {
    tk_connect() &&
    tk_scan_for_devices()
}

#[instrument]
pub fn tk_connect() -> bool {
    info!("Creating new connection");
    match Telekinesis::connect_with(telekinesis::in_process_server()) {
        Ok(tk) => {
            TK.write().unwrap().replace(tk);
            true
        }
        Err(e) => {
             error!("tk_connect error {:?}", e); 
             false
        }, 
    }
}

#[instrument]
pub fn tk_scan_for_devices() -> bool {
    tk_ffi!(scan_for_devices,)
}

#[instrument]
pub fn tk_vibrate_all(speed: i64) -> bool {
    let s = Speed::new(speed);
    tk_ffi!(vibrate_all, s)
}

#[instrument]
pub fn tk_vibrate_all_for(
    speed: i64,
    duration_sec: u64,
) -> bool {
    let duration_ms = Duration::from_millis(duration_sec * 1000.0 as u64);
    let s = Speed::new(speed);
    let stop = Speed::new(0);
    tk_ffi!(vibrate_all, s) &&
        tk_ffi!(vibrate_all_delayed, stop, duration_ms)
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
        let events = tk.get_next_events().iter().map(|evt| evt.to_string()).collect::<Vec<String>>();             
        guard.replace(tk);
        return events;
    }
    vec![]
}

// Rust Library
pub fn new_with_default_settings() -> impl Tk {
    Telekinesis::connect_with(telekinesis::in_process_server()).unwrap()
}

pub trait Tk {
    fn scan_for_devices(&self) -> bool;
    fn vibrate_all(&self, speed: Speed) -> bool;
    fn vibrate_all_delayed(&self, speed: Speed, duration: std::time::Duration) -> bool;
    fn stop_all(&self) -> bool;
    fn disconnect(&mut self);
    fn get_next_event(&mut self) -> Option<TkEvent>;
    fn get_next_events(&mut self) -> Vec<TkEvent>;
}
