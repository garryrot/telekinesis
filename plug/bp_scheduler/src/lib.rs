use buttplug::client::{ButtplugClientDevice, ButtplugClientError, LinearCommand, ScalarCommand};
use buttplug::core::message::ActuatorType;
use funscript::{FSPoint, FScript};
use std::collections::HashMap;
use std::fmt::{self, Display};
use tokio::runtime::Handle;
use tokio::task::JoinHandle;

use std::{sync::Arc, time::Duration};
use tokio::{
    sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    time::{sleep, Instant},
};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, trace};

type ButtplugClientResult<T = ()> = Result<T, ButtplugClientError>;

/// Pattern executor that can be passed to a sub-thread
pub struct PatternPlayer {
    pub actuators: Vec<Arc<Actuator>>,
    pub action_sender: UnboundedSender<DeviceAction>,
    pub result_sender: UnboundedSender<ButtplugClientResult>,
    pub result_receiver: UnboundedReceiver<ButtplugClientResult>,
    pub player_scalar_resolution_ms: i32,
    pub handle: i32,
    pub cancellation_token: CancellationToken,
}

pub struct ButtplugScheduler {
    device_action_sender: UnboundedSender<DeviceAction>,
    settings: PlayerSettings,
    cancellation_tokens: HashMap<i32, CancellationToken>,
    last_handle: i32,
}

pub struct PlayerSettings {
    pub player_scalar_resolution_ms: i32,
}

pub struct ButtplugWorker {
    tasks: UnboundedReceiver<DeviceAction>,
}

#[derive(Clone, Debug)]
pub struct Actuator {
    pub device: Arc<ButtplugClientDevice>,
    pub actuator: ActuatorType,
    pub index_in_device: u32,
}

impl Display for Actuator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}[{}].{}", self.device.name(), self.index_in_device, self.actuator)
    }
}

impl Actuator {
    pub fn identifier(&self) -> String {
        self.to_string()
    }
}

pub fn get_actuators(devices: Vec<Arc<ButtplugClientDevice>>) -> Vec<Arc<Actuator>> {
    let mut actuators = vec![];
    for device in devices {
        if let Some(scalar_cmd) = device.message_attributes().scalar_cmd() {
            for (idx, scalar_cmd) in scalar_cmd.iter().enumerate() {
                actuators.push(Arc::new(Actuator {
                    device: device.clone(),
                    actuator: *scalar_cmd.actuator_type(),
                    index_in_device: idx as u32,
                }))
            }
        }
        if let Some(linear_cmd) = device.message_attributes().linear_cmd() {
            for (idx, _) in linear_cmd.iter().enumerate() {
                actuators.push(Arc::new(Actuator {
                    device: device.clone(),
                    actuator: ActuatorType::Position,
                    index_in_device: idx as u32,
                }));
            }
        }
        if let Some(rotate_cmd) = device.message_attributes().rotate_cmd() {
            for (idx, _) in rotate_cmd.iter().enumerate() {
                actuators.push(Arc::new(Actuator {
                    device: device.clone(),
                    actuator: ActuatorType::Rotate,
                    index_in_device: idx as u32,
                }))
            }
        }
    }
    actuators
}

#[derive(Clone, Debug)]
pub enum DeviceAction {
    Start(Arc<Actuator>, Speed, bool, i32),
    Update(Arc<Actuator>, Speed),
    End(
        Arc<Actuator>,
        bool,
        i32,
        UnboundedSender<ButtplugClientResult>,
    ),
    Move(
        Arc<Actuator>,
        f64,
        u32,
        bool,
        UnboundedSender<ButtplugClientResult>,
    ),
    StopAll, // global but required for resetting device state
}

#[derive(Debug, Clone, Copy)]
pub struct Speed {
    pub value: u16,
}

impl Display for Speed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Speed {
    pub fn new(mut percentage: i64) -> Speed {
        if percentage < 0 {
            percentage = 0;
        }
        if percentage > 100 {
            percentage = 100;
        }
        Speed {
            value: percentage as u16,
        }
    }
    pub fn from_fs(point: &FSPoint) -> Speed {
        Speed::new(point.pos.into())
    }
    pub fn min() -> Speed {
        Speed { value: 0 }
    }
    pub fn max() -> Speed {
        Speed { value: 100 }
    }
    pub fn as_float(self) -> f64 {
        self.value as f64 / 100.0
    }
}

