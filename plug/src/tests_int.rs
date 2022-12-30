#[cfg(test)]
mod tests_int {
    use crate::*;

    use futures::StreamExt;
    use std::ffi::c_void;

    use std::thread;
    use std::time::Duration;
    use tracing::Level;

    #[allow(dead_code)]
    fn enable_log() {
        tracing::subscriber::set_global_default(
            tracing_subscriber::fmt()
                .with_max_level(Level::INFO)
                .finish(),
        )
        .unwrap();
    }

    #[allow(dead_code)]
    fn tk_wait_for_first_device(_tk: *const c_void) {
        let tk = unsafe { &*(_tk as *const Telekinesis) };
        tk.runtime.block_on(async {
            let _ = tk.client.event_stream().next().await;
        });
    }

    fn _connect_scan_and_vibrate_devices() {
        let tk = tk_connect();
        let scanned = tk_scan_for_devices(tk);

        let mut done = false;
        while !done {
            let event = tk_await_next_event(tk);
            if event.is_null() {
                println!("Waiting for event...");
                thread::sleep(Duration::from_secs(1));
            } else {
                let raw_string = unsafe { CString::from_raw(event) };
                assert!(raw_string.to_str().unwrap().starts_with("Device"));
                forget(raw_string);
                tk_free_event(tk, event);
                done = true;
            }
        }

        let vibrated = tk_vibrate_all(tk, 1.0);
        thread::sleep(Duration::from_secs(1));
        let stopped = tk_stop_all(tk);
 
        tk_close(tk);
        assert!(vibrated == 1);
        assert!(stopped == 1);
        assert!(scanned);
    }

    #[test]
    fn connect_scan_and_vibrate_devices_2e2() {
        enable_log();
        _connect_scan_and_vibrate_devices();
    }

    #[test]
    fn connect_scan_and_vibrate_devices_works_after_reconnect_e2e() {
        _connect_scan_and_vibrate_devices();
        _connect_scan_and_vibrate_devices();
    }
}
