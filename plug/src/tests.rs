#[cfg(test)]
mod tests {
    use tracing::{Level};
    use crate::*;
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
    fn connect_and_scan() {
        assert_eq!(tk_connect(), true);
        tk_scan_for_devices();
    }

    #[test]
    fn vibrate_delayer_applied_after_timeout() {
        let mut tk = Telekinesis::connect_with(telekinesis::in_process_server()).unwrap();
        _sleep(200);

        tk.vibrate_all(Speed::new(0));
        _assert_one_event(&mut tk);

        tk.vibrate_all_delayed(Speed::new(22), Duration::from_millis(50));
        _sleep(25);
        _assert_no_event(&mut tk);

        _sleep(50);
        _assert_one_event(&mut tk);
    }

    #[test]
    fn vibrate_delayed_command_is_overwritten() {
        let mut tk = Telekinesis::connect_with(telekinesis::in_process_server()).unwrap();
        _sleep(200);

        tk.vibrate_all_delayed(Speed::new(22), Duration::from_millis(50));
        tk.vibrate_all(Speed::new(33));
        _assert_one_event(&mut tk);
    }

    #[test]
    fn get_next_events_empty_when_nothing_happens() 
    {
        let mut tk = Telekinesis::connect_with(telekinesis::in_process_server()).unwrap();
        _sleep(200); // TODO: Finally needs a mocked version

        assert_eq!( tk.get_next_events().len(), 0);
    }

    #[test]
    fn get_next_events_after_action_returns_1() 
    {
        let mut tk = Telekinesis::connect_with(telekinesis::in_process_server()).unwrap();
        _sleep(200);

        tk.vibrate_all(Speed::new(22));
        _sleep(200);

        assert_eq!( tk.get_next_events().len(), 1);
    }

    #[test]
    fn get_next_events_multiple_actions_are_returned_in_correct_order() 
    {
        let mut tk = Telekinesis::connect_with(telekinesis::in_process_server()).unwrap();
        _sleep(200); 

        tk.vibrate_all(Speed::new(20));
        tk.stop_all();

        _sleep(200);

        let events = tk.get_next_events();
        assert!( events[0].to_string().starts_with("Vibrating") );
        assert!( events[1].to_string().starts_with("Stopping") );
    }

    #[test]
    fn get_next_events_over_128_actions_respects_papyrus_limits_and_does_not_return_more_than_128_events() 
    {
        let mut tk = Telekinesis::connect_with(telekinesis::in_process_server()).unwrap();
        _sleep(200);
        for _ in 1..200 {
            tk.stop_all();
        }
        _sleep(200);
        assert_eq!( tk.get_next_events().len(), 128 );
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

    #[test]
    fn speed_correct_conversion() {
        assert_eq!(Speed::new(-1000).as_0_to_1_f64(), 0.0);
        assert_eq!(Speed::new(0).as_0_to_1_f64(), 0.0);
        assert_eq!(Speed::new(9).as_0_to_1_f64(), 0.09);
        assert_eq!(Speed::new(100).as_0_to_1_f64(), 1.0);
        assert_eq!(Speed::new(1000).as_0_to_1_f64(), 1.0);
    }
}