/// Stores information about concurrent accesses to a buttplug actuator
/// to calculate the actual vibration speed or linear movement
struct DeviceEntry {
    /// The amount of tasks that currently access this device,
    pub task_count: usize,
    /// Priority calculation work like a stack with the top of the stack
    /// task being the used vibration speed
    pub linear_tasks: Vec<(i32, Speed)>
}

struct DeviceAccess {
    device_actions: HashMap<String, DeviceEntry>
}

impl DeviceAccess {
    pub fn default() -> Self {
        DeviceAccess {
            device_actions: HashMap::new(),
        }
    }

    async fn set_scalar(&self, actuator: &Arc<Actuator>, speed: Speed) -> Result<(), ButtplugClientError> {
        let cmd = ScalarCommand::ScalarMap(HashMap::from([(
            actuator.index_in_device,
            (speed.as_float(), actuator.actuator),
        )]));
        if let Err(err) = actuator
            .device
            .scalar(&cmd)
        .await
        {
            error!("failed to set scalar speed {:?}", err);
            return Err(err);
        }
        debug!("Device stopped {}", actuator.identifier());
        Ok(())
    }

    pub async fn start_scalar(&mut self, actuator: &Arc<Actuator>, speed: Speed, is_not_pattern: bool, handle: i32) {
        trace!("start scalar {} {}", actuator, handle);
        self.device_actions
            .entry(actuator.identifier())
            .and_modify(|entry| {
                entry.task_count += 1;
                if is_not_pattern {
                    entry.linear_tasks.push((handle, speed))
                }
            })
            .or_insert_with(|| {
                DeviceEntry {
                    task_count: 1,
                    linear_tasks: if is_not_pattern { vec![(handle, speed)] } else { vec![] }
                }
            });
        self.update_scalar(actuator, speed).await;
    }

    pub async fn stop_scalar(&mut self, actuator: &Arc<Actuator>, is_not_pattern: bool, handle: i32) -> Result<(), ButtplugClientError> {
        trace!("stop scalar {} {}", actuator, handle);
        if let Some(mut entry) = self.device_actions.remove(&actuator.identifier()) {
            if is_not_pattern {
                entry.linear_tasks.retain(|t| t.0 != handle);
            }
            let mut task_count = entry.task_count;
            if task_count != 0 {
                task_count -= 1;
            }
            entry.task_count = task_count;
            self.device_actions.insert(actuator.identifier(), entry);
            if task_count == 0 {
                // nothing else is controlling the device, stop it
                return self.set_scalar( actuator, Speed::min() ).await;
            } else if let Some(last_speed) = self.get_priority_speed(actuator) {
                self.update_scalar(actuator, last_speed).await;
            }
        }
        Ok(())
    }

    pub async fn update_scalar(&self, actuator: &Arc<Actuator>, new_speed: Speed) {
        let speed = self.get_priority_speed(actuator).unwrap_or(new_speed);
        debug!("updating {} speed to {}", actuator, speed);
        let _ = self.set_scalar( actuator, speed ).await;
    }

    fn get_priority_speed(&self, actuator: &Arc<Actuator>) -> Option<Speed> {
        if let Some(entry) = self.device_actions.get(&actuator.identifier()) {
            let mut sorted: Vec<(i32, Speed)> = entry.linear_tasks.clone();
            sorted.sort_by_key(|b| b.0);
            if let Some(tuple) = sorted.last() {
                return Some(tuple.1);
            }
        }
        None
    }

    pub fn clear_all(&mut self) {
        self.device_actions.clear();
    }
}

