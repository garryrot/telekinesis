use anyhow::anyhow;
use buttplug::client::{device, ButtplugClientDevice, ScalarValueCommand};
use buttplug::core::message::ActuatorType;
use funscript::{FSPoint, FScript};
use std::collections::HashMap;
use std::fmt::{self, Display};

use std::{
    fs::{self},
    path::PathBuf,
    sync::Arc,
    time::Duration,
};
use tokio::{
    sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    time::{sleep, Instant},
};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, trace, warn};

#[derive(Clone, Debug)]
pub enum TkDuration {
    Infinite,
    Timed(Duration),
}

pub struct TkPatternPlayer {
    pub devices: Vec<Arc<TkActuator>>,
    pub action_sender: UnboundedSender<TkDeviceAction>,
    pub resolution_ms: i32,
    pub pattern_path: String,
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
    pub player_resolution_ms: i32,
    pub pattern_path: String,
}

pub struct TkButtplugWorker {
    tasks: UnboundedReceiver<TkDeviceAction>,
}

struct TkPatternFile {
    path: PathBuf,
    is_vibration: bool,
    name: String,
}

#[derive(Clone, Debug)]
pub struct TkActuator {
    pub device: Arc<ButtplugClientDevice>,
    pub actuator: ActuatorType,
    pub index_in_device: usize,
}

