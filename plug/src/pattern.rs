use buttplug::client::{device, ButtplugClientDevice, ScalarCommand, ScalarValueCommand};
use buttplug::core::message::ActuatorType;
use buttplug::util::future;
use funscript::{FSPoint, FScript};
use std::collections::HashMap;
use std::fmt::{self, Display};

use std::{path::PathBuf, sync::Arc, time::Duration};
use tokio::{
    sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    time::{sleep, Instant},
};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, trace, warn};

pub struct TkPatternPlayer {
    pub actuators: Vec<Arc<TkActuator>>,
    pub action_sender: UnboundedSender<TkDeviceAction>,
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
    Move(Arc<TkActuator>, f64, u32),
    End(Arc<TkActuator>, bool, i32),
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
    pub async fn run_worker_thread(&mut self) {
        trace!("Exec run_worker_thread");

        // TODO do cleanup of cancelled
        let mut device_counter = ReferenceCounter::new();
        let mut device_access = DeviceAccess::new();
        loop {
            if let Some(next_action) = self.tasks.recv().await {
                trace!("Exec device action {:?}", next_action);
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
                                // TODO: Send device error event
                                // TODO: Implement better connected/disconnected handling for devices
                                error!("Failed to set scalar speed {:?}", err)
                            }
                            _ => {}
                        }
                    }
                    TkDeviceAction::Update(actuator, speed) => {
                        let cmd = &ScalarCommand::ScalarMap(HashMap::from([(
                            actuator.index_in_device,
                            (device_access.get_actual_speed(&actuator, speed).as_float(), actuator.actuator),
                        )]));
                        actuator
                            .device
                            .scalar(cmd)
                            .await
                            .unwrap_or_else(|_| error!("Failed to set device vibration speed."))
                        // TODO: Send device error event
                    }
                    TkDeviceAction::End(actuator, priority, handle) => {
                        device_counter.release(&actuator);
                        if priority {
                            device_access.record_stop(&actuator, handle);
                        }
                        if device_counter.should_stop(&actuator) {
                            // nothing else is controlling the device, stop it
                            let map = HashMap::from([(
                                actuator.index_in_device,
                                (0.0, actuator.actuator),
                            )]);
                            actuator
                                .device
                                .scalar(&ScalarCommand::ScalarMap(map))
                                .await
                                .unwrap_or_else(|_| error!("Failed to stop vibration"));
                            info!("Device stopped {}", actuator.identifier())
                        } else if let Some(remaining_speed) =
                            device_access.get_remaining_speed(&actuator)
                        {
                            // see if we have an earlier action still requiring movement
                            actuator
                                .device
                                .vibrate(&ScalarValueCommand::ScalarValue(
                                    remaining_speed.as_float(),
                                ))
                                .await
                                .unwrap_or_else(|_| error!("Failed to reset vibration"))
                        }
                    }
                    TkDeviceAction::Move(actuator, position, duration) => {
                        actuator
                            .device
                            .linear(&device::LinearCommand::Linear(duration, position))
                            .await
                            .unwrap_or_else(|_| error!("Failed to move linear"));
                    }
                    TkDeviceAction::StopAll => {
                        device_counter.clear();
                        info!("Stop all action");
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
        let token = CancellationToken::new();
        let handle = self.get_next_handle();
        self.cancellation_tokens.insert(handle, token.clone());
        TkPatternPlayer {
            actuators,
            action_sender: self.device_action_sender.clone(),
            player_scalar_resolution_ms: self.settings.player_scalar_resolution_ms,
            handle: handle,
            cancellation_token: token,
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
    pub async fn play_linear(&mut self, funscript: FScript, duration: Duration) {
        let start = Instant::now();
        while start.elapsed() <= duration {
            let current_start = Instant::now();
            for point in &funscript.actions {
                if start.elapsed() > duration {
                    break;
                }
                if point.at != 0 {
                    let point_as_float = (point.pos as f64) / 100.0;
                    let future_at = Duration::from_millis(point.at as u64);
                    let duration = future_at - current_start.elapsed();
                    self.do_linear(point_as_float, duration.as_millis() as u32);
                    trace!("do_linear to {} over {:?}", point_as_float, duration);
                    if false == cancellable_wait(duration, &self.cancellation_token).await {
                        break;
                    };
                }
            }
        }
    }

    pub async fn play_scalar(&mut self, pattern: TkPattern) {
        info!("Playing pattern {:?}", pattern);
        match pattern {
            TkPattern::Linear(duration, speed) => {
                self.do_scalar(speed, true, self.handle);
                cancellable_wait(duration, &self.cancellation_token).await;
                self.do_stop(true, self.handle);
                info!("Linear finished");
            },
            TkPattern::Funscript(duration, fscript) => {
                let actions = &fscript.actions;
                if actions.len() == 0 || 
                    actions.iter().all(|x| x.at == 0) {
                    return;
                }
                
                let start_timer = Instant::now();
                let mut dropped = 0;
                let mut ignored = 0;
                let mut cancel = false;
                while !cancel && start_timer.elapsed() < duration {
                    info!("OUTER {:?} {:?}", start_timer.elapsed(), duration);
                    let current_start = Instant::now();
                    let first_speed = Speed::from_fs(&actions[0]);
                    self.do_scalar(first_speed, false, self.handle);

                    let mut i = 1;
                    let mut last_speed = first_speed.value as i32;

                    let actions = &fscript.actions;
                    while i < actions.len() && start_timer.elapsed() < duration {
                        info!("INNER i={} {:?} {:?}", i, start_timer.elapsed(), duration);
                        // skip until we have reached a delay of player_scalar_resolution_ms
                        let mut j = i;
                        while j + 1 < actions.len() && (actions[j + 1].at - actions[i].at) < self.player_scalar_resolution_ms
                        {
                            dropped += 1;
                            j += 1;
                        }
                        i = j;

                        let future_at = Duration::from_millis(actions[i].at as u64);
                        if current_start.elapsed() < future_at {
                            let remaining = future_at - current_start.elapsed();
                            if false == cancellable_wait(remaining, &self.cancellation_token).await
                            {
                                cancel = true;
                                break;
                            };
                            let point = &actions[i];
                            if last_speed != point.pos {
                                self.do_update(Speed::from_fs(point));
                                last_speed = point.pos;
                            } else {
                                ignored += 1;
                            }
                        }
                        i += 1;
                    }
                }
                self.do_stop(false, self.handle);

                info!(
                    "Pattern finished {:?} dropped={} ignored={} cancelled={}",
                    start_timer.elapsed(),
                    dropped,
                    ignored,
                    cancel
                );
            }
        }
    }

    fn do_update(&self, speed: Speed) {
        for actuator in self.actuators.iter() {
            trace!("do_update {} {:?}", speed, actuator);
            self.action_sender
                .send(TkDeviceAction::Update(actuator.clone(), speed))
                .unwrap_or_else(|_| error!("queue full"));
        }
    }

    fn do_linear(&self, pos: f64, duration_ms: u32) {
        for actuator in self.actuators.iter() {
            self.action_sender
                .send(TkDeviceAction::Move(actuator.clone(), pos, duration_ms))
                .unwrap_or_else(|_| error!("queue full"));
        }
    }

    fn do_scalar(&self, speed: Speed, priority: bool, handle: i32) {
        for actuator in self.actuators.iter() {
            trace!("do_scalar {} {:?}", speed, actuator);
            self.action_sender
                .send(TkDeviceAction::Start(
                    actuator.clone(),
                    speed,
                    priority,
                    handle,
                ))
                .unwrap_or_else(|_| error!("queue full"));
        }
    }

    fn do_stop(&self, priority: bool, handle: i32) {
        trace!("do_stop");
        for actuator in self.actuators.iter() {
            self.action_sender
                .send(TkDeviceAction::End(actuator.clone(), priority, handle))
                .unwrap_or_else(|_| error!("queue full"));
        }
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
    use crate::util::enable_log;
    use std::sync::Arc;
    use std::thread;
    use std::time::{Duration, Instant};

    use tokio::runtime::Handle;
    use tokio::task::JoinHandle;

    use funscript::{FSPoint, FScript};
    use futures::future::join_all;

    use buttplug::client::ButtplugClientDevice;
    use buttplug::core::message::ActuatorType;

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
            let mut player = self.scheduler.create_player(actuators);
            self.handles.push(Handle::current().spawn(async move {
                player.play_scalar(pattern).await;
            }));
        }

        fn play_background(&mut self, pattern: TkPattern) {
            let mut player = self
                .scheduler
                .create_player(get_actuators(self.all_devices.clone()));
            self.handles.push(Handle::current().spawn(async move {
                player.play_scalar(pattern).await;
            }));
        }

        async fn play(&mut self, pattern: TkPattern) {
            let mut player = self
                .scheduler
                .create_player(get_actuators(self.all_devices.clone()));
            player.play_scalar(pattern).await;
        }

        async fn play_scalar_on_actuator(&mut self, pattern: TkPattern, actuator: Arc<TkActuator>) {
            let mut player = self.scheduler.create_player(vec![actuator]);
            player.play_scalar(pattern).await;
        }

        async fn play_linear(&mut self, funscript: FScript, duration: Duration) {
            let mut player = self
                .scheduler
                .create_player(get_actuators(self.all_devices.clone()));
            player.play_linear(funscript, duration).await;
        }

        async fn finish_background(self) {
            join_all(self.handles).await;
        }
    }

    // Linear Tests

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
        player.play_linear(get_repeated_pattern(n), get_duration_ms(&fscript)).await;
        wait_ms(1).await; // TODO await last

        // assert
        client.print_device_calls(start);
        check_timing( client.get_device_calls(1), n, start);
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
        assert_eq!(
            client.get_device_calls(1).len(),
            1,
            "Stops after duration ends"
        );
    }

