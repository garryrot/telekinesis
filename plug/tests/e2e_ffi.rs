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

// These tests asserts that exactly one vibrating device is connected
// and will not work without actual hardwarce connected via usb
#[test]
#[nonparallel(M)]
fn ffi_test_multiple_reconnects() {
    enable_log();
    _ffi_test_event_polling();
    _ffi_test_event_polling();
    _ffi_test_event_polling();
}

#[test]
#[nonparallel(M)]
fn ffi_test_event_polling() {
    _ffi_test_event_polling();
}

fn _ffi_test_event_polling() {
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
 