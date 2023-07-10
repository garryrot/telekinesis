use buttplug::{core::connector::ButtplugInProcessClientConnectorBuilder, server::{ButtplugServerBuilder, device::hardware::communication::btleplug::BtlePlugCommunicationManagerBuilder}};
use event::TkEvent;
use lazy_static::lazy_static;
use tracing::{
    error,
    info, instrument
};
use util::Narrow;
use std::{
    sync::RwLock,
    sync::RwLockWriteGuard,
    time::Duration, fmt::{Display, self},
};

use telekinesis::{
    Telekinesis,
    in_process_connector
};

mod commands;
mod fakes;
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

    let muh_callback = || async move { ButtplugInProcessClientConnectorBuilder::default()
        .server(
            ButtplugServerBuilder::default()
                .comm_manager(BtlePlugCommunicationManagerBuilder::default())
                .finish()
                .expect("Could not create in-process-server."),
        )
        .finish()
    };
    match Telekinesis::connect_with(muh_callback) {
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

// deprecated
#[instrument]
pub fn tk_vibrate_all(speed: i64) -> bool {
    let s = Speed::new(speed);
    let duration = Duration::from_secs(30);
    tk_ffi!(vibrate_all, s, duration)
}

#[instrument]
pub fn tk_vibrate_all_for(
    speed: i64,
    duration_sec: u64,
) -> bool {
    let duration_ms = Duration::from_millis(duration_sec * 1000.0 as u64);
    let s = Speed::new(speed);
    tk_ffi!(vibrate_all, s, duration_ms)
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
    Telekinesis::connect_with(|| async move { in_process_connector() }).unwrap()
}

pub trait Tk {
    fn scan_for_devices(&self) -> bool;
    fn vibrate(&self, speed: Speed, duration: Duration, devices: Vec<String>) -> bool;
    fn vibrate_all(&self, speed: Speed, duration: Duration) -> bool;
    fn stop_all(&self) -> bool;
    fn disconnect(&mut self);
    fn get_next_event(&mut self) -> Option<TkEvent>;
    fn get_next_events(&mut self) -> Vec<TkEvent>;
}

#[derive(Debug, Clone, Copy)]
pub struct Speed {
    pub value: u16 
}
impl Speed {
    pub fn new(percentage: i64) -> Speed {
        Speed { 
            value: percentage.narrow(0, 100) as u16
        }
    }
    pub fn min() -> Speed {
        Speed { value: 0 }
    }
    pub fn max() -> Speed {
        Speed { value: 100 }
    }
    pub fn as_float(self) -> f64 {
        self.value as f64 / 100.0
    } 
}
impl Display for Speed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}
