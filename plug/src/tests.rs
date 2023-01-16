#[cfg(test)]
mod tests {
    use crate::logging::{tk_init_logging, LogLevel};
    use crate::*;
    use std::ptr::null;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn init_logging_handles_nullpointr_without_panic() {
        tk_init_logging(LogLevel::Trace, null());
    }

    #[test]
    fn connect_and_scan() {
        let tk = tk_connect();
        tk_scan_for_devices(tk);
    }

    #[test]
    fn vibrate_delayer_applied_after_timeout() {
        let mut tk = Telekinesis::new_with_default_settings().unwrap();
        tk.vibrate_all(0.0);
        _assert_one_event(&mut tk);

        tk.vibrate_all_delayed(0.22, Duration::from_millis(50));
        _sleep(25);
        _assert_no_event(&mut tk);

        _sleep(50);
        _assert_one_event(&mut tk);
    }

    #[test]
    fn vibrate_delayed_command_is_overwritten() {
        let mut tk = Telekinesis::new_with_default_settings().unwrap();
        tk.vibrate_all_delayed(0.22, Duration::from_millis(50));
        tk.vibrate_all(0.33);
        _assert_one_event(&mut tk);
    }

    fn _sleep( milliseconds: u64 ) {
        thread::sleep(Duration::from_millis(milliseconds));
    }

    fn _assert_one_event( tk: &mut Telekinesis) {
        _sleep(10);
        assert!( tk.get_next_event().is_some() );
        assert!( tk.get_next_event().is_none() );
    }

    fn _assert_no_event( tk: &mut Telekinesis) {
        _sleep(10);
        assert!( tk.get_next_event().is_none() );
    }

}
