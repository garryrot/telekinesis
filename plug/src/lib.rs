use event::TkEvent;
use lazy_static::lazy_static;
use tracing::{
    error,
    debug, info
};
use std::{
    ffi::{c_float, c_int},
    sync::RwLock,
    sync::RwLockWriteGuard,
    time::Duration,
};

use telekinesis::Telekinesis;

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
        fn tk_vibrate_all(speed: i32) -> bool;
        fn tk_vibrate_all_for(speed: i32, duration_sec: f32) -> bool;
        fn tk_stop_all() -> bool;
        fn tk_close() -> bool;
        fn tk_poll_events() -> Vec<String>;
    }
}

lazy_static! {
    static ref TK: RwLock<Option<Telekinesis>> = RwLock::new(None);
}
macro_rules! tk_ffi (
    ($call:ident, $( $arg:ident ),* ) => {
        match TK.read().unwrap().as_ref() {
            None => { 
                error!("[Papyrus] {}(): TK None", stringify!($call));
                false
            }, 
            Some(tk) => { 
                debug!("[Papyrus] {:?}({:?}) !!!!!!!!!!!-----", stringify!($call), stringify!($( $arg:ident ),*));
                tk.$call( $( $arg ),* )
            }
        }
    };
);

// FFI Library
pub fn tk_connect_and_scan() -> bool {
    tk_connect() &&
    tk_scan_for_devices()
}

pub fn tk_connect() -> bool {
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

pub fn tk_scan_for_devices() -> bool {
    tk_ffi!(scan_for_devices,)
}

pub fn tk_vibrate_all(speed: c_int) -> bool {
    let sp: f64 = speed as f64;
    tk_ffi!(vibrate_all, sp)
}

pub fn tk_vibrate_all_for(
    speed: c_int,
    duration_sec: c_float,
) -> bool {
    let duration_ms = Duration::from_millis((duration_sec * 1000.0) as u64);
    let sp = speed as f64;
    let stop = 0.0 as f64;

    tk_ffi!(vibrate_all, sp) &&
        tk_ffi!(vibrate_all_delayed, stop, duration_ms)
}

pub fn tk_stop_all() -> bool {
    tk_ffi!(stop_all,)
}

pub fn tk_close() -> bool {
    let tk = TK.write().unwrap().take();
    if let None = tk {
        return false;
    }
    tk.unwrap().disconnect();
    return true;
}

// PollEvents
pub fn tk_try_get_next_event() -> Option<String> {
    info!("tk_try_get_next_event");
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

pub fn tk_poll_events() -> Vec<String> {
    info!("tk_poll_events");
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
    fn vibrate_all(&self, speed: f64) -> bool;
    fn vibrate_all_delayed(&self, speed: f64, duration: std::time::Duration) -> bool;
    fn stop_all(&self) -> bool;
    fn disconnect(&mut self);
    fn get_next_event(&mut self) -> Option<TkEvent>;
    fn get_next_events(&mut self) -> Vec<TkEvent>;
}
