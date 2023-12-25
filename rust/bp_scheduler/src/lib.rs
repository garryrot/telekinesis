use actuator::Actuator;
use buttplug::client::ButtplugClientError;
use player::PatternPlayer;
use std::collections::HashMap;
use worker::{ButtplugWorker, WorkerTask};

use std::{sync::Arc, time::Duration};
use tokio::{
    sync::mpsc::{unbounded_channel, UnboundedSender},
    time::sleep,
};
use tokio_util::sync::CancellationToken;
use tracing::error;

mod access;
pub mod actuator;
mod player;
pub mod speed;
mod worker;

pub struct ButtplugScheduler {
    worker_task_sender: UnboundedSender<WorkerTask>,
    settings: PlayerSettings,
    cancellation_tokens: HashMap<i32, CancellationToken>,
    last_handle: i32,
}

pub struct PlayerSettings {
    pub player_scalar_resolution_ms: i32,
}

impl ButtplugScheduler {
    pub fn create(settings: PlayerSettings) -> (ButtplugScheduler, ButtplugWorker) {
        let (worker_task_sender, task_receiver) = unbounded_channel::<WorkerTask>();
        (
            ButtplugScheduler {
                worker_task_sender,
                settings,
                cancellation_tokens: HashMap::new(),
                last_handle: 0,
            },
            ButtplugWorker { task_receiver },
        )
    }

    fn get_next_handle(&mut self) -> i32 {
        self.last_handle += 1;
        self.last_handle
    }

    pub fn stop_task(&mut self, handle: i32) {
        if self.cancellation_tokens.contains_key(&handle) {
            self.cancellation_tokens.remove(&handle).unwrap().cancel();
        } else {
            error!("Unknown handle {}", handle);
        }
    }

    pub fn create_player(&mut self, actuators: Vec<Arc<Actuator>>) -> PatternPlayer {
        let cancellation_token = CancellationToken::new();
        let handle = self.get_next_handle();
        self.cancellation_tokens
            .insert(handle, cancellation_token.clone());
        let (result_sender, result_receiver) =
            unbounded_channel::<Result<(), ButtplugClientError>>();
        PatternPlayer {
            actuators,
            result_sender,
            result_receiver,
            handle,
            cancellation_token,
            action_sender: self.worker_task_sender.clone(),
            player_scalar_resolution_ms: self.settings.player_scalar_resolution_ms,
        }
    }

    pub fn stop_all(&mut self) {
        let queue_full_err = "Event sender full";
        self.worker_task_sender
            .send(WorkerTask::StopAll)
            .unwrap_or_else(|_| error!(queue_full_err));
        for entry in self.cancellation_tokens.drain() {
            entry.1.cancel();
        }
    }
}

