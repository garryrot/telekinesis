#[cfg(test)]
mod tests {
    use crate::telekinesis::in_process_connector;
    use crate::*;
    use crate::util::assert_timeout;
    use std::thread;
    use std::time::Duration;
    use std::time::Instant;

    #[test]
    fn vibrate_delayed_command_is_overwritten() {
        let mut tk = Telekinesis::connect_with(|| async move { in_process_connector() }, None).unwrap();
        tk.vibrate(Speed::new(33), TkDuration::from_millis(50), vec![]);
        _assert_one_event(&mut tk)
    }

    #[test]
    fn process_next_events_empty_when_nothing_happens() {
        let mut tk = Telekinesis::connect_with(|| async move { in_process_connector() }, None).unwrap();
        _sleep();
        assert_eq!(tk.process_next_events().len(), 0);
    }

    #[test]
    fn process_next_events_after_action_returns_1() {
        let mut tk = Telekinesis::connect_with(|| async move { in_process_connector() }, None).unwrap();
        _sleep();
        tk.vibrate(Speed::new(22), TkDuration::from_millis(1), vec![]);
        _sleep();
        assert_eq!(tk.process_next_events().len(), 1);
    }

    #[test]
    fn process_next_events_works() {
        let mut tk = Telekinesis::connect_with(|| async move { in_process_connector() }, None).unwrap();
        _sleep();
        tk.vibrate(Speed::new(10), TkDuration::from_millis(100), vec![]);
        tk.vibrate(Speed::new(20), TkDuration::from_millis(200), vec![]);
        _sleep();
        _sleep();
        let events = tk.process_next_events();
        assert!(events[0].serialize_papyrus().starts_with("DeviceEvent|Vibrator|0.1"));
        assert!(events[1].serialize_papyrus().starts_with("DeviceEvent|Vibrator|0.2"));
    }

    #[test]
    fn process_next_events_over_128_actions_respects_papyrus_limits_and_does_not_return_more_than_128_events(
    ) {
        let mut tk = Telekinesis::connect_with(|| async move { in_process_connector() }, None).unwrap();
        _sleep();
        for _ in 1..200 {
            tk.vibrate(Speed::min(), TkDuration::Timed(Duration::from_micros(1)), vec![]);
        }
        _sleep();
        assert_eq!(tk.process_next_events().len(), 128);
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
