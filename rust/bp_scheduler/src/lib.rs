use actuator::Actuator;
use buttplug::client::ButtplugClientError;
use player::PatternPlayer;
use settings::ActuatorSettings;
use speed::Speed;
use std::collections::HashMap;
use worker::{ButtplugWorker, WorkerTask};

use std::{sync::Arc, time::Duration};
use tokio::{
    sync::mpsc::{unbounded_channel, UnboundedSender},
    time::sleep,
};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error};

mod access;
pub mod actuator;
pub mod player;
pub mod speed;
pub mod settings;
mod worker;

#[derive(Debug)]
pub struct ButtplugScheduler {
    worker_task_sender: UnboundedSender<WorkerTask>,
    settings: PlayerSettings,
    control_handles: HashMap<i32, ControlHandle>,
    last_handle: i32,
}

#[derive(Debug)]
struct ControlHandle {
    cancellation_token: CancellationToken,
    update_sender: UnboundedSender<Speed>,
}

#[derive(Debug)]
pub struct PlayerSettings {
    pub scalar_resolution_ms: i32,
}

impl ButtplugScheduler {
    pub fn create(settings: PlayerSettings) -> (ButtplugScheduler, ButtplugWorker) {
        let (worker_task_sender, task_receiver) = unbounded_channel::<WorkerTask>();
        (
            ButtplugScheduler {
                worker_task_sender,
                settings,
                control_handles: HashMap::new(),
                last_handle: 0,
            },
            ButtplugWorker { task_receiver },
        )
    }

    fn get_next_handle(&mut self) -> i32 {
        self.last_handle += 1;
        self.last_handle
    }

    /// Clean up finished tasks
    pub fn clean_finished_tasks(&mut self) {
        self.control_handles
            .retain(|_, handle| !handle.cancellation_token.is_cancelled());
    }

    pub fn stop_task(&mut self, handle: i32) {
        if self.control_handles.contains_key(&handle) {
            debug!("stop handle {}", handle);
            self.control_handles
                .remove(&handle)
                .unwrap()
                .cancellation_token
                .cancel();
        } else {
            error!("Unknown handle {}", handle);
        }
    }

    pub fn update_task(&mut self, handle: i32, speed: Speed) -> bool {
        if self.control_handles.contains_key(&handle) {
            debug!("updating handle {}", handle);
            let _ = self
                .control_handles
                .get(&handle)
                .unwrap()
                .update_sender
                .send(speed);
            true
        } else {
            error!("Unknown handle {}", handle);
            false
        }
    }

    pub fn stop_all(&mut self) {
        let queue_full_err = "Event sender full";
        self.worker_task_sender
            .send(WorkerTask::StopAll)
            .unwrap_or_else(|_| error!(queue_full_err));
        for entry in self.control_handles.drain() {
            debug!("stop-all - stopping handle {:?}", entry.0);
            entry.1.cancellation_token.cancel();
        }
        self.control_handles.clear();
    }

    pub fn create_player(&mut self, actuators: Vec<Arc<Actuator>>) -> PatternPlayer {
        let empty_settings = actuators.iter().map(|_| ActuatorSettings::None).collect::<Vec<ActuatorSettings>>();
        self.create_player_with_settings(actuators, empty_settings)
    }

    pub fn create_player_with_settings(&mut self, actuators: Vec<Arc<Actuator>>, settings: Vec<ActuatorSettings>) -> PatternPlayer {
        let (update_sender, update_receiver) = unbounded_channel::<Speed>();

        let cancellation_token = CancellationToken::new();
        let handle = self.get_next_handle();
        self.control_handles.insert(
            handle,
            ControlHandle {
                cancellation_token: cancellation_token.clone(),
                update_sender,
            },
        );

        let (result_sender, result_receiver) =
            unbounded_channel::<Result<(), ButtplugClientError>>();
        PatternPlayer {
            actuators,
            settings,
            result_sender,
            result_receiver,
            update_receiver,
            handle,
            cancellation_token,
            worker_task_sender: self.worker_task_sender.clone(),
            scalar_resolution_ms: self.settings.scalar_resolution_ms,
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
    use crate::player::PatternPlayer;
    use crate::settings::ActuatorSettings;
    use crate::settings::LinearRange;
    use crate::speed::Speed;
    use bp_fakes::get_test_client;
    use bp_fakes::FakeMessage;
    use bp_fakes::*;
    use std::sync::Arc;
    use std::thread;
    use std::time::{Duration, Instant};
    use tracing::Level;

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
                    scalar_resolution_ms: 1,
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

        fn get_player(&mut self) -> PatternPlayer {
            self.scheduler
                .create_player(get_actuators(self.all_devices.clone()))
        }

        fn get_player_with_settings(&mut self, settings: Vec<ActuatorSettings>) -> PatternPlayer {
            self.scheduler.create_player_with_settings(get_actuators(self.all_devices.clone()), settings)
        }

        async fn play_linear(&mut self, funscript: FScript, duration: Duration, speed: Speed) {
            let player = self
                .scheduler
                .create_player(get_actuators(self.all_devices.clone()));
            player
                .play_linear(duration, funscript, speed)
                .await
                .unwrap();
        }

        fn play_linear_background(&mut self, funscript: FScript, duration: Duration, speed: Speed) {
            let player = self
                .scheduler
                .create_player(get_actuators(self.all_devices.clone()));
            self.handles.push(Handle::current().spawn(async move {
                player
                    .play_linear(duration, funscript, speed)
                    .await
                    .unwrap();
            }));
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
                player.play_linear(fs, Duration::from_millis(50), Speed::max()),
            )
            .await
            .is_ok(),
            "Linear finishes within timeout"
        );
    }