impl ButtplugWorker {
    /// Process the queue of all device actions from all player threads
    ///
    /// This was introduced so that that the housekeeping and the decision which
    /// thread gets priority on a device is always done in the same thread and
    /// its not necessary to introduce Mutex/etc to handle multithreaded access
    pub async fn run_worker_thread(&mut self) {
        let mut device_access = DeviceAccess::default();
        loop {
            if let Some(next_action) = self.tasks.recv().await {
                trace!("exec device action {:?}", next_action);
                match next_action {
                    DeviceAction::Start(actuator, speed, is_not_pattern, handle) => {
                        device_access.start_scalar(&actuator, speed, is_not_pattern, handle).await;
                    }
                    DeviceAction::Update(actuator, speed) => {
                        device_access.update_scalar(&actuator, speed).await;
                    }
                    DeviceAction::End(actuator, is_not_pattern, handle, result_sender) => {
                        let result = device_access.stop_scalar(&actuator, is_not_pattern, handle).await;
                        result_sender.send(result).unwrap();
                    }
                    DeviceAction::Move(
                        actuator,
                        position,
                        duration_ms,
                        finish,
                        result_sender,
                    ) => {
                        let cmd = LinearCommand::LinearMap(HashMap::from([(
                            actuator.index_in_device,
                            (duration_ms, position),
                        )]));
                        let result = actuator.device.linear(&cmd).await;
                        if finish {
                            result_sender.send(result).unwrap();
                        }
                    }
                    DeviceAction::StopAll => {
                        device_access.clear_all();
                        info!("stop all action");
                    }
                }
            }
        }
    }
}

impl ButtplugScheduler {
    fn get_next_handle(&mut self) -> i32 {
        self.last_handle += 1;
        self.last_handle
    }

    pub fn create(settings: PlayerSettings) -> (ButtplugScheduler, ButtplugWorker) {
        let (device_action_sender, tasks) = unbounded_channel::<DeviceAction>();
        (
            ButtplugScheduler {
                device_action_sender,
                settings,
                cancellation_tokens: HashMap::new(),
                last_handle: 0,
            },
            ButtplugWorker { tasks },
        )
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
            action_sender: self.device_action_sender.clone(),
            player_scalar_resolution_ms: self.settings.player_scalar_resolution_ms,
        }
    }

    pub fn stop_all(&mut self) {
        let queue_full_err = "Event sender full";
        self.device_action_sender
            .send(DeviceAction::StopAll)
            .unwrap_or_else(|_| error!(queue_full_err));
        for entry in self.cancellation_tokens.drain() {
            entry.1.cancel();
        }
    }
}

impl PatternPlayer {
    /// Executes the linear 'fscript' for 'duration' and consumes the player
    pub async fn play_linear(
        mut self,
        fscript: FScript,
        duration: Duration,
    ) -> ButtplugClientResult {
        let handle = self.handle;
        info!("start pattern {:?} <linear> ({})", fscript, handle);
        let mut last_result = Ok(());
        if fscript.actions.is_empty() || fscript.actions.iter().all(|x| x.at == 0) {
            return last_result;
        }
        let waiter = self.stop_after(duration);
        while !self.cancellation_token.is_cancelled() {
            let started = Instant::now();
            for point in fscript.actions.iter() {
                let point_as_float = Speed::from_fs(point).as_float();
                if let Some(waiting_time) =
                    Duration::from_millis(point.at as u64).checked_sub(started.elapsed())
                {
                    let token = &self.cancellation_token.clone();
                    if let Some(result) = tokio::select! {
                        _ = token.cancelled() => { None }
                        result = async {
                            let r = self.do_linear(point_as_float, waiting_time.as_millis() as u32).await;
                            sleep(waiting_time).await;
                            r
                        } => {
                            Some(result)
                        }
                    } {
                        last_result = result;
                    }
                }
            }
        }
        waiter.abort();
        info!("stop pattern ({})", handle);
        last_result
    }

    pub async fn play_scalar_pattern(
        self,
        duration: Duration,
        fscript: FScript,
    ) -> ButtplugClientResult {
        if fscript.actions.is_empty() || fscript.actions.iter().all(|x| x.at == 0) {
            return Ok(());
        }
        info!(
            "start pattern {}(ms) for {:?} <scalar> ({})",
            fscript.actions.last().unwrap().at,
            duration,
            self.handle
        );
        let waiter = self.stop_after(duration);
        let action_len = fscript.actions.len();
        let mut started = false;
        let mut loop_started = Instant::now();
        let mut i: usize = 0;
        loop {
            let mut j = 1;
            while j + i < action_len - 1
                && (fscript.actions[i + j].at - fscript.actions[i].at)
                    < self.player_scalar_resolution_ms
            {
                j += 1;
            }
            let current = &fscript.actions[i % action_len];
            let next = &fscript.actions[(i + j) % action_len];

            if !started {
                self.do_scalar(Speed::from_fs(current), false);
                started = true;
            } else {
                self.do_update(Speed::from_fs(current))
            }
            if let Some(waiting_time) =
                Duration::from_millis(next.at as u64).checked_sub(loop_started.elapsed())
            {
                if !(cancellable_wait(waiting_time, &self.cancellation_token).await) {
                    break;
                }
            }
            i += j;
            if (i % action_len) == 0 {
                loop_started = Instant::now();
            }
        }
        waiter.abort();
        self.do_stop(false).await
    }

