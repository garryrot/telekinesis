use telekinesis_plug::*;

use std::{ffi::{c_void, CString}, time::Duration, thread};
use lazy_static::lazy_static;
use nonparallel::nonparallel;
use std::sync::Mutex;

lazy_static! { static ref M: Mutex<()> = Mutex::new(()); }

// Asserts that exactly one device is connected

#[test]
#[nonparallel(M)]
fn ffi_connect_scan_and_vibrate_devices_2e2() {
    _ffi_connect_scan_and_vibrate_devices();
}

#[test]
#[nonparallel(M)]
fn ffi_connect_scan_and_vibrate_devices_works_after_reconnect_e2e() {
    _ffi_connect_scan_and_vibrate_devices();
    _ffi_connect_scan_and_vibrate_devices();
}

fn _poll_next_event(tk: *const c_void) -> CString {
    loop {
        let event = tk_try_get_next_event(tk);
        if event.is_null() {
            println!("Polling...");
            thread::sleep(Duration::from_secs(1));
        } else {
            let raw_string = unsafe { CString::from_raw(event) };
            return raw_string;
        }
    }
}

fn _assert_event(tk: *const c_void, raw_string: CString, starts_with: &str) {
    assert!(raw_string.to_str().unwrap().starts_with(starts_with));
    tk_free_event(tk, raw_string.into_raw());
}

fn _ffi_connect_scan_and_vibrate_devices() {
    let tk = tk_connect();
    tk_scan_for_devices(tk);
    thread::sleep(Duration::from_secs(5));
    _assert_event(tk, _poll_next_event(tk), "Device");
    tk_vibrate_all(tk, 1.0);
    thread::sleep(Duration::from_millis(200));
    _assert_event(tk, _poll_next_event(tk), "Vibrating");
    tk_stop_all(tk);
    thread::sleep(Duration::from_millis(200));
    _assert_event(tk, _poll_next_event(tk), "Stopping");
    tk_close(tk);
}