    // Scalar Tests

    #[tokio::test]
    async fn test_scalar_pattern_actuator_access() {
        // arrange
        let client = get_test_client(vec![scalars(1, "vib1", ActuatorType::Vibrate, 2)]).await;
        let mut player = PlayerTest::setup(&client.created_devices);
        let actuators = get_actuators(client.created_devices.clone());

        // act
        let start = Instant::now();

        let mut fs1 = FScript::default();
        fs1.actions = vec![FSPoint { pos: 10, at: 0 }, FSPoint { pos: 20, at: 100 }];
        player
            .play_scalar_on_actuator(
                TkPattern::Funscript(Duration::from_millis(100), Arc::new(fs1)),
                actuators[1].clone(),
            )
            .await;

        let mut fs2 = FScript::default();
        fs2.actions = vec![FSPoint { pos: 30, at: 0 }, FSPoint { pos: 40, at: 100 }];
        player
            .play_scalar_on_actuator(
                TkPattern::Funscript(Duration::from_millis(100), Arc::new(fs2)),
                actuators[0].clone(),
            )
            .await;
        wait_ms(1).await; // TODO await last

        // assert
        client.print_device_calls(start);
        let calls = client.get_device_calls(1);
        calls[0].assert_strengths(vec![(1, 0.1)]);
        calls[1].assert_strengths(vec![(1, 0.2)]);
        calls[2].assert_strengths(vec![(1, 0.0)]);
        calls[3].assert_strengths(vec![(0, 0.3)]);
        calls[4].assert_strengths(vec![(0, 0.4)]);
        calls[5].assert_strengths(vec![(0, 0.0)]);
    }