    #[tokio::test]
    async fn test_oscillate_linear_1() {
        let (client, _) = test_oscillate(
            Speed::new(100),
            LinearRange{ min_pos: 0.0, max_pos: 1.0, min_ms: 50, max_ms: 400, invert: false },
        )
        .await;

        let calls = client.get_device_calls(1);
        calls[0].assert_duration(50);
        calls[1].assert_duration(50);
        calls[2].assert_duration(50);
    }

    #[tokio::test]
    async fn test_oscillate_linear_2() {
        let (client, _) = test_oscillate(
            Speed::new(0),
            LinearRange{ min_pos: 1.0, max_pos: 0.0, min_ms: 10, max_ms: 100, invert: false }
        )
        .await;

        let calls = client.get_device_calls(1);
        calls[0].assert_duration(100).assert_pos(0.0);
        calls[1].assert_duration(100).assert_pos(1.0);
        calls[2].assert_duration(100).assert_pos(0.0);
    }

    #[tokio::test]
    async fn test_oscillate_linear_3() {
        let (client, _) = test_oscillate(
            Speed::new(75),
            LinearRange{ min_pos: 0.2, max_pos: 0.7, min_ms: 100, max_ms: 200, invert: false }
        )
        .await;

        let calls = client.get_device_calls(1);
        calls[0].assert_duration(125).assert_pos(0.7);
        calls[1].assert_duration(125).assert_pos(0.2);
        calls[2].assert_duration(125).assert_pos(0.7);
    }

    #[tokio::test]
    async fn test_oscillate_linear_invert() {
        let (client, _) = test_oscillate(
            Speed::new(100),
            LinearRange{ min_pos: 0.2, max_pos: 0.7, min_ms: 50, max_ms: 50, invert: true }
        )
        .await;

        let calls = client.get_device_calls(1);
        calls[0].assert_pos(0.3);
        calls[1].assert_pos(0.8);
        calls[2].assert_pos(0.3);
    }

    #[tokio::test]
    async fn test_oscillate_update() {
        let client: ButtplugTestClient = get_test_client(vec![linear(1, "lin1")]).await;
        let mut test = PlayerTest::setup(&client.created_devices);

        // act
        let start = Instant::now();
        let player = test.get_player();
        let join = Handle::current().spawn(async move {
            let _ = player
                .play_oscillate_linear(Duration::from_millis(250), Speed::new(100), LinearRange { 
                    min_pos: 0.0, 
                    max_pos: 1.0, 
                    min_ms: 10, 
                    max_ms: 100, 
                    invert: true })
                .await;
        });

        test.scheduler.update_task(1, Speed::new(0));
        let _ = join.await;

        client.print_device_calls(start);
        let calls = client.get_device_calls(1);
        calls[0].assert_duration(100);
        calls[1].assert_duration(100);
        calls[2].assert_duration(100);
    }

    async fn test_oscillate(speed: Speed, range: LinearRange) -> (ButtplugTestClient, Instant) {
        let client = get_test_client(vec![linear(1, "lin1")]).await;
        let mut test = PlayerTest::setup(&client.created_devices);

        // act
        let start = Instant::now();
        let duration_ms = range.max_ms as f64 * 2.5;
        let player = test.get_player_with_settings(vec![ ActuatorSettings::Linear(range)]);
        let _ = player
            .play_oscillate_linear(Duration::from_millis(duration_ms as u64), speed, LinearRange::max())
            .await;

        client.print_device_calls(start);
        (client, start)
    }

