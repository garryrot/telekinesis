use tracing::Level;

use lazy_static::lazy_static;
use std::sync::Mutex;

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

// #[test]
// #[ignore = "Requires vibrator to be connected via BTLE (vibrates it)"]
// #[nonparallel(M)]
// fn ffi_test_reconnect() {
//     enable_log();
//     test_vibration_e2e();
//     thread::sleep(Duration::from_secs(5));
//     test_vibration_e2e();
// }

#[test]
#[ignore = "Requires vibrator to be connected via BTLE (vibrates it)"]
fn test_intiface() {

    // let settings = TkSettings::default();
    // settings.connection = TkConnectionType::WebSocket("127.0.0.1:12345");

    // let mut tk = Telekinesis::connect(settings);
    // tk.scan_for_devices();

    // thread::sleep(Duration::from_secs(5));

    // for device in tk.get_devices() {
    //     tk.settings.set_enabled(device.name(), true);
    // }

    // tk.vibrate(Speed::max(), TkDuration::Infinite, vec![]);
    // thread::sleep(Duration::from_secs(5));

    // arrange
    // tk_connect_with_settings(None);
    // tk_scan_for_devices();
    // 

    // // act
    // tk_vibrate(100, 1, CxxVector::try_from());

    // thread::sleep(Duration::from_millis(200));
    // tk_stop_all();
    // thread::sleep(Duration::from_millis(200));
    // // assert
    // let events = tk_poll_events();
    // assert!(events[0].starts_with("Vibrated"));
    // assert!(events[1].starts_with("Stopping"));
    // tk_close();
}

// fn wait_for_device_connect( tk: &Telekinesis, duration: Duration) {
//     thread::sleep(duration);
//     let events = tk.get_next_events()[0];
    
//     if let TkEvent::DeviceAdded(d) = events {
        
//     }

//     // assert!(events[0].
//     // let mut split = events[1].split("'");s
//     // assert!( split.next().unwrap().starts_with("Device") );
//     // let device = split.next().unwrap();
//     // info!("Enabling device '{}'", device);
 
//     // tk_settings_set_enabled(device, true);
// }
