use telekinesis_plug::{self, Tk};
use lazy_static::lazy_static;
use nonparallel::nonparallel;

use std::thread;
use std::time::Duration;
use std::sync::Mutex;
use tracing::Level;

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

#[test]
#[nonparallel(M)]
fn scan_vibrate_and_stop_events_are_returned_e2e() {
    // arrange
    let mut tk = telekinesis_plug::new_with_default_settings();

    // act & assert
    tk.scan_for_devices();
    thread::sleep(Duration::from_secs(5));
    tk.get_next_event().unwrap().to_string().contains("connected");

    tk.vibrate_all(1.0);
    _sleep();
    tk.get_next_event().unwrap().to_string().contains("Vibrating");

    tk.vibrate_all(0.5);
    _sleep();
    tk.get_next_event().unwrap().to_string().contains("Vibrating");

    tk.stop_all();
    _sleep();
    tk.get_next_event().unwrap().to_string().contains("Stopping");

    tk.disconnect();
    let _ = tk.get_next_event();
}

#[test]
#[nonparallel(M)]
fn scan_vibrate_and_stop_events_are_queued_e2e() {
    // arrange
    let mut tk = telekinesis_plug::new_with_default_settings();

    // act
    tk.scan_for_devices();
    thread::sleep(Duration::from_secs(5));
    tk.vibrate_all(1.0);
    tk.vibrate_all(0.5);
    tk.stop_all();
    thread::sleep(Duration::from_secs(2));
    tk.disconnect();

    // assert
    assert!(tk.get_next_event().unwrap().to_string().contains("connected"));
    assert!(tk.get_next_event().unwrap().to_string().contains("Vibrating"));
    assert!(tk.get_next_event().unwrap().to_string().contains("Vibrating"));
    assert!(tk.get_next_event().unwrap().to_string().contains("Stopping"));
}

fn _sleep() {
    thread::sleep(Duration::from_millis(200));
}
