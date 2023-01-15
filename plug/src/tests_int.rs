#[cfg(test)]
mod tests_int {
    use crate::*;
    use lazy_static::lazy_static;
    use nonparallel::nonparallel;

    use std::ffi::c_void;

    use std::thread;
    use std::time::Duration;
    use std::sync::Mutex;
    use tracing::Level;

    lazy_static! { static ref M: Mutex<()> = Mutex::new(()); }
            
    fn _sleep1() {
        thread::sleep(Duration::from_millis(200));
    }

    #[allow(dead_code)]
    fn enable_log() {
        tracing::subscriber::set_global_default(
            tracing_subscriber::fmt()
                .with_max_level(Level::INFO)
                .finish(),
        )
        .unwrap();
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

    fn _assert_string(tk: *const c_void, raw_string: CString, starts_with: &str) {
        assert!(raw_string.to_str().unwrap().starts_with(starts_with));
        tk_free_event(tk, raw_string.into_raw());
    }

    fn _ffi_connect_scan_and_vibrate_devices() {
        let tk = tk_connect();
        tk_scan_for_devices(tk);
        thread::sleep(Duration::from_secs(5));
        _assert_string(tk, _poll_next_event(tk), "Device");
        tk_vibrate_all(tk, 1.0);
        _sleep1();
        _assert_string(tk, _poll_next_event(tk), "Vibrating");
        tk_stop_all(tk);
        _sleep1();
        _assert_string(tk, _poll_next_event(tk), "Stopping");
        tk_close(tk);
    }

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

    #[test]
    #[nonparallel(M)]
    fn scan_vibrate_and_stop_events_are_returned_e2e() {
        // arrange
        let mut tk: Telekinesis = Telekinesis::new_with_default_settings().unwrap();

        // act & assert
        tk.scan_for_devices();
        thread::sleep(Duration::from_secs(5));
        tk.get_next_event().unwrap().to_string().contains("connected");

        tk.vibrate_all(1.0);
        _sleep1();
        tk.get_next_event().unwrap().to_string().contains("Vibrating");

        tk.vibrate_all(0.5);
        _sleep1();
        tk.get_next_event().unwrap().to_string().contains("Vibrating");

        tk.stop_all();
        _sleep1();
        tk.get_next_event().unwrap().to_string().contains("Stopping");

        tk.disconnect();
        let _ = tk.get_next_event();
    }

    #[test]
    #[nonparallel(M)]
    fn scan_vibrate_and_stop_events_are_queued_e2e() {
        // arrange
        let mut tk: Telekinesis = Telekinesis::new_with_default_settings().unwrap();

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
}