    #[tokio::test]
    async fn test_linear_empty_pattern_finishes_and_does_not_panic() {
        let client = get_test_client(vec![linear(1, "lin1")]).await;
        let mut player = PlayerTest::setup(&client.created_devices);

        // act & assert
        player
            .play_linear(FScript::default(), Duration::from_millis(1), Speed::max())
            .await;

        let mut fs = FScript::default();
        fs.actions.push(FSPoint { pos: 0, at: 0 });
        fs.actions.push(FSPoint { pos: 0, at: 0 });
        player
            .play_linear(FScript::default(), Duration::from_millis(1), Speed::max())
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
        player.play_linear(fscript, duration, Speed::max()).await;

        // assert
        client.print_device_calls(start);
        client.get_device_calls(1)[0]
            .assert_pos(0.0)
            .assert_duration(200)
            .assert_time(0, start);
        client.get_device_calls(1)[1]
            .assert_pos(1.0)
            .assert_duration(200)
            .assert_time(200, start);
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
            .play_linear(
                get_repeated_pattern(n),
                get_duration_ms(&fscript),
                Speed::max(),
            )
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
        player.play_linear(fscript, duration, Speed::max()).await;

        // assert
        client.print_device_calls(start);

        let calls = client.get_device_calls(1);
        calls[0].assert_pos(1.0).assert_time(0, start);
        calls[1].assert_pos(0.0).assert_time(200, start);
        calls[2].assert_pos(1.0).assert_time(400, start);
        calls[3].assert_pos(0.0).assert_time(600, start);
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
        player.play_linear(fscript, duration, Speed::max()).await;

        // assert
        client.print_device_calls(start);
        client.get_device_calls(1)[0]
            .assert_pos(0.0)
            .assert_duration(400);
        assert!(
            start.elapsed().as_millis() < 425,
            "Stops after duration ends"
        );
    }

    #[tokio::test]
    async fn test_linear_control() {
        // arrange
        let client = get_test_client(vec![linear(1, "lin1")]).await;
        let mut player = PlayerTest::setup(&client.created_devices);

        let mut fs = FScript::default();
        fs.actions.push(FSPoint { pos: 0, at: 0 });
        fs.actions.push(FSPoint { pos: 100, at: 25 });
        fs.actions.push(FSPoint { pos: 0, at: 50 });

        // act
        let start = Instant::now();
        player.play_linear_background(fs, Duration::from_secs(2), Speed::new(10));
        wait_ms(400).await;
        player.scheduler.update_task(1, Speed::new(50));
        player.await_all().await;

        // assert
        client.print_device_calls(start);
        let calls = client.get_device_calls(1);
        calls[0].assert_time(0, start);
        calls[1].assert_time(250, start);
        calls[2].assert_time(500, start);
        calls[3].assert_time(550, start);
        calls[4].assert_time(600, start);
    }

    #[tokio::test]
    async fn test_linear_speed_factors_above_50percent_work() {
        // arrange
        let client = get_test_client(vec![linear(1, "lin1")]).await;
        let mut player = PlayerTest::setup(&client.created_devices);

        let mut fs = FScript::default();
        fs.actions.push(FSPoint { pos: 0, at: 0 });
        fs.actions.push(FSPoint { pos: 100, at: 1000 });
        fs.actions.push(FSPoint { pos: 0, at: 2000 });

        // act
        let start = Instant::now();
        player.play_linear_background(fs, Duration::from_secs(4), Speed::new(80));
        player.await_all().await;

        // assert
        client.print_device_calls(start);
        let calls = client.get_device_calls(1);
        calls[0].assert_time(0, start);
        calls[1].assert_time(1250, start);
        calls[2].assert_time(2500, start);
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
        calls[4].assert_strenth(0.0).assert_time(125, start);
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
                scalar_resolution_ms: 100,
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
        calls[0].assert_strenth(0.42).assert_time(0, start);
        calls[1].assert_strenth(0.42).assert_time(100, start);
    }

    #[tokio::test]
    async fn test_scalar_pattern_control() {
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

    #[tokio::test]
    async fn test_scalar_constant_control() {
        // arrange
        let client = get_test_client(vec![scalar(1, "vib1", ActuatorType::Vibrate)]).await;
        let mut player = PlayerTest::setup(&client.created_devices);

        // act
        let start = Instant::now();
        player.play_scalar(Duration::from_millis(300), Speed::new(100), None);
        wait_ms(100).await;
        player.scheduler.update_task(1, Speed::new(50));
        wait_ms(100).await;
        player.scheduler.update_task(1, Speed::new(10));
        player.await_all().await;

        client.print_device_calls(start);
        client.get_device_calls(1)[0]
            .assert_strenth(1.0)
            .assert_time(0, start);
        client.get_device_calls(1)[1]
            .assert_strenth(0.5)
            .assert_time(100, start);
        client.get_device_calls(1)[2]
            .assert_strenth(0.1)
            .assert_time(200, start);
        client.get_device_calls(1)[3]
            .assert_strenth(0.0)
            .assert_time(300, start);
    }

    #[tokio::test]
    async fn test_clean_finished_tasks() {
        // arrange
        let start = Instant::now();
        let client = get_test_client(vec![scalar(1, "vib1", ActuatorType::Vibrate)]).await;

        let mut player = PlayerTest::setup(&client.created_devices);
        player.play_scalar(Duration::from_millis(100), Speed::max(), None);
        for _ in 0..2 {
            player.play_scalar(Duration::from_millis(1), Speed::max(), None);
            player.await_last().await;
        }

        // act
        player.scheduler.clean_finished_tasks();

        // assert
        client.print_device_calls(start);
        assert_eq!(player.scheduler.control_handles.len(), 1);
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
            device_calls[i].assert_time((i * 100) as i32, start);
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
