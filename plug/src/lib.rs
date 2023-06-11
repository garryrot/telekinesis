use event::TkEvent;
use lazy_static::lazy_static;
use tracing::{
    error,
    debug, info
};
use std::{
    ffi::{c_float, CString, c_int},
    sync::RwLock,
    time::Duration,
};

use telekinesis::Telekinesis;

mod commands;
mod event;
mod logging;
mod telekinesis;
mod tests;
mod util;

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
#[no_mangle]
pub extern "C" fn tk_connect_and_scan() -> bool {
    tk_connect() &&
    tk_scan_for_devices()
}

#[no_mangle]
pub extern "C" fn tk_connect() -> bool {
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

#[no_mangle]
pub extern "C" fn tk_scan_for_devices() -> bool {
    tk_ffi!(scan_for_devices,)
}

#[no_mangle]
pub extern "C" fn tk_vibrate_all(speed: c_int) -> bool {
    let sp: f64 = speed as f64;
    tk_ffi!(vibrate_all, sp)
}

#[no_mangle]
pub extern "C" fn tk_vibrate_all_for(
    speed: c_int,
    duration_sec: c_float,
) -> bool {
    let duration_ms = Duration::from_millis((duration_sec * 1000.0) as u64);
    let sp = speed as f64;
    let stop = 0.0 as f64;

    tk_ffi!(vibrate_all, sp) &&
        tk_ffi!(vibrate_all_delayed, stop, duration_ms)
}

#[no_mangle]
pub extern "C" fn tk_stop_all() -> bool {
    tk_ffi!(stop_all,)
}

#[no_mangle]
pub extern "C" fn tk_close() {
    let tk = TK.write().unwrap().take();
    if let None = tk {
        return;
    }
    tk.unwrap().disconnect();
}

// PollEvents

#[no_mangle]
pub extern "C" fn tk_try_get_next_event() -> *mut i8 {
    info!("tk_try_get_next_event");

    let mut str = std::ptr::null_mut();
    let mut aa = TK.write().unwrap();
    if let Some(mut tk) = aa.take() {
        if let Some(ok) = tk.get_next_event() {
            str = CString::new(ok.to_string()).unwrap().into_raw() as *mut i8;
        }
        aa.replace(tk); // put it back
    }
    str
}

#[no_mangle]
pub extern "C" fn tk_free_event(event: *mut i8) {
    assert!(false == event.is_null());
    unsafe { CString::from_raw(event) }; // deallocs string
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
}