use std::{
    ffi::{c_float, c_void, CString},
    mem::forget,
    time::Duration,
};
use event::TkEvent;
use telekinesis::{Telekinesis};
use tracing::error;

mod commands;
mod event;
mod logging;
mod telekinesis;
mod tests;
mod util;

// Export as C FFI
#[no_mangle]
pub extern "C" fn tk_connect() -> *mut c_void {
    match Telekinesis::connect_with( telekinesis::in_process_server() ) {
        Ok(unwrapped) => Box::into_raw(Box::new(unwrapped)) as *mut c_void,
        Err(_) => {
            error!("Failed creating server.");
            std::ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "C" fn tk_scan_for_devices(_tk: *const c_void) -> bool {
    get_handle_unsafe(_tk).scan_for_devices()
}

#[no_mangle]
pub extern "C" fn tk_vibrate_all(_tk: *const c_void, speed: c_float) -> bool {
    get_handle_unsafe(_tk).vibrate_all(speed.into())
}

#[no_mangle]
pub extern "C" fn tk_vibrate_all_for(_tk: *const c_void, speed: c_float, dur_sec: c_float) -> bool {
    let handle = get_handle_unsafe(_tk);

    handle.vibrate_all(speed.into() )
        && handle.vibrate_all_delayed(0.0, Duration::from_millis((dur_sec * 1000.0) as u64))
}

#[no_mangle]
pub extern "C" fn tk_try_get_next_event(_tk: *const c_void) -> *mut i8 {
    assert!(false == _tk.is_null());
    let mut tk = unsafe { Box::from_raw(_tk as *mut Telekinesis) };
    let evt = tk.get_next_event();
    forget(tk);
    if let Some(ok) = evt {
        CString::new(ok.to_string()).unwrap().into_raw() as *mut i8
    } else {
        std::ptr::null_mut()
    }
}

#[no_mangle]
pub extern "C" fn tk_free_event(_: *const c_void, event: *mut i8) {
    assert!(false == event.is_null());
    unsafe { CString::from_raw(event) }; // dealloc string
}

#[no_mangle]
pub extern "C" fn tk_stop_all(_tk: *const c_void) -> bool {
    get_handle_unsafe(_tk).stop_all()
}

#[no_mangle]
pub extern "C" fn tk_close(_tk: *mut c_void) {
    let mut tk = unsafe { Box::from_raw(_tk as *mut Telekinesis) };
    tk.disconnect();
}

fn get_handle_unsafe(tk: *const c_void) -> &'static Telekinesis {
    assert!(false == tk.is_null());
    unsafe { &*(tk as *const Telekinesis) }
}

// Export as rust library (for tests)
pub fn new_with_default_settings() -> impl Tk
{
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