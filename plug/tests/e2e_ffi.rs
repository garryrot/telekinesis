use telekinesis_plug::*;
use tracing::Level;

use std::{time::Duration, thread};
use lazy_static::lazy_static;
use nonparallel::nonparallel;
use std::sync::Mutex;

lazy_static! { static ref M: Mutex<()> = Mutex::new(()); }

#[allow(dead_code)]
fn enable_log() {
    tracing::subscriber::set_global_default(
        tracing_subscriber::fmt()
            .with_max_level(Level::DEBUG)
            .finish(),
    )
    .unwrap();
}

// Asserts that exactly one device is connected

#[test]
#[nonparallel(M)]
fn ffi_connect_scan_and_vibrate_devices_2e2() {
    // enable_log();
    _ffi_connect_scan_and_vibrate_devices();
}

#[test]
#[nonparallel(M)]
fn ffi_connect_scan_and_vibrate_devices_works_after_reconnect_e2e() {
    // enable_log();
    _ffi_connect_scan_and_vibrate_devices();
    _ffi_connect_scan_and_vibrate_devices();
    _ffi_connect_scan_and_vibrate_devices();
    _ffi_connect_scan_and_vibrate_devices();
    _ffi_connect_scan_and_vibrate_devices();
}

#[test]
#[nonparallel(M)]
fn ffi_test_event_polling() {    
    enable_log();
    tk_connect();
    tk_scan_for_devices();
    thread::sleep(Duration::from_secs(5));
    tk_vibrate_all(100);
    thread::sleep(Duration::from_millis(200));
    tk_stop_all();
    thread::sleep(Duration::from_millis(200));
    let events = tk_poll_events();
    assert!(events[0].starts_with("Device"));
    assert!(events[1].starts_with("Vibrating"));
    assert!(events[2].starts_with("Stopping"));
    tk_close();
    thread::sleep(Duration::from_secs(5));
}

fn _ffi_connect_scan_and_vibrate_devices() {
    tk_connect();
    tk_scan_for_devices();
    thread::sleep(Duration::from_secs(5));
    assert!(tk_poll_event().unwrap().starts_with("Device"));
    tk_vibrate_all(100 );
    thread::sleep(Duration::from_millis(200));
    assert!(tk_poll_event().unwrap().starts_with("Vibrating"));
    tk_stop_all();
    thread::sleep(Duration::from_millis(200));
    assert!(tk_poll_event().unwrap().starts_with("Stopping"));
    tk_close();
    thread::sleep(Duration::from_secs(5));
}
 