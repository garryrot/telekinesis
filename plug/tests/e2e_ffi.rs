use telekinesis_plug::*;
use tracing::{Level, info, instrument};

use lazy_static::lazy_static;
use nonparallel::nonparallel;
use std::sync::Mutex;
use std::{thread, time::Duration};

lazy_static! {
    static ref M: Mutex<()> = Mutex::new(());
}

#[allow(dead_code)]
fn enable_log() {
    tracing::subscriber::set_global_default(
        tracing_subscriber::fmt()
            .with_max_level(Level::DEBUG)
            .finish(),
    )
    .unwrap();
}

#[test]
#[ignore = "Requires vibrator to be connected via BTLE (vibrates it)"]
#[nonparallel(M)]
fn ffi_test_reconnect() {
    enable_log();
    test_vibration_e2e();
    thread::sleep(Duration::from_secs(5));
    test_vibration_e2e();
}

#[instrument]
fn test_vibration_e2e() {
    // arrange
    tk_connect();
    tk_scan_for_devices();
    wait_for_device_connect(Duration::from_secs(5));

    // act
    tk_vibrate(100, 1);
    thread::sleep(Duration::from_millis(200));
    tk_stop_all();
    thread::sleep(Duration::from_millis(200));

    // assert
    let events = tk_poll_events();
    assert!(events[0].starts_with("Vibrating"));
    assert!(events[1].starts_with("Stopping"));
    tk_close();
}

fn wait_for_device_connect(duration: Duration) {
    thread::sleep(duration);
    let events = tk_poll_events();
    let mut split = events[0].split("'");
    assert!( split.next().unwrap().starts_with("Device") );
    let device = split.next().unwrap();
    info!("Enabling device '{}'", device);
    tk_settings_set_enabled(device, true);
}