async fn cancellable_wait(duration: Duration, cancel: &CancellationToken) -> bool {
    tokio::select! {
        _ = cancel.cancelled() => {
            false
        }
        _ = sleep(duration) => {
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::actuator::get_actuators;
    use crate::speed::Speed;
    use bp_fakes::get_test_client;
    use bp_fakes::FakeMessage;
    use bp_fakes::*;
    use std::sync::Arc;
    use std::thread;
    use std::time::{Duration, Instant};

    use funscript::{FSPoint, FScript};
    use futures::future::join_all;

    use buttplug::client::ButtplugClientDevice;
    use buttplug::core::message::ActuatorType;

    use tokio::runtime::Handle;
    use tokio::task::JoinHandle;
    use tokio::time::timeout;

    use super::{Actuator, ButtplugScheduler, PlayerSettings};

    struct PlayerTest {
        pub scheduler: ButtplugScheduler,
        pub handles: Vec<JoinHandle<()>>,
        pub all_devices: Vec<Arc<ButtplugClientDevice>>,
    }

    impl PlayerTest {
        fn setup(all_devices: &[Arc<ButtplugClientDevice>]) -> Self {
            PlayerTest::setup_with_settings(
                all_devices,
                PlayerSettings {
                    player_scalar_resolution_ms: 1,
                },
            )
        }

        fn setup_with_settings(
            all_devices: &[Arc<ButtplugClientDevice>],
            settings: PlayerSettings,
        ) -> Self {
            let (scheduler, mut worker) = ButtplugScheduler::create(settings);
            Handle::current().spawn(async move {
                worker.run_worker_thread().await;
            });
            PlayerTest {
                scheduler,
                handles: vec![],
                all_devices: all_devices.to_owned(),
            }
        }

        async fn play_scalar_pattern(
            &mut self,
            duration: Duration,
            fscript: FScript,
            speed: Speed,
            actuators: Option<Vec<Arc<Actuator>>>,
        ) {
            let actuators = match actuators {
                Some(actuators) => actuators,
                None => get_actuators(self.all_devices.clone()),
            };
            let player: super::PatternPlayer = self.scheduler.create_player(actuators);
            player
                .play_scalar_pattern(duration, fscript, speed)
                .await
                .unwrap();
        }

        fn play_scalar(
            &mut self,
            duration: Duration,
            speed: Speed,
            actuators: Option<Vec<Arc<Actuator>>>,
        ) {
            let actuators = match actuators {
                Some(actuators) => actuators,
                None => get_actuators(self.all_devices.clone()),
            };
            let player = self.scheduler.create_player(actuators);
            self.handles.push(Handle::current().spawn(async move {
                player.play_scalar(duration, speed).await.unwrap();
            }));
        }

        async fn play_linear(&mut self, funscript: FScript, duration: Duration) {
            let player = self
                .scheduler
                .create_player(get_actuators(self.all_devices.clone()));
            player.play_linear(duration, funscript).await.unwrap();
        }

        async fn await_last(&mut self) {
            let _ = self.handles.pop().unwrap().await;
        }

        async fn await_all(self) {
            join_all(self.handles).await;
        }
    }

    /// Linear
    #[tokio::test]
    async fn test_no_devices_does_not_block() {
        // arrange
        let client = get_test_client(vec![]).await;
        let mut player = PlayerTest::setup(&client.created_devices);

        let mut fs: FScript = FScript::default();
        fs.actions.push(FSPoint { pos: 1, at: 10 });
        fs.actions.push(FSPoint { pos: 2, at: 20 });

        // act & assert
        player.play_scalar(Duration::from_millis(50), Speed::max(), None);
        assert!(
            timeout(Duration::from_secs(1), player.await_last(),)
                .await
                .is_ok(),
            "Scalar finishes within timeout"
        );
        assert!(
            timeout(
                Duration::from_secs(1),
                player.play_linear(fs, Duration::from_millis(50)),
            )
            .await
            .is_ok(),
            "Linear finishes within timeout"
        );
    }

    #[tokio::test]
    async fn test_linear_empty_pattern_finishes_and_does_not_panic() {
        let client = get_test_client(vec![linear(1, "lin1")]).await;
        let mut player = PlayerTest::setup(&client.created_devices);

        // act & assert
        player
            .play_linear(FScript::default(), Duration::from_millis(1))
            .await;

        let mut fs = FScript::default();
        fs.actions.push(FSPoint { pos: 0, at: 0 });
        fs.actions.push(FSPoint { pos: 0, at: 0 });
        player
            .play_linear(FScript::default(), Duration::from_millis(1))
            .await;
    }

    #[tokio::test]
    async fn test_linear_pattern() {
        // arrange
        let client = get_test_client(vec![linear(1, "lin1")]).await;
        let mut player = PlayerTest::setup(&client.created_devices);

        let mut fscript = FScript::default();
        fscript.actions.push(FSPoint { pos: 50, at: 0 }); // zero_action_is_ignored
        fscript.actions.push(FSPoint { pos: 0, at: 200 });
        fscript.actions.push(FSPoint { pos: 100, at: 400 });

        // act
        let start = Instant::now();
        let duration = get_duration_ms(&fscript);
        player.play_linear(fscript, duration).await;

        // assert
        client.print_device_calls(start);
        client.get_device_calls(1)[0]
            .assert_position(0.0)
            .assert_duration(200)
            .assert_timestamp(0, start);
        client.get_device_calls(1)[1]
            .assert_position(1.0)
            .assert_duration(200)
            .assert_timestamp(200, start);
    }

    #[tokio::test]
    async fn test_linear_timing_remains_synced_with_clock() {
        // arrange
        let n = 40;
        let client = get_test_client(vec![linear(1, "lin1")]).await;
        let mut player = PlayerTest::setup(&client.created_devices);
        let fscript = get_repeated_pattern(n);

        // act
        let start = Instant::now();
        player
            .play_linear(get_repeated_pattern(n), get_duration_ms(&fscript))
            .await;

        // assert
        client.print_device_calls(start);
        check_timing(client.get_device_calls(1), n, start);
    }

    #[tokio::test]
    async fn test_linear_repeats_until_duration_ends() {
        // arrange
        let client = get_test_client(vec![linear(1, "lin1")]).await;
        let mut player = PlayerTest::setup(&client.created_devices);

        let mut fscript = FScript::default();
        fscript.actions.push(FSPoint { pos: 100, at: 200 });
        fscript.actions.push(FSPoint { pos: 0, at: 400 });

        // act
        let start = Instant::now();
        let duration = Duration::from_millis(800);
        player.play_linear(fscript, duration).await;

        // assert
        client.print_device_calls(start);

        let calls = client.get_device_calls(1);
        calls[0].assert_position(1.0).assert_timestamp(0, start);
        calls[1].assert_position(0.0).assert_timestamp(200, start);
        calls[2].assert_position(1.0).assert_timestamp(400, start);
        calls[3].assert_position(0.0).assert_timestamp(600, start);
    }

    #[tokio::test]
    async fn test_linear_cancels_after_duration() {
        // arrange
        let client = get_test_client(vec![linear(1, "lin1")]).await;
        let mut player = PlayerTest::setup(&client.created_devices);

        let mut fscript = FScript::default();
        fscript.actions.push(FSPoint { pos: 0, at: 400 });
        fscript.actions.push(FSPoint { pos: 0, at: 800 });

        // act
        let start = Instant::now();
        let duration = Duration::from_millis(400);
        player.play_linear(fscript, duration).await;

        // assert
        client.print_device_calls(start);
        client.get_device_calls(1)[0]
            .assert_position(0.0)
            .assert_duration(400);
        assert!(
            start.elapsed().as_millis() < 425,
            "Stops after duration ends"
        );
    }

    /// Scalar
    #[tokio::test]
    async fn test_scalar_empty_pattern_finishes_and_does_not_panic() {
        // arrange
        let client = get_test_client(vec![scalar(1, "vib1", ActuatorType::Vibrate)]).await;
        let mut player = PlayerTest::setup(&client.created_devices);

        // act & assert
        let duration = Duration::from_millis(1);
        let fscript = FScript::default();
        player
            .play_scalar_pattern(duration, fscript, Speed::max(), None)
            .await;

        let mut fscript = FScript::default();
        fscript.actions.push(FSPoint { pos: 0, at: 0 });
        fscript.actions.push(FSPoint { pos: 0, at: 0 });
        player
            .play_scalar_pattern(Duration::from_millis(200), fscript, Speed::max(), None)
            .await;
    }

    #[tokio::test]
    async fn test_scalar_pattern_actuator_selection() {
        // arrange
        let client = get_test_client(vec![scalars(1, "vib1", ActuatorType::Vibrate, 2)]).await;
        let mut player = PlayerTest::setup(&client.created_devices);
        let actuators = get_actuators(client.created_devices.clone());

        // act
        let start = Instant::now();

        let mut fs1 = FScript::default();
        fs1.actions.push(FSPoint { pos: 10, at: 0 });
        fs1.actions.push(FSPoint { pos: 20, at: 100 });
        player
            .play_scalar_pattern(
                Duration::from_millis(125),
                fs1,
                Speed::max(),
                Some(vec![actuators[1].clone()]),
            )
            .await;

        let mut fs2 = FScript::default();
        fs2.actions.push(FSPoint { pos: 30, at: 0 });
        fs2.actions.push(FSPoint { pos: 40, at: 100 });
        player
            .play_scalar_pattern(
                Duration::from_millis(125),
                fs2,
                Speed::max(),
                Some(vec![actuators[0].clone()]),
            )
            .await;

        // assert
        client.print_device_calls(start);
        let calls = client.get_device_calls(1);
        calls[0].assert_strengths(vec![(1, 0.1)]);
        calls[1].assert_strengths(vec![(1, 0.2)]);
        calls[4].assert_strengths(vec![(0, 0.3)]);
        calls[5].assert_strengths(vec![(0, 0.4)]);
    }

    #[tokio::test]
    async fn test_scalar_pattern_repeats_until_duration_ends() {
        // arrange
        let client = get_test_client(vec![scalar(1, "vib1", ActuatorType::Vibrate)]).await;
        let mut player = PlayerTest::setup(&client.created_devices);

        // act
        let mut fs = FScript::default();
        fs.actions.push(FSPoint { pos: 100, at: 0 });
        fs.actions.push(FSPoint { pos: 50, at: 50 });
        fs.actions.push(FSPoint { pos: 70, at: 100 });

        let start = Instant::now();
        player
            .play_scalar_pattern(Duration::from_millis(125), fs, Speed::max(), None)
            .await;

        // assert
        client.print_device_calls(start);
        let calls = client.get_device_calls(1);
        calls[0].assert_strenth(1.0);
        calls[1].assert_strenth(0.5);
        calls[2].assert_strenth(0.7);
        calls[3].assert_strenth(1.0);
        calls[4].assert_strenth(0.0).assert_timestamp(125, start);
        assert_eq!(calls.len(), 5)
    }

    #[tokio::test]
    async fn test_scalar_timing_remains_synced_with_clock() {
        // arrange
        let n = 40;
        let client = get_test_client(vec![scalar(1, "vib1", ActuatorType::Vibrate)]).await;
        let mut player = PlayerTest::setup(&client.created_devices);
        let fscript = get_repeated_pattern(n);

        // act
        let start = Instant::now();
        player
            .play_scalar_pattern(get_duration_ms(&fscript), fscript, Speed::max(), None)
            .await;

        // assert
        client.print_device_calls(start);
        check_timing(client.get_device_calls(1), n, start);
    }

    #[tokio::test]
    async fn test_scalar_points_below_min_resolution() {
        // arrange
        let client = get_test_client(vec![scalar(1, "vib1", ActuatorType::Vibrate)]).await;
        let mut player = PlayerTest::setup_with_settings(
            &client.created_devices,
            PlayerSettings {
                player_scalar_resolution_ms: 100,
            },
        );

        let mut fs = FScript::default();
        fs.actions.push(FSPoint { pos: 42, at: 0 });
        fs.actions.push(FSPoint { pos: 1, at: 1 });
        fs.actions.push(FSPoint { pos: 1, at: 99 });
        fs.actions.push(FSPoint { pos: 42, at: 100 });

        // act
        let start = Instant::now();
        player
            .play_scalar_pattern(Duration::from_millis(150), fs, Speed::max(), None)
            .await;

        // assert
        client.print_device_calls(start);
        let calls = client.get_device_calls(1);
        calls[0].assert_strenth(0.42).assert_timestamp(0, start);
        calls[1].assert_strenth(0.42).assert_timestamp(100, start);
    }

    #[tokio::test]
    async fn test_scalar_speed_control() {
        // arrange
        let client = get_test_client(vec![scalar(1, "vib1", ActuatorType::Vibrate)]).await;
        let mut player = PlayerTest::setup(&client.created_devices);

        let mut fs = FScript::default();
        fs.actions.push(FSPoint { pos: 100, at: 0 });
        fs.actions.push(FSPoint { pos: 70, at: 25 });
        fs.actions.push(FSPoint { pos: 0, at: 50 });

        // act
        let start = Instant::now();
        player
            .play_scalar_pattern(Duration::from_millis(50), fs, Speed::new(10), None)
            .await;

        // assert
        client.print_device_calls(start);
        let calls = client.get_device_calls(1);
        calls[0].assert_strenth(0.1);
        calls[1].assert_strenth(0.07);
        calls[2].assert_strenth(0.0);
    }

    // Concurrency Tests

    #[tokio::test]
    async fn test_concurrent_linear_access_2_threads() {
        // call1  |111111111111111111111-->|
        // call2         |2222->|
        // result |111111122222211111111-->|

        // arrange
        let client = get_test_client(vec![scalar(1, "vib1", ActuatorType::Vibrate)]).await;
        let mut player = PlayerTest::setup(&client.created_devices);

        // act
        let start = Instant::now();

        player.play_scalar(Duration::from_millis(500), Speed::new(50), None);
        wait_ms(100).await;
        player.play_scalar(Duration::from_millis(100), Speed::new(100), None);
        player.await_all().await;

        // assert
        client.print_device_calls(start);
        client.get_device_calls(1)[0].assert_strenth(0.5);
        client.get_device_calls(1)[1].assert_strenth(1.0);
        client.get_device_calls(1)[2].assert_strenth(0.5);
        client.get_device_calls(1)[3].assert_strenth(0.0);
        assert_eq!(client.call_registry.get_device(1).len(), 4);
    }

    #[tokio::test]
    async fn test_concurrent_linear_access_3_threads() {
        // call1  |111111111111111111111111111-->|
        // call2       |22222222222222->|
        // call3            |333->|
        // result |111122222333332222222111111-->|

        // arrange
        let client = get_test_client(vec![scalar(1, "vib1", ActuatorType::Vibrate)]).await;
        let mut player = PlayerTest::setup(&client.created_devices);

        // act
        let start = Instant::now();
        player.play_scalar(Duration::from_secs(3), Speed::new(20), None);
        wait_ms(250).await;

        player.play_scalar(Duration::from_secs(2), Speed::new(40), None);
        wait_ms(250).await;

        player.play_scalar(Duration::from_secs(1), Speed::new(80), None);
        player.await_all().await;

        // assert
        client.print_device_calls(start);

        client.get_device_calls(1)[0].assert_strenth(0.2);
        client.get_device_calls(1)[1].assert_strenth(0.4);
        client.get_device_calls(1)[2].assert_strenth(0.8);
        client.get_device_calls(1)[3].assert_strenth(0.4);
        client.get_device_calls(1)[4].assert_strenth(0.2);
        client.get_device_calls(1)[5].assert_strenth(0.0);
        assert_eq!(client.call_registry.get_device(1).len(), 6);
    }

    #[tokio::test]
    async fn test_concurrent_linear_access_3_threads_2() {
        // call1  |111111111111111111111111111-->|
        // call2       |22222222222->|
        // call3            |333333333-->|
        // result |111122222222222233333331111-->|

        // arrange
        let client = get_test_client(vec![scalar(1, "vib1", ActuatorType::Vibrate)]).await;
        let mut player = PlayerTest::setup(&client.created_devices);

        // act
        let start = Instant::now();
        player.play_scalar(Duration::from_secs(3), Speed::new(20), None);
        wait_ms(250).await;

        player.play_scalar(Duration::from_secs(1), Speed::new(40), None);
        wait_ms(250).await;

        player.play_scalar(Duration::from_secs(1), Speed::new(80), None);
        player.await_last().await;
        thread::sleep(Duration::from_secs(2));
        player.await_all().await;

        // assert
        client.print_device_calls(start);
        client.get_device_calls(1)[0].assert_strenth(0.2);
        client.get_device_calls(1)[1].assert_strenth(0.4);
        client.get_device_calls(1)[2].assert_strenth(0.8);
        client.get_device_calls(1)[3].assert_strenth(0.8);
        client.get_device_calls(1)[4].assert_strenth(0.2);
        client.get_device_calls(1)[5].assert_strenth(0.0);
        assert_eq!(client.call_registry.get_device(1).len(), 6);
    }

    #[tokio::test]
    async fn test_concurrency_linear_and_pattern() {
        // lin1   |11111111111111111-->|
        // pat1       |23452345234523452345234-->|
        // result |1111111111111111111123452345234-->|

        // arrange
        let client = get_test_client(vec![scalar(1, "vib1", ActuatorType::Vibrate)]).await;
        let mut player = PlayerTest::setup(&client.created_devices);

        // act
        let mut fscript = FScript::default();
        for i in 0..10 {
            fscript.actions.push(FSPoint {
                pos: 10 * i,
                at: 100 * i,
            });
        }

        let start = Instant::now();
        player.play_scalar(Duration::from_secs(1), Speed::new(99), None);
        wait_ms(250).await;
        player
            .play_scalar_pattern(Duration::from_secs(3), fscript, Speed::max(), None)
            .await;

        // assert
        client.print_device_calls(start);
        assert!(client.call_registry.get_device(1).len() > 3);
    }

    #[tokio::test]
    async fn test_concurrency_two_devices_simulatenously_both_are_started_and_stopped() {
        let client = get_test_client(vec![
            scalar(1, "vib1", ActuatorType::Vibrate),
            scalar(2, "vib2", ActuatorType::Vibrate),
        ])
        .await;
        let mut player = PlayerTest::setup(&client.created_devices);

        // act
        let start = Instant::now();
        player.play_scalar(
            Duration::from_millis(300),
            Speed::new(99),
            Some(get_actuators(vec![client.get_device(1)])),
        );
        player.play_scalar(
            Duration::from_millis(200),
            Speed::new(88),
            Some(get_actuators(vec![client.get_device(2)])),
        );

        player.await_all().await;

        // assert
        client.print_device_calls(start);
        client.get_device_calls(1)[0].assert_strenth(0.99);
        client.get_device_calls(1)[1].assert_strenth(0.0);
        client.get_device_calls(2)[0].assert_strenth(0.88);
        client.get_device_calls(2)[1].assert_strenth(0.0);
    }

    async fn wait_ms(ms: u64) {
        tokio::time::sleep(Duration::from_millis(ms)).await;
    }

    fn get_duration_ms(fs: &FScript) -> Duration {
        Duration::from_millis(fs.actions.last().unwrap().at as u64)
    }

    fn check_timing(device_calls: Vec<FakeMessage>, n: usize, start: Instant) {
        for i in 0..n - 1 {
            device_calls[i].assert_timestamp((i * 100) as i32, start);
        }
    }

    fn get_repeated_pattern(n: usize) -> FScript {
        let mut fscript = FScript::default();
        for i in 0..n {
            fscript.actions.push(FSPoint {
                pos: (i % 100) as i32,
                at: (i * 100) as i32,
            });
        }
        fscript
    }

    // Utils

    #[test]
    fn speed_correct_conversion() {
        assert_eq!(Speed::new(-1000).as_float(), 0.0);
        assert_eq!(Speed::new(0).as_float(), 0.0);
        assert_eq!(Speed::new(9).as_float(), 0.09);
        assert_eq!(Speed::new(100).as_float(), 1.0);
        assert_eq!(Speed::new(1000).as_float(), 1.0);
    }
}