impl TkActuator {
    pub fn identifier(&self) -> &String {
        // TODO: Needs to be actuator-specfic
        self.device.name()
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
                    index_in_device: idx,
                }))
            }
        }
        if let Some(linear_cmd) = device.message_attributes().linear_cmd() {
            for (idx, _) in linear_cmd.iter().enumerate() {
                actuators.push(Arc::new(TkActuator {
                    device: device.clone(),
                    actuator: ActuatorType::Position,
                    index_in_device: idx,
                }));
            }
        }
        if let Some(rotate_cmd) = device.message_attributes().rotate_cmd() {
            for (idx, _) in rotate_cmd.iter().enumerate() {
                actuators.push(Arc::new(TkActuator {
                    device: device.clone(),
                    actuator: ActuatorType::Rotate,
                    index_in_device: idx,
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
    Linear(TkDuration, Speed),
    Funscript(TkDuration, String),
}

#[derive(Clone, Debug)]
pub struct TkFunscript {
    pub duration: TkDuration,
    pub pattern: String,
}

struct ReferenceCounter {
    access_list: HashMap<u32, u32>,
}

struct DeviceAccess {
    device_actions: HashMap<u32, Vec<(i32, Speed)>>,
}

impl TkDuration {
    pub fn from_input_float(secs: f32) -> TkDuration {
        if secs > 0.0 {
            return TkDuration::Timed(Duration::from_millis((secs * 1000.0) as u64));
        } else {
            return TkDuration::Infinite;
        }
    }
    pub fn from_millis(ms: u64) -> TkDuration {
        TkDuration::Timed(Duration::from_millis(ms))
    }
    pub fn from_secs(s: u64) -> TkDuration {
        TkDuration::Timed(Duration::from_secs(s))
    }
    pub fn as_us(&self) -> u64 {
        match self {
            TkDuration::Infinite => u64::MAX,
            TkDuration::Timed(duration) => duration.as_micros() as u64,
        }
    }
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
            .entry(actuator.device.index())
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
            .entry(actuator.device.index())
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
        match self.access_list.get(&actuator.device.index()) {
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
                        let result = actuator
                            .device
                            .vibrate(&ScalarValueCommand::ScalarValue(
                                device_access.get_actual_speed(&actuator, speed).as_float(),
                                // TODO select only the
                            ))
                            .await;
                        match result {
                            Err(err) => {
                                // TODO: Send device error event
                                // TODO: Implement better connected/disconnected handling for devices
                                error!("Failed to set device vibration speed {:?}", err)
                            }
                            _ => {}
                        }
                    }
                    TkDeviceAction::Update(actuator, speed) => {
                        actuator
                            .device
                            .vibrate(&ScalarValueCommand::ScalarValue(
                                device_access.get_actual_speed(&actuator, speed).as_float(),
                                // TODO select just the actuator
                            ))
                            .await
                            .unwrap_or_else(|_| error!("Failed to set device vibration speed."))
                    }
                    TkDeviceAction::End(actuator, priority, handle) => {
                        device_counter.release(&actuator);
                        if priority {
                            device_access.record_stop(&actuator, handle);
                        }
                        if device_counter.should_stop(&actuator) {
                            // nothing else is controlling the device, stop it
                            actuator
                                .device
                                .vibrate(
                                    &ScalarValueCommand::ScalarValue(0.0), // TODO: select just the actuator
                                )
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
            devices: actuators,
            action_sender: self.device_action_sender.clone(),
            resolution_ms: 100,
            pattern_path: self.settings.pattern_path.clone(),
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
    pub async fn play_linear(&mut self, funscript: FScript, duration: TkDuration) {
        let mut last_timestamp: u32 = 0;
        for point in funscript.actions {
            if point.at != 0 {
                let point_as_float = (point.pos as f64) / 100.0;
                let duration_ms = point.at as u32 - last_timestamp;
                self.do_linear(point_as_float, duration_ms);
                trace!("do_linear to {} over {}ms", point_as_float, duration_ms);
                if false
                    == cancellable_wait(
                        Duration::from_millis(duration_ms as u64),
                        &self.cancellation_token,
                    )
                    .await
                {
                    break;
                };
            }
            last_timestamp = point.at as u32;
        }
    }

    pub async fn play_scalar(&mut self, pattern: TkPattern) {
        info!("Playing pattern {:?}", pattern);
        match pattern {
            TkPattern::Linear(duration, speed) => match duration {
                TkDuration::Infinite => {
                    self.do_scalar(speed, true, self.handle);
                    self.cancellation_token.cancelled().await;
                    self.do_stop(true, self.handle);
                    info!("Infinite stopped")
                }
                TkDuration::Timed(duration) => {
                    self.do_scalar(speed, true, self.handle);
                    cancellable_wait(duration, &self.cancellation_token).await;
                    self.do_stop(true, self.handle);
                    info!("Linear finished");
                }
            },
            TkPattern::Funscript(duration, pattern_name) => {
                match read_pattern_name(&self.pattern_path, &pattern_name, true) {
                    Ok(funscript) => {
                        let mut cancel = false;
                        let mut elapsed_us = 0 as u64;
                        while !cancel && elapsed_us < duration.as_us() {
                            let last_timer_us;
                            (cancel, last_timer_us) =
                                self.play_scalar_pattern(&duration, &funscript).await;
                            elapsed_us += last_timer_us;
                            info!("Elapsed: {} Cancel: {}", elapsed_us, cancel)
                        }
                    }
                    Err(err) => error!(
                        "Error loading funscript vibration pattern={} err={}",
                        pattern_name, err
                    ),
                }
            }
        }
    }

    async fn play_scalar_pattern(&self, duration: &TkDuration, funscript: &FScript) -> (bool, u64) {
        let actions = &funscript.actions;
        if actions.len() == 0 {
            return (true, 0);
        }
        let duration = match duration {
            TkDuration::Infinite => Duration::MAX,
            TkDuration::Timed(duration) => duration.clone(),
        };

        let mut cancelled = false;
        let mut dropped = 0;
        let mut ignored = 0;
        let now = Instant::now();

        let first_speed = Speed::from_fs(&actions[0]);
        self.do_scalar(first_speed, false, self.handle);

        let mut i = 1;
        let mut last_speed = first_speed.value as i32;
        let mut next_timer_us = 0;
        while i < actions.len() && now.elapsed() < duration {
            let point = &actions[i];

            // skip until we have reached a delay of resolution_ms
            let mut j = i;
            while j + 1 < actions.len() && (actions[j + 1].at - actions[i].at) < self.resolution_ms
            {
                dropped += 1;
                j += 1;
            }
            i = j;

            next_timer_us = (actions[i].at * 1000) as u64;
            let elapsed_us = now.elapsed().as_micros() as u64;
            if elapsed_us < next_timer_us {
                if false
                    == cancellable_wait(
                        Duration::from_micros(next_timer_us - elapsed_us),
                        &self.cancellation_token,
                    )
                    .await
                {
                    cancelled = true;
                    break;
                };
                if last_speed != point.pos {
                    self.do_update(Speed::from_fs(point));
                    last_speed = point.pos;
                } else {
                    ignored += 1;
                }
            }
            i += 1;
        }
        self.do_stop(false, self.handle);
        info!(
            "Pattern finished in {:?} dropped={} ignored={}",
            now.elapsed(),
            dropped,
            ignored
        );
        (cancelled, next_timer_us)
    }

    fn do_update(&self, speed: Speed) {
        for device in self.devices.iter() {
            trace!("do_update {} {:?}", speed, device);
            self.action_sender
                .send(TkDeviceAction::Update(device.clone(), speed))
                .unwrap_or_else(|_| error!("queue full"));
        }
    }

    fn do_linear(&self, pos: f64, duration_ms: u32) {
        for device in self.devices.iter() {
            self.action_sender
                .send(TkDeviceAction::Move(device.clone(), pos, duration_ms))
                .unwrap_or_else(|_| error!("queue full"));
        }
    }

    fn do_scalar(&self, speed: Speed, priority: bool, handle: i32) {
        for device in self.devices.iter() {
            trace!("do_scalar {} {:?}", speed, device);
            self.action_sender
                .send(TkDeviceAction::Start(
                    device.clone(),
                    speed,
                    priority,
                    handle,
                ))
                .unwrap_or_else(|_| error!("queue full"));
        }
    }

    fn do_stop(&self, priority: bool, handle: i32) {
        trace!("do_stop");
        for device in self.devices.iter() {
            self.action_sender
                .send(TkDeviceAction::End(device.clone(), priority, handle))
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

pub fn get_pattern_names(pattern_path: &str, vibration_patterns: bool) -> Vec<String> {
    match get_pattern_paths(pattern_path) {
        Ok(patterns) => patterns
            .iter()
            .filter(|p| p.is_vibration == vibration_patterns)
            .map(|p| p.name.clone())
            .collect::<Vec<String>>(),
        Err(err) => {
            error!("Failed reading patterns {}", err);
            vec![]
        }
    }
}

fn read_pattern_name(
    pattern_path: &str,
    pattern_name: &str,
    vibration_pattern: bool,
) -> Result<FScript, anyhow::Error> {
    let now = Instant::now();
    let patterns = get_pattern_paths(pattern_path)?;
    let pattern = patterns
        .iter()
        .filter(|d| {
            d.is_vibration == vibration_pattern
                && d.name.to_lowercase() == pattern_name.to_lowercase()
        })
        .next()
        .ok_or_else(|| anyhow!("Pattern '{}' not found", pattern_name))?;

    let fs = funscript::load_funscript(pattern.path.to_str().unwrap())?;
    debug!("Read pattern {} in {:?}", pattern_name, now.elapsed());
    Ok(fs)
}

fn get_pattern_paths(pattern_path: &str) -> Result<Vec<TkPatternFile>, anyhow::Error> {
    let mut patterns = vec![];
    let pattern_dir = fs::read_dir(pattern_path)?;
    for entry in pattern_dir {
        let file = entry?;

        let path = file.path();
        let path_clone = path.clone();
        let file_name = path
            .file_name()
            .ok_or_else(|| anyhow!("No file name"))?
            .to_str()
            .ok_or_else(|| anyhow!("Invalid unicode"))?;
        if false == file_name.to_lowercase().ends_with(".funscript") {
            continue;
        }

        let is_vibration = file_name.to_lowercase().ends_with(".vibrator.funscript");
        let removal;
        if is_vibration {
            removal = file_name.len() - ".vibrator.funscript".len();
        } else {
            removal = file_name.len() - ".funscript".len();
        }

        patterns.push(TkPatternFile {
            path: path_clone,
            is_vibration,
            name: String::from(&file_name[0..removal]),
        })
    }
    Ok(patterns)
}

#[cfg(test)]
mod tests {
    use crate::fakes::tests::get_test_client;
    use crate::fakes::{linear, scalar};
    use crate::pattern::TkPattern;
    use crate::{Speed, TkDuration};
    use buttplug::client::ButtplugClientDevice;
    use buttplug::core::message::ActuatorType;
    use funscript::{FSPoint, FScript};
    use futures::future::join_all;
    use std::sync::Arc;
    use std::thread;
    use std::time::{Duration, Instant};
    use tokio::runtime::Handle;
    use tokio::task::JoinHandle;

    use super::{get_actuators, TkButtplugScheduler, TkPlayerSettings};

    struct PlayerTest {
        scheduler: TkButtplugScheduler,
        handles: Vec<JoinHandle<()>>,
        all_devices: Vec<Arc<ButtplugClientDevice>>,
    }

    impl PlayerTest {
        fn setup(all_devices: &Vec<Arc<ButtplugClientDevice>>) -> Self {
            PlayerTest {
                scheduler: get_test_scheduler(),
                handles: vec![],
                all_devices: all_devices.clone(),
            }
        }

        fn play_background_on_device(
            &mut self,
            pattern: TkPattern,
            device: Arc<ButtplugClientDevice>,
        ) {
            let mut player = self.scheduler.create_player(get_actuators(vec![device]));
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

        async fn play_scalar(&mut self, pattern: TkPattern) {
            let mut player = self
                .scheduler
                .create_player(get_actuators(self.all_devices.clone()));
            player.play_scalar(pattern).await;
        }

        async fn play_linear(&mut self, funscript: FScript, duration: TkDuration) {
            let mut player = self
                .scheduler
                .create_player(get_actuators(self.all_devices.clone()));
            player.play_linear(funscript, duration).await;
        }

        async fn finish_background(self) {
            join_all(self.handles).await;
        }
    }

    async fn wait_ms(ms: u64) {
        tokio::time::sleep(Duration::from_millis(ms)).await;
    }

    fn get_test_scheduler() -> TkButtplugScheduler {
        let pattern_path =
            String::from("../contrib/Distribution/SKSE/Plugins/Telekinesis/Patterns");
        let (scheduler, mut worker) = TkButtplugScheduler::create(TkPlayerSettings {
            player_resolution_ms: 100,
            pattern_path,
        });
        Handle::current().spawn(async move {
            worker.run_worker_thread().await;
        });
        scheduler
    }

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
            TkDuration::from_millis(500),
            Speed::new(50),
        ));
        wait_ms(100).await;
        player
            .play_scalar(TkPattern::Linear(
                TkDuration::from_millis(100),
                Speed::new(100),
            ))
            .await;
        player.finish_background().await;

        // assert
        client.print_device_calls(1, start);
        client.get_messages(1)[0].assert_strenth(0.5);
        client.get_messages(1)[1].assert_strenth(1.0);
        client.get_messages(1)[2].assert_strenth(0.5);
        client.get_messages(1)[3].assert_strenth(0.0);
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
        player.play_background(TkPattern::Linear(TkDuration::from_secs(3), Speed::new(20)));
        wait_ms(250).await;

        player.play_background(TkPattern::Linear(TkDuration::from_secs(2), Speed::new(40)));
        wait_ms(250).await;

        player
            .play_scalar(TkPattern::Linear(TkDuration::from_secs(1), Speed::new(80)))
            .await;
        player.finish_background().await;

        // assert
        client.print_device_calls(1, start);

        client.get_messages(1)[0].assert_strenth(0.2);
        client.get_messages(1)[1].assert_strenth(0.4);
        client.get_messages(1)[2].assert_strenth(0.8);
        client.get_messages(1)[3].assert_strenth(0.4);
        client.get_messages(1)[4].assert_strenth(0.2);
        client.get_messages(1)[5].assert_strenth(0.0);
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
        player.play_background(TkPattern::Linear(TkDuration::from_secs(3), Speed::new(20)));
        wait_ms(250).await;

        player.play_background(TkPattern::Linear(TkDuration::from_secs(1), Speed::new(40)));
        wait_ms(250).await;

        player
            .play_scalar(TkPattern::Linear(TkDuration::from_secs(1), Speed::new(80)))
            .await;
        thread::sleep(Duration::from_secs(2));
        player.finish_background().await;

        // assert
        client.print_device_calls(1, start);
        client.get_messages(1)[0].assert_strenth(0.2);
        client.get_messages(1)[1].assert_strenth(0.4);
        client.get_messages(1)[2].assert_strenth(0.8);
        client.get_messages(1)[3].assert_strenth(0.8);
        client.get_messages(1)[4].assert_strenth(0.2);
        client.get_messages(1)[5].assert_strenth(0.0);
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
        let start = Instant::now();
        player.play_background(TkPattern::Linear(TkDuration::from_secs(1), Speed::new(99)));
        wait_ms(250).await;
        player
            .play_scalar(TkPattern::Funscript(
                TkDuration::from_secs(3),
                String::from("31_Sawtooth-Fast"),
            ))
            .await;

        // assert
        client.print_device_calls(1, start);
        assert!(client.call_registry.get_device(1).len() > 3);
    }

    #[tokio::test]
    async fn test_concurrency_two_devices_simulatenously_both_are_started_and_stopped() {
        let client = get_test_client(vec![
            scalar(1, "vib1", ActuatorType::Vibrate),
            scalar(2, "vib2", ActuatorType::Vibrate),
        ])
        .await;

        let devices = client.created_devices.clone();
        let mut player = PlayerTest::setup(&client.created_devices);

        // act
        player.play_background_on_device(
            TkPattern::Linear(TkDuration::from_millis(3000), Speed::new(99)),
            devices[0].clone(),
        );

        player.play_background_on_device(
            TkPattern::Linear(TkDuration::from_millis(3000), Speed::new(88)),
            devices[1].clone(),
        );

        player.finish_background().await;

        // assert
        client.get_messages(1)[0].assert_strenth(0.99);
        client.get_messages(1)[1].assert_strenth(0.0);
        client.get_messages(2)[0].assert_strenth(0.88);
        client.get_messages(2)[1].assert_strenth(0.0);
    }

    #[tokio::test]
    async fn test_linear_pattern() {
        let client = get_test_client(vec![linear(1, "lin1")]).await;

        let mut fscript = FScript::default();
        fscript.actions.push(FSPoint { pos: 50, at: 0 }); // ignored
        fscript.actions.push(FSPoint { pos: 0, at: 200 });
        fscript.actions.push(FSPoint { pos: 100, at: 450 });
        fscript.actions.push(FSPoint { pos: 25, at: 1000 });

        let mut player = PlayerTest::setup(&client.created_devices);
        player.play_linear(fscript, TkDuration::Infinite).await;

        client.get_messages(1)[0]
            .assert_position(0.0)
            .assert_duration(200);
        client.get_messages(1)[1]
            .assert_position(1.0)
            .assert_duration(250);
        client.get_messages(1)[2]
            .assert_position(0.25)
            .assert_duration(550);
    }
}
