#[cfg(test)]
mod tests {
    use tracing::Level;
    use crate::logging::{tk_init_logging, LogLevel, tk_init_logging_stdout};
    use crate::*;
    use std::ptr::{null};
    use std::thread;
    use std::time::Duration;
        
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
    fn init_logging_handles_nullpointr_without_panic() {
        tk_init_logging(LogLevel::Trace, null());
    }

    #[test]
    fn connect_and_scan() {
        assert_eq!( tk_connect(), true);
        tk_scan_for_devices();
    }

    #[test]
    fn some_test() {
        tk_init_logging_stdout(LogLevel::Trace);
        tk_connect_and_scan();
        _sleep(500);
        tk_vibrate_all(0);
        _sleep(500);
        let m = tk_try_get_next_event();
        assert!( !m.is_null() );
        _sleep(500);
        tk_close();
    }

    #[test]
    fn vibrate_delayer_applied_after_timeout() {
        let mut tk = Telekinesis::connect_with(telekinesis::in_process_server()).unwrap();
        _sleep(200);

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
        let mut tk = Telekinesis::connect_with(telekinesis::in_process_server()).unwrap();
        _sleep(200);

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
