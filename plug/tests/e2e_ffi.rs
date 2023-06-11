use telekinesis_plug::*;
use tracing::Level;

use std::{ffi::{CString}, time::Duration, thread};
use lazy_static::lazy_static;
use nonparallel::nonparallel;
use std::sync::Mutex;

lazy_static! { static ref M: Mutex<()> = Mutex::new(()); }

#[allow(dead_code)]
fn enable_log() {
    tracing::subscriber::set_global_default(
        tracing_subscriber::fmt()
            .with_max_level(Level::INFO)
            .finish(),
    )
    .unwrap();
}

// Asserts that exactly one device is connected

#[test]
#[nonparallel(M)]
fn ffi_connect_scan_and_vibrate_devices_2e2() {
    enable_log();
    _ffi_connect_scan_and_vibrate_devices();
}

#[test]
#[nonparallel(M)]
fn ffi_connect_scan_and_vibrate_devices_works_after_reconnect_e2e() {
    enable_log();
    _ffi_connect_scan_and_vibrate_devices();
    _ffi_connect_scan_and_vibrate_devices();
    _ffi_connect_scan_and_vibrate_devices();
    _ffi_connect_scan_and_vibrate_devices();
    _ffi_connect_scan_and_vibrate_devices();
}

fn _poll_next_event() -> CString {
    loop {
        let event = tk_try_get_next_event();
        if event.is_null() {
            println!("Polling...");
            thread::sleep(Duration::from_secs(1));
        } else {
            let raw_string = unsafe { CString::from_raw(event) };
            return raw_string;
        }
    }
}

fn _assert_event(raw_string: CString, starts_with: &str) {
    assert!(raw_string.to_str().unwrap().starts_with(starts_with));
    tk_free_event(raw_string.into_raw());
}

fn _ffi_connect_scan_and_vibrate_devices() {
    tk_connect();
    tk_scan_for_devices();
    thread::sleep(Duration::from_secs(5));
    _assert_event(_poll_next_event(), "Device");
    tk_vibrate_all(100 );
    thread::sleep(Duration::from_millis(200));
    _assert_event(_poll_next_event(), "Vibrating");
    tk_stop_all();
    thread::sleep(Duration::from_millis(200));
    _assert_event(_poll_next_event(), "Stopping");
    tk_close();
}
 