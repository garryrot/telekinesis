#[cfg(test)]
mod tests {
    use crate::telekinesis::in_process_connector;
    use crate::*;
    use crate::util::assert_timeout;
    use std::thread;
    use std::time::Duration;
    use std::time::Instant;

    #[test]
    fn connect_and_scan() {
        assert_eq!(tk_connect_with_settings(None), true);
        tk_scan_for_devices();
    }

    #[test]
    fn vibrate_delayer_applied_after_timeout() {
        let mut tk = Telekinesis::connect_with(|| async move { in_process_connector() }, None).unwrap();
        tk.vibrate_all(Speed::new(0), Duration::from_millis(50));
        _assert_one_event(&mut tk);
        _assert_no_event(&mut tk);
        _sleep();
        _assert_one_event(&mut tk);
    }

    #[test]
    fn vibrate_delayed_command_is_overwritten() {
        let mut tk = Telekinesis::connect_with(|| async move { in_process_connector() }, None).unwrap();
        tk.vibrate_all(Speed::new(33), Duration::from_millis(50));
        _assert_one_event(&mut tk)
    }

    #[test]
    fn get_next_events_empty_when_nothing_happens() {
        let mut tk = Telekinesis::connect_with(|| async move { in_process_connector() }, None).unwrap();
        _sleep();
        assert_eq!(tk.get_next_events().len(), 0);
    }

    #[test]
    fn get_next_events_after_action_returns_1() {
        let mut tk = Telekinesis::connect_with(|| async move { in_process_connector() }, None).unwrap();
        _sleep();
        tk.vibrate_all(Speed::new(22), Duration::from_secs(10));
        _sleep();
        assert_eq!(tk.get_next_events().len(), 1);
    }

    #[test]
    fn get_next_events_multiple_actions_are_returned_in_correct_order() {
        let mut tk = Telekinesis::connect_with(|| async move { in_process_connector() }, None).unwrap();
        _sleep();
        tk.vibrate_all(Speed::new(20), Duration::from_secs(10));
        tk.stop_all();
        _sleep();
        let events = tk.get_next_events();
        assert!(events[0].to_string().starts_with("Vibrating"));
        assert!(events[1].to_string().starts_with("Stopping"));
    }

    #[test]
    fn get_next_events_over_128_actions_respects_papyrus_limits_and_does_not_return_more_than_128_events(
    ) {
        let mut tk = Telekinesis::connect_with(|| async move { in_process_connector() }, None).unwrap();
        _sleep();
        for _ in 1..200 {
            tk.stop_all();
        }
        _sleep();
        assert_eq!(tk.get_next_events().len(), 128);
    }

    fn _sleep() {
        thread::sleep(Duration::from_millis(250));
    }

    fn _assert_one_event(tk: &mut Telekinesis) {
        assert_timeout!(tk.get_next_event().is_some(), "Exactly one element exists");
        assert!(tk.get_next_event().is_none());
    }

    fn _assert_no_event(tk: &mut Telekinesis) {
        assert!(tk.get_next_event().is_none());
    }

    #[test]
    fn speed_correct_conversion() {
        assert_eq!(Speed::new(-1000).as_float(), 0.0);
        assert_eq!(Speed::new(0).as_float(), 0.0);
        assert_eq!(Speed::new(9).as_float(), 0.09);
        assert_eq!(Speed::new(100).as_float(), 1.0);
        assert_eq!(Speed::new(1000).as_float(), 1.0);
    }
}
