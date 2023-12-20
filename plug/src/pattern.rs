use buttplug::client::{
    ButtplugClientDevice, ButtplugClientError, LinearCommand, ScalarCommand
};
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
use tracing::{error, info, trace, warn, debug};

type TkButtplugClientResult<T = ()> = Result<T, ButtplugClientError>;

pub struct TkPatternPlayer {
    pub actuators: Vec<Arc<TkActuator>>,
    pub action_sender: UnboundedSender<TkDeviceAction>,
    pub result_sender: UnboundedSender<TkButtplugClientResult>,
    pub result_receiver: UnboundedReceiver<TkButtplugClientResult>,
    pub player_scalar_resolution_ms: i32,
    pub handle: i32,
    pub cancellation_token: CancellationToken,
}

pub struct TkButtplugScheduler {
    device_action_sender: UnboundedSender<TkDeviceAction>,
    settings: TkPlayerSettings,
    cancellation_tokens: HashMap<i32, CancellationToken>,
    last_handle: i32,
}

pub struct TkPlayerSettings {
    pub player_scalar_resolution_ms: i32,
}

pub struct TkButtplugWorker {
    tasks: UnboundedReceiver<TkDeviceAction>,
}

#[derive(Clone, Debug)]
pub struct TkActuator {
    pub device: Arc<ButtplugClientDevice>,
    pub actuator: ActuatorType,
    pub index_in_device: u32,
}

impl TkActuator {
    pub fn identifier(&self) -> String {
        format!(
            "{}.{}[{}]",
            self.device.index(),
            self.actuator,
            self.index_in_device
        )
    }
}