    /// Executes the scalar 'fscript' for 'duration' and consumes the player
    pub async fn play_scalar(self, duration: Duration, speed: Speed) -> ButtplugClientResult {
        self.do_scalar(speed, true);
        cancellable_wait(duration, &self.cancellation_token).await;
        self.do_stop(true).await
    }

    fn do_update(&self, speed: Speed) {
        for actuator in self.actuators.iter() {
            trace!("do_update {} {:?}", speed, actuator);
            self.action_sender
                .send(DeviceAction::Update(actuator.clone(), speed))
                .unwrap_or_else(|_| error!("queue full"));
        }
    }

    fn do_scalar(&self, speed: Speed, is_not_pattern: bool) {
        for actuator in self.actuators.iter() {
            trace!("do_scalar {} {:?}", speed, actuator);
            self.action_sender
                .send(DeviceAction::Start(
                    actuator.clone(),
                    speed,
                    is_not_pattern,
                    self.handle,
                ))
                .unwrap_or_else(|_| error!("queue full"));
        }
    }

    async fn do_stop(mut self, is_not_pattern: bool) -> ButtplugClientResult {
        trace!("do_stop");
        for actuator in self.actuators.iter() {
            trace!("do_stop actuator {:?}", actuator);
            self.action_sender
                .send(DeviceAction::End(
                    actuator.clone(),
                    is_not_pattern,
                    self.handle,
                    self.result_sender.clone(),
                ))
                .unwrap_or_else(|_| error!("queue full"));
        }
        let mut last_result = Ok(());
        for _ in self.actuators.iter() {
            last_result = self.result_receiver.recv().await.unwrap();
        }
        last_result
    }

    async fn do_linear(&mut self, pos: f64, duration_ms: u32) -> ButtplugClientResult {
        for actuator in &self.actuators {
            self.action_sender
                .send(DeviceAction::Move(
                    actuator.clone(),
                    pos,
                    duration_ms,
                    true,
                    self.result_sender.clone(),
                ))
                .unwrap_or_else(|_| error!("queue full"));
        }
        self.result_receiver.recv().await.unwrap()
    }

    fn stop_after(&self, duration: Duration) -> JoinHandle<()> {
        let cancellation_clone = self.cancellation_token.clone();
        Handle::current().spawn(async move {
            sleep(duration).await;
            cancellation_clone.cancel();
        })
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
    use bp_fakes::get_test_client;
    use bp_fakes::FakeMessage;
    use bp_fakes::*;

    use crate::Speed;
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

    use super::{get_actuators, Actuator, ButtplugScheduler, PlayerSettings};

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
            actuators: Option<Vec<Arc<Actuator>>>,
        ) {
            let actuators = match actuators {
                Some(actuators) => actuators,
                None => get_actuators(self.all_devices.clone()),
            };
            let player: super::PatternPlayer = self.scheduler.create_player(actuators);
            player.play_scalar_pattern(duration, fscript).await.unwrap();
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
            player.play_linear(funscript, duration).await.unwrap();
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
        let client =
            get_test_client(vec![linear(1, "lin1")]).await;
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
        player.play_scalar_pattern(duration, fscript, None).await;

        let mut fscript = FScript::default();
        fscript.actions.push(FSPoint { pos: 0, at: 0 });
        fscript.actions.push(FSPoint { pos: 0, at: 0 });
        player
            .play_scalar_pattern(Duration::from_millis(200), fscript, None)
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
            .play_scalar_pattern(Duration::from_millis(125), fs, None)
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
            .play_scalar_pattern(get_duration_ms(&fscript), fscript, None)
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
            .play_scalar_pattern(Duration::from_millis(150), fs, None)
            .await;

        // assert
        client.print_device_calls(start);
        let calls = client.get_device_calls(1);
        calls[0].assert_strenth(0.42).assert_timestamp(0, start);
        calls[1].assert_strenth(0.42).assert_timestamp(100, start);
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
            .play_scalar_pattern(Duration::from_secs(3), fscript, None)
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
