use anyhow::anyhow;
use buttplug::client::{ButtplugClientDevice, ScalarValueCommand};
use funscript::FScript;
use std::collections::HashMap;
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

use crate::{Speed, TkDuration, TkPattern};

#[derive(Clone, Debug)]
pub enum TkDeviceAction {
    Start(Arc<ButtplugClientDevice>, Speed, bool, i32),
    Update(Arc<ButtplugClientDevice>, Speed),
    End(Arc<ButtplugClientDevice>, bool, i32),
    StopAll, // global but required for resetting device state
}

struct ReferenceCounter {
    access_list: HashMap<u32, u32>,
}

impl ReferenceCounter {
    pub fn new() -> Self {
        ReferenceCounter {
            access_list: HashMap::new(),
        }
    }
    pub fn reserve(&mut self, device: &Arc<ButtplugClientDevice>) {
        self.access_list
            .entry(device.index())
            .and_modify(|counter| *counter += 1)
            .or_insert(1);
        trace!(
            "Reserved device={} ref-count={}",
            device.name(),
            self.current_references(&device)
        )
    }
    pub fn release(&mut self, device: &Arc<ButtplugClientDevice>) {
        self.access_list
            .entry(device.index())
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
            device.name(),
            self.current_references(&device)
        )
    }

    pub fn needs_to_stop(&self, device: &Arc<ButtplugClientDevice>) -> bool {
        self.current_references(device) == 0
    }

    fn current_references(&self, device: &Arc<ButtplugClientDevice>) -> u32 {
        match self.access_list.get(&device.index()) {
            Some(count) => *count,
            None => 0,
        }
    }

    pub fn clear(&mut self) {
        self.access_list.clear();
    }
}

struct DeviceAccess {
    device_actions: HashMap<u32, Vec<(i32, Speed)>>,
}

impl DeviceAccess {
    pub fn new() -> Self {
        DeviceAccess {
            device_actions: HashMap::new(),
        }
    }

    pub fn record_start(&mut self, device: &Arc<ButtplugClientDevice>, handle: i32, action: Speed) {
        self.device_actions
            .entry(device.index())
            .and_modify(|bag| bag.push((handle, action)))
            .or_insert(vec![(handle, action)]);
    }

    pub fn record_stop(&mut self, device: &Arc<ButtplugClientDevice>, handle: i32) {
        if let Some(action) = self.device_actions.remove(&device.index()) {
            let except_handle: Vec<(i32, Speed)> =
                action.into_iter().filter(|t| t.0 != handle).collect();
            self.device_actions.insert(device.index(), except_handle);
        }
    }

    pub fn get_remaining_speed(&self, device: &Arc<ButtplugClientDevice>) -> Option<Speed> {
        if let Some(actions) = self.device_actions.get(&device.index()) {
            let mut sorted: Vec<(i32, Speed)> = actions.clone();
            sorted.sort_by_key(|b| b.0);
            if let Some(tuple) = sorted.last() {
                return Some(tuple.1);
            }
        }
        None
    }

    pub fn calculate_actual_speed(
        &self,
        device: &Arc<ButtplugClientDevice>,
        new_speed: Speed,
    ) -> Speed {
        if let Some(actions) = self.device_actions.get(&device.index()) {
            let mut sorted: Vec<(i32, Speed)> = actions.clone();
            sorted.sort_by_key(|b| b.0);
            if let Some(tuple) = sorted.last() {
                return tuple.1;
            }
        }
        new_speed
    }
}

pub struct TkPlayerSettings {
    pub player_resolution_ms: i32,
    pub pattern_path: String,
}

pub struct TkButtplugScheduler {
    device_action_sender: UnboundedSender<TkDeviceAction>,
    settings: TkPlayerSettings,
    cancellation_tokens: HashMap<i32, CancellationToken>,
    last_handle: i32,
}

impl TkButtplugScheduler {
    fn get_next_handle(&mut self) -> i32 {
        self.last_handle += 1;
        self.last_handle
    }

    pub fn create(settings: TkPlayerSettings) -> (TkButtplugScheduler, UnboundedReceiver<TkDeviceAction>) {
        let (device_action_sender, device_action_receiver) = unbounded_channel::<TkDeviceAction>();
        (
            TkButtplugScheduler {
                device_action_sender,
                settings,
                cancellation_tokens: HashMap::new(),
                last_handle: 0,
            },
            device_action_receiver,
        )
    }