pub fn get_actuators(devices: Vec<Arc<ButtplugClientDevice>>) -> Vec<Arc<TkActuator>> {
    let mut actuators = vec![];
    for device in devices {
        if let Some(scalar_cmd) = device.message_attributes().scalar_cmd() {
            for (idx, scalar_cmd) in scalar_cmd.iter().enumerate() {
                actuators.push(Arc::new(TkActuator {
                    device: device.clone(),
                    actuator: scalar_cmd.actuator_type().clone(),
                    index_in_device: idx as u32,
                }))
            }
        }
        if let Some(linear_cmd) = device.message_attributes().linear_cmd() {
            for (idx, _) in linear_cmd.iter().enumerate() {
                actuators.push(Arc::new(TkActuator {
                    device: device.clone(),
                    actuator: ActuatorType::Position,
                    index_in_device: idx as u32,
                }));
            }
        }
        if let Some(rotate_cmd) = device.message_attributes().rotate_cmd() {
            for (idx, _) in rotate_cmd.iter().enumerate() {
                actuators.push(Arc::new(TkActuator {
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
pub enum TkDeviceAction {
    Start(Arc<TkActuator>, Speed, bool, i32),
    Update(Arc<TkActuator>, Speed),
    End(
        Arc<TkActuator>,
        bool,
        i32,
        UnboundedSender<TkButtplugClientResult>,
    ),
    Move(
        Arc<TkActuator>,
        f64,
        u32,
        bool,
        UnboundedSender<TkButtplugClientResult>,
    ),
    StopAll, // global but required for resetting device state
}

#[derive(Debug, Clone, Copy)]
pub struct Speed {
    pub value: u16,
}

#[derive(Clone, Debug)]
pub enum TkPattern {
    Linear(Duration, Speed),
    Funscript(Duration, Arc<FScript>),
}

#[derive(Clone, Debug)]
pub struct TkFunscript {
    pub duration: Duration,
    pub pattern: String,
}

struct ReferenceCounter {
    access_list: HashMap<String, u32>,
}

struct DeviceAccess {
    device_actions: HashMap<u32, Vec<(i32, Speed)>>,
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

impl ReferenceCounter {
    pub fn new() -> Self {
        ReferenceCounter {
            access_list: HashMap::new(),
        }
    }
    pub fn reserve(&mut self, actuator: &Arc<TkActuator>) {
        self.access_list
            .entry(actuator.identifier())
            .and_modify(|counter| *counter += 1)
            .or_insert(1);
        trace!(
            "Reserved device={} ref-count={}",
            actuator.identifier(),
            self.current_references(&actuator)
        )
    }
    pub fn release(&mut self, actuator: &Arc<TkActuator>) {
        self.access_list
            .entry(actuator.identifier())
            .and_modify(|counter| {
                if *counter > 0 {
                    *counter -= 1
                } else {
                    warn!("Release on ref-count=0")
                }
            })
            .or_insert(0);
        trace!(
            "Released device={} ref-count={}",
            actuator.identifier(),
            self.current_references(&actuator)
        )
    }

    pub fn should_stop(&self, actuator: &Arc<TkActuator>) -> bool {
        self.current_references(actuator) == 0
    }

    fn current_references(&self, actuator: &Arc<TkActuator>) -> u32 {
        match self.access_list.get(&actuator.identifier()) {
            Some(count) => *count,
            None => 0,
        }
    }

    pub fn clear(&mut self) {
        self.access_list.clear();
    }
}

impl DeviceAccess {
    pub fn new() -> Self {
        DeviceAccess {
            device_actions: HashMap::new(),
        }
    }

    pub fn record_start(&mut self, actuator: &Arc<TkActuator>, handle: i32, action: Speed) {
        self.device_actions
            .entry(actuator.device.index())
            .and_modify(|bag| bag.push((handle, action)))
            .or_insert(vec![(handle, action)]);
    }

    pub fn record_stop(&mut self, actuator: &Arc<TkActuator>, handle: i32) {
        if let Some(action) = self.device_actions.remove(&actuator.device.index()) {
            let except_handle: Vec<(i32, Speed)> =
                action.into_iter().filter(|t| t.0 != handle).collect();
            self.device_actions
                .insert(actuator.device.index(), except_handle);
        }
    }

    pub fn get_remaining_speed(&self, actuator: &Arc<TkActuator>) -> Option<Speed> {
        if let Some(actions) = self.device_actions.get(&actuator.device.index()) {
            let mut sorted: Vec<(i32, Speed)> = actions.clone();
            sorted.sort_by_key(|b| b.0);
            if let Some(tuple) = sorted.last() {
                return Some(tuple.1);
            }
        }
        None
    }

    pub fn get_actual_speed(&self, actuator: &Arc<TkActuator>, new_speed: Speed) -> Speed {
        if let Some(actions) = self.device_actions.get(&actuator.device.index()) {
            let mut sorted: Vec<(i32, Speed)> = actions.clone();
            sorted.sort_by_key(|b| b.0);
            if let Some(tuple) = sorted.last() {
                return tuple.1;
            }
        }
        new_speed
    }
}

impl TkButtplugWorker {
    /// Process the queue of all device actions from all player threads
    ///
    /// This was introduced so that that the housekeeping and the decision which
    /// thread gets priority  on a device is always done in the same thread and
    /// its not necessary to introduce Mutex/etc to handle multithreaded access
    pub async fn run_worker_thread(&mut self) {

        // TODO do cleanup of cancelled
        let mut device_counter = ReferenceCounter::new();
        let mut device_access = DeviceAccess::new();
        loop {
            if let Some(next_action) = self.tasks.recv().await {
                trace!("exec device action {:?}", next_action);
                match next_action {
                    TkDeviceAction::Start(actuator, speed, priority, handle) => {
                        device_counter.reserve(&actuator);
                        if priority {
                            device_access.record_start(&actuator, handle, speed);
                        }
                        let cmd = &ScalarCommand::ScalarMap(HashMap::from([(
                            actuator.index_in_device,
                            (speed.as_float(), actuator.actuator),
                        )]));
                        match actuator.device.scalar(cmd).await {
                            Err(err) => {
                                error!("failed to set scalar speed {:?}", err)
                            }
                            _ => {}
                        }
                    }
                    TkDeviceAction::Update(actuator, speed) => {
                        let cmd = &ScalarCommand::ScalarMap(HashMap::from([(
                            actuator.index_in_device,
                            (
                                device_access.get_actual_speed(&actuator, speed).as_float(),
                                actuator.actuator,
                            ),
                        )]));
                        actuator
                            .device
                            .scalar(cmd)
                            .await
                            .unwrap_or_else(|_| error!("Failed to set device vibration speed."))
                    }
                    TkDeviceAction::End(actuator, priority, handle, result_sender) => {
                        device_counter.release(&actuator);
                        if priority {
                            device_access.record_stop(&actuator, handle);
                        }

                        let mut result = Ok(());
                        if device_counter.should_stop(&actuator) {
                            // nothing else is controlling the device, stop it
                            if let Err(error) = actuator
                                .device
                                .scalar(&ScalarCommand::ScalarMap(HashMap::from([(
                                    actuator.index_in_device,
                                    (0.0, actuator.actuator),
                                )])))
                                .await
                            {
                                error!("Failed to stop vibration on actuator {:?}", actuator);
                                result = Err(error);
                            }
                            debug!("Device stopped {}", actuator.identifier())
                        } else if let Some(remaining_speed) =
                            device_access.get_remaining_speed(&actuator)
                        {
                            // see if we have an earlier action still requiring movement
                            if let Err(error) = actuator
                                .device
                                .scalar(&ScalarCommand::ScalarMap(HashMap::from([(
                                    actuator.index_in_device,
                                    (remaining_speed.as_float(), actuator.actuator),
                                )])))
                                .await
                            {
                                result = Err(error);
                                error!("Failed to reset vibration to previous speed={} on actuator {:?}", remaining_speed, actuator);
                            }
                        }
                        result_sender.send(result).unwrap();
                    }
                    TkDeviceAction::Move(
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
                            result_sender
                                .send(result)
                                .unwrap();
                        }
                    }
                    TkDeviceAction::StopAll => {
                        device_counter.clear();
                        info!("stop all action");
                    }
                }
            }
        }
    }
}

impl TkButtplugScheduler {
    fn get_next_handle(&mut self) -> i32 {
        self.last_handle += 1;
        self.last_handle
    }

    pub fn create(settings: TkPlayerSettings) -> (TkButtplugScheduler, TkButtplugWorker) {
        let (device_action_sender, tasks) = unbounded_channel::<TkDeviceAction>();
        (
            TkButtplugScheduler {
                device_action_sender,
                settings,
                cancellation_tokens: HashMap::new(),
                last_handle: 0,
            },
            TkButtplugWorker { tasks },
        )
    }

    pub fn stop_task(&mut self, handle: i32) {
        if self.cancellation_tokens.contains_key(&handle) {
            self.cancellation_tokens.remove(&handle).unwrap().cancel();
        } else {
            error!("Unknown handle {}", handle);
        }
    }

    pub fn create_player(&mut self, actuators: Vec<Arc<TkActuator>>) -> TkPatternPlayer {
        let cancellation_token = CancellationToken::new();
        let handle = self.get_next_handle();
        self.cancellation_tokens
            .insert(handle, cancellation_token.clone());
        let (result_sender, result_receiver) = unbounded_channel::<Result<(), ButtplugClientError>>();
        TkPatternPlayer {
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
            .send(TkDeviceAction::StopAll)
            .unwrap_or_else(|_| error!(queue_full_err));
        for entry in self.cancellation_tokens.drain() {
            entry.1.cancel();
        }
    }
}

impl TkPatternPlayer {
    /// Executes the linear 'fscript' for 'duration' and consumes the player
    pub async fn play_linear(
        mut self,
        fscript: FScript,
        duration: Duration,
    ) -> TkButtplugClientResult {
        let handle = self.handle;
        info!("start pattern {:?} <linear> ({})", fscript, handle);
        let mut last_result = Ok(());
        if fscript.actions.len() == 0 || fscript.actions.iter().all(|x| x.at == 0) {
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

    /// Executes the scalar 'fscript' for 'duration' and consumes the player
    pub async fn play_scalar(self, pattern: TkPattern) -> TkButtplugClientResult {
        let handle = self.handle;
        info!("start pattern {:?} <scalar> ({})", pattern, handle);
        let result = match pattern {
            TkPattern::Linear(duration, speed) => {
                self.do_scalar(speed, true);
                cancellable_wait(duration, &self.cancellation_token).await;
                self.do_stop(true).await
            }
            TkPattern::Funscript(duration, fscript) => {
                if fscript.actions.len() == 0 || fscript.actions.iter().all(|x| x.at == 0) {
                    return Ok(());
                }
                let waiter = self.stop_after(duration);
                let action_len = fscript.actions.len();
                let mut started = false;
                let mut loop_started = Instant::now();
                let mut i: usize = 0;
                loop {
                    let mut j = 1;
                    while j + i < action_len - 1
                        && (&fscript.actions[i + j].at - &fscript.actions[i].at)
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
                        if false == cancellable_wait(waiting_time, &self.cancellation_token).await {
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
        };
        info!("stop pattern ({})", handle);
        result
    }

    fn do_update(&self, speed: Speed) {
        for actuator in self.actuators.iter() {
            trace!("do_update {} {:?}", speed, actuator);
            self.action_sender
                .send(TkDeviceAction::Update(actuator.clone(), speed))
                .unwrap_or_else(|_| error!("queue full"));
        }
    }

    fn do_scalar(&self, speed: Speed, priority: bool) {
        for actuator in self.actuators.iter() {
            trace!("do_scalar {} {:?}", speed, actuator);
            self.action_sender
                .send(TkDeviceAction::Start(
                    actuator.clone(),
                    speed,
                    priority,
                    self.handle,
                ))
                .unwrap_or_else(|_| error!("queue full"));
        }
    }

    async fn do_stop(mut self, priority: bool) -> TkButtplugClientResult {
        trace!("do_stop");
        for actuator in self.actuators.iter() {
            trace!("do_stop actuator {:?}", actuator);
            self.action_sender
                .send(TkDeviceAction::End(
                    actuator.clone(),
                    priority,
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

    async fn do_linear(&mut self, pos: f64, duration_ms: u32) -> TkButtplugClientResult {
        for actuator in self.actuators.iter() {
            self.action_sender
                .send(TkDeviceAction::Move(
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
            return false;
        }
        _ = sleep(duration) => {
            return true;
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::fakes::tests::get_test_client;
    use crate::fakes::{linear, scalar, scalars, FakeMessage};
    use crate::pattern::TkPattern;
    use crate::Speed;
    use std::sync::Arc;
    use std::thread;
    use std::time::{Duration, Instant};

    use tokio::runtime::Handle;

    use funscript::{FSPoint, FScript};
    use futures::future::join_all;

    use buttplug::client::ButtplugClientDevice;
    use buttplug::core::message::ActuatorType;
    use tokio::task::JoinHandle;
    use tokio::time::timeout;

    use super::{get_actuators, TkActuator, TkButtplugScheduler, TkPlayerSettings};

    struct PlayerTest {
        pub scheduler: TkButtplugScheduler,
        pub handles: Vec<JoinHandle<()>>,
        pub all_devices: Vec<Arc<ButtplugClientDevice>>,
    }

    impl PlayerTest {
        fn setup(all_devices: &Vec<Arc<ButtplugClientDevice>>) -> Self {
            PlayerTest::setup_with_settings(
                all_devices,
                TkPlayerSettings {
                    player_scalar_resolution_ms: 1,
                },
            )
        }

        fn setup_with_settings(
            all_devices: &Vec<Arc<ButtplugClientDevice>>,
            settings: TkPlayerSettings,
        ) -> Self {
            let (scheduler, mut worker) = TkButtplugScheduler::create(settings);
            Handle::current().spawn(async move {
                worker.run_worker_thread().await;
            });
            PlayerTest {
                scheduler,
                handles: vec![],
                all_devices: all_devices.clone(),
            }
        }

        fn play_background_on(&mut self, pattern: TkPattern, actuators: Vec<Arc<TkActuator>>) {
            let player = self.scheduler.create_player(actuators);
            self.handles.push(Handle::current().spawn(async move {
                player.play_scalar(pattern).await.unwrap();
            }));
        }

        fn play_background(&mut self, pattern: TkPattern) {
            let player = self
                .scheduler
                .create_player(get_actuators(self.all_devices.clone()));
            self.handles.push(Handle::current().spawn(async move {
                player.play_scalar(pattern).await.unwrap();
            }));
        }

        async fn play(&mut self, pattern: TkPattern) {
            let player = self
                .scheduler
                .create_player(get_actuators(self.all_devices.clone()));
            player.play_scalar(pattern).await.unwrap();
        }

        async fn play_scalar_on_actuator(&mut self, pattern: TkPattern, actuator: Arc<TkActuator>) {
            let player = self.scheduler.create_player(vec![actuator]);
            player.play_scalar(pattern).await.unwrap();
        }

        async fn play_linear(&mut self, funscript: FScript, duration: Duration) {
            let player = self
                .scheduler
                .create_player(get_actuators(self.all_devices.clone()));
            player.play_linear(funscript, duration).await.unwrap();
        }

        async fn finish_background(self) {
            join_all(self.handles).await;
        }
    }

    /// Linear
    #[tokio::test]
    async fn test_no_devices_does_not_block() {
        // arrange
        let client: crate::fakes::tests::ButtplugTestClient = get_test_client(vec![]).await;
        let mut player = PlayerTest::setup(&client.created_devices);

        let mut fs: FScript = FScript::default();
        fs.actions.push(FSPoint { pos: 1, at: 10 });
        fs.actions.push(FSPoint { pos: 2, at: 20 });
        
        // act & assert
        assert!(
            timeout(
                Duration::from_secs(1),
                player.play(TkPattern::Linear(Duration::from_millis(50), Speed::max())),
            )
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
        let client: crate::fakes::tests::ButtplugTestClient =
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
        let mut pattern =
            TkPattern::Funscript(Duration::from_millis(1), Arc::new(FScript::default()));
        player.play(pattern).await;

        let mut fscript = FScript::default();
        fscript.actions.push(FSPoint { pos: 0, at: 0 });
        fscript.actions.push(FSPoint { pos: 0, at: 0 });
        pattern = TkPattern::Funscript(Duration::from_millis(200), Arc::new(fscript));
        player.play(pattern).await;
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
            .play_scalar_on_actuator(
                TkPattern::Funscript(Duration::from_millis(125), Arc::new(fs1)),
                actuators[1].clone(),
            )
            .await;

        let mut fs2 = FScript::default();
        fs2.actions.push(FSPoint { pos: 30, at: 0 });
        fs2.actions.push(FSPoint { pos: 40, at: 100 });
        player
            .play_scalar_on_actuator(
                TkPattern::Funscript(Duration::from_millis(125), Arc::new(fs2)),
                actuators[0].clone(),
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
        let pattern = TkPattern::Funscript(Duration::from_millis(125), Arc::new(fs));
        player.play(pattern).await;

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
            .play(TkPattern::Funscript(
                get_duration_ms(&fscript),
                Arc::new(fscript),
            ))
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
            TkPlayerSettings {
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
            .play(TkPattern::Funscript(
                Duration::from_millis(150),
                Arc::new(fs),
            ))
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

        player.play_background(TkPattern::Linear(
            Duration::from_millis(500),
            Speed::new(50),
        ));
        wait_ms(100).await;
        player
            .play(TkPattern::Linear(
                Duration::from_millis(100),
                Speed::new(100),
            ))
            .await;
        player.finish_background().await;

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
        player.play_background(TkPattern::Linear(Duration::from_secs(3), Speed::new(20)));
        wait_ms(250).await;

        player.play_background(TkPattern::Linear(Duration::from_secs(2), Speed::new(40)));
        wait_ms(250).await;

        player
            .play(TkPattern::Linear(Duration::from_secs(1), Speed::new(80)))
            .await;
        player.finish_background().await;

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
        player.play_background(TkPattern::Linear(Duration::from_secs(3), Speed::new(20)));
        wait_ms(250).await;

        player.play_background(TkPattern::Linear(Duration::from_secs(1), Speed::new(40)));
        wait_ms(250).await;

        player
            .play(TkPattern::Linear(Duration::from_secs(1), Speed::new(80)))
            .await;
        thread::sleep(Duration::from_secs(2));
        player.finish_background().await;

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
        player.play_background(TkPattern::Linear(Duration::from_secs(1), Speed::new(99)));
        wait_ms(250).await;
        player
            .play(TkPattern::Funscript(
                Duration::from_secs(3),
                Arc::new(fscript),
            ))
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
        player.play_background_on(
            TkPattern::Linear(Duration::from_millis(300), Speed::new(99)),
            get_actuators(vec![client.get_device(1)]),
        );
        player.play_background_on(
            TkPattern::Linear(Duration::from_millis(200), Speed::new(88)),
            get_actuators(vec![client.get_device(2)]),
        );

        player.finish_background().await;

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