    #[tokio::test]
    async fn test_scalar_pattern_repeats_until_duration_ends() {
        // arrange
        let client = get_test_client(vec![scalar(1, "vib1", ActuatorType::Vibrate)]).await;
        let mut player = PlayerTest::setup(&client.created_devices);

        // act
        let start = Instant::now();
        let mut fs = FScript::default();
        fs.actions.push(FSPoint { pos: 100, at: 0 });
        fs.actions.push(FSPoint { pos: 50, at: 50 });
        fs.actions.push(FSPoint { pos: 70, at: 100 });
        
        let pattern = TkPattern::Funscript(Duration::from_millis(125), Arc::new(fs));
        player.play(pattern).await;
        wait_ms(1).await; // TODO await last

        // assert
        client.print_device_calls(start);
        let calls = client.get_device_calls(1);
        calls[0].assert_strenth(1.0);
        calls[1].assert_strenth(0.5);
        calls[2].assert_strenth(0.7);
        calls[3].assert_strenth(1.0);
        calls[4].assert_strenth(0.5);
        assert_eq!(calls.len(), 5)
        // TODO: Repeat 1,5x times and abort in middle
    }

    #[tokio::test]
    async fn test_scalar_pattern_with_duration_0_does_not_block_forever() {
        let client = get_test_client(vec![scalar(1, "vib1", ActuatorType::Vibrate)]).await;
        let mut player = PlayerTest::setup(&client.created_devices);

        let mut fscript = FScript::default();
        fscript.actions.push(FSPoint { pos: 0, at: 0 });

        // act
        let pattern = TkPattern::Funscript(Duration::from_millis(200), Arc::new(fscript));
        player.play(pattern).await;
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
        player.play(TkPattern::Funscript(get_duration_ms(&fscript), Arc::new(fscript))).await;
        wait_ms(1).await; // TODO await last

        // assert
        client.print_device_calls(start);
        check_timing(client.get_device_calls(1), n, start);
    }

    fn check_timing( device_calls: Vec<FakeMessage>, n: usize, start: Instant ) {
        for i in 0..n-1 {
            device_calls[i].assert_timestamp((i * 100) as i32, start);
        }
    }

    fn get_repeated_pattern( n: usize ) -> FScript {
        let mut fscript = FScript::default();
        fscript.actions.push(FSPoint { pos: 0, at: 0 });
        for i in 0..n {
            fscript.actions.push(FSPoint {
                pos: (i % 100) as i32,
                at: (i * 100) as i32,
            });
        }
        fscript
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
}