    pub async fn run_worker_thread(mut device_action_receiver: UnboundedReceiver<TkDeviceAction>) {
        // TODO do cleanup of cancelled
        let mut device_counter = ReferenceCounter::new();
        let mut device_access = DeviceAccess::new();
        loop {
            if let Some(next_action) = device_action_receiver.recv().await {
                trace!("Exec device action {:?}", next_action);
                match next_action {
                    TkDeviceAction::Start(device, speed, priority, handle) => {
                        device_counter.reserve(&device);
                        if priority {
                            device_access.record_start(&device, handle, speed);
                        }
                        let result = device
                            .vibrate(&ScalarValueCommand::ScalarValue(
                                device_access
                                    .calculate_actual_speed(&device, speed)
                                    .as_float(),
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
                    TkDeviceAction::Update(device, speed) => device
                        .vibrate(&ScalarValueCommand::ScalarValue(
                            device_access
                                .calculate_actual_speed(&device, speed)
                                .as_float(),
                        ))
                        .await
                        .unwrap_or_else(|_| error!("Failed to set device vibration speed.")),
                    TkDeviceAction::End(device, priority, handle) => {
                        device_counter.release(&device);
                        if priority {
                            device_access.record_stop(&device, handle);
                        }
                        if device_counter.needs_to_stop(&device) {
                            // nothing else is controlling the device, stop it
                            device
                                .vibrate(&ScalarValueCommand::ScalarValue(0.0))
                                .await
                                .unwrap_or_else(|_| error!("Failed to stop vibration"));
                            info!("Device stopped {}", device.name())
                        } else if let Some(remaining_speed) =
                            device_access.get_remaining_speed(&device)
                        {
                            // see if we have a lower priority vibration still running
                            device
                                .vibrate(&ScalarValueCommand::ScalarValue(
                                    remaining_speed.as_float(),
                                ))
                                .await
                                .unwrap_or_else(|_| error!("Failed to reset vibration"))
                        }
                    }
                    TkDeviceAction::StopAll => {
                        device_counter.clear();
                        info!("Stop all action");
                    }
                }
            }
        }
    }

    pub fn stop_task(&mut self, handle: i32) {
        if self.cancellation_tokens.contains_key(&handle) {
            self.cancellation_tokens.remove(&handle).unwrap().cancel();
        } else {
            error!("Unknown handle {}", handle);
        }
    }

    pub fn create_player(&mut self, devices: Vec<Arc<ButtplugClientDevice>>) -> TkPatternPlayer {
        let token = CancellationToken::new();
        let handle = self.get_next_handle();
        self.cancellation_tokens.insert(handle, token.clone());
        TkPatternPlayer {
            devices: devices,
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

pub struct TkPatternPlayer {
    pub devices: Vec<Arc<ButtplugClientDevice>>,
    pub action_sender: UnboundedSender<TkDeviceAction>,
    pub resolution_ms: i32,
    pub pattern_path: String,
    pub handle: i32,
    pub cancellation_token: CancellationToken,
}

impl TkPatternPlayer {
    pub async fn play(&mut self, pattern: TkPattern) {
        info!("Playing pattern {:?}", pattern);
        match pattern {
            TkPattern::Linear(duration, speed) => match duration {
                TkDuration::Infinite => {
                    self.do_vibrate(speed, true, self.handle);
                    self.cancellation_token.cancelled().await;
                    self.do_stop(true, self.handle);
                    info!("Infinite stopped")
                }
                TkDuration::Timed(duration) => {
                    self.do_vibrate(speed, true, self.handle);
                    cancellable_wait(duration, &self.cancellation_token).await;
                    self.do_stop(true, self.handle);
                    info!("Linear finished");
                }
            },
            TkPattern::Funscript(duration, pattern_name) => {
                match read_pattern_name(&self.pattern_path, &pattern_name, true) {
                    Ok(funscript) => {
                        let actions = funscript.actions;
                        if actions.len() == 0 {
                            return;
                        }
                        let duration = match duration {
                            TkDuration::Infinite => Duration::MAX,
                            TkDuration::Timed(duration) => duration,
                        };

                        let mut dropped = 0;
                        let mut ignored = 0;
                        let now = Instant::now();

                        let first_speed = Speed::from_fs(&actions[0]);
                        self.do_vibrate(first_speed, false, self.handle);

                        let mut i = 1;
                        let mut last_speed = first_speed.value as i32;
                        while i < actions.len() && now.elapsed() < duration {
                            let point = &actions[i];

                            // skip until we have reached a delay of resolution_ms
                            let mut j = i;
                            while j + 1 < actions.len()
                                && (actions[j + 1].at - actions[i].at) < self.resolution_ms
                            {
                                dropped += 1;
                                j += 1;
                            }
                            i = j;

                            let next_timer_us = (actions[i].at * 1000) as u64;
                            let elapsed_us = now.elapsed().as_micros() as u64;
                            if elapsed_us < next_timer_us {
                                if false
                                    == cancellable_wait(
                                        Duration::from_micros(next_timer_us - elapsed_us),
                                        &self.cancellation_token,
                                    )
                                    .await
                                {
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
                    }
                    Err(err) => error!(
                        "Error loading funscript pattern={} err={}",
                        pattern_name, err
                    ),
                }
            }
        }
    }

    fn do_update(&self, speed: Speed) {
        for device in self.devices.iter() {
            trace!("do_update {} {:?}", speed, device);
            self.action_sender
                .send(TkDeviceAction::Update(device.clone(), speed))
                .unwrap_or_else(|_| error!("queue full"));
        }
    }

    fn do_vibrate(&self, speed: Speed, priority: bool, handle: i32) {
        for device in self.devices.iter() {
            trace!("do_vibrate {} {:?}", speed, device);
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

struct TkPatternFile {
    path: PathBuf,
    is_vibration: bool,
    name: String,
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
