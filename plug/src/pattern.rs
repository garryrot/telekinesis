use anyhow::anyhow;
use buttplug::client::ButtplugClientDevice;
use funscript::FScript;
use std::{
    fs::{self},
    path::PathBuf,
    sync::Arc,
    time::Duration,
};
use tokio::{
    sync::mpsc::UnboundedSender,
    time::{sleep, Instant},
};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, trace, debug};

use crate::{commands::TkDeviceAction, event::TkEvent, inputs::Speed, TkDuration, TkPattern};

pub struct TkPatternPlayer {
    pub devices: Vec<Arc<ButtplugClientDevice>>,
    pub action_sender: UnboundedSender<TkDeviceAction>,
    pub event_sender: UnboundedSender<TkEvent>,
    pub resolution_ms: i32,
    pub pattern_path: String,
}

impl TkPatternPlayer {
    pub async fn play(self, pattern: TkPattern, cancel: CancellationToken, handle: i32) {
        info!("Playing pattern {:?}", pattern);
        match pattern {
            TkPattern::Linear(duration, speed) => match duration {
                TkDuration::Infinite => {
                    self.do_vibrate(speed, true, handle);
                    cancel.cancelled().await;
                    self.do_stop(true, handle);
                    info!("Infinite stopped")
                }
                TkDuration::Timed(duration) => {
                    self.do_vibrate(speed, true, handle);
                    cancellable_wait(duration, &cancel).await;
                    self.do_stop(true, handle);
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
                        self.do_vibrate(first_speed, false, handle);

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
                                        &cancel,
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
                        self.do_stop(false, handle);
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
        trace!("do_update {}", speed);
        for device in self.devices.iter() {
            self.action_sender
                .send(TkDeviceAction::Update(device.clone(), speed))
                .unwrap_or_else(|_| error!("queue full"));
        }
    }

    fn do_vibrate(&self, speed: Speed, priority: bool, handle: i32) {
        trace!("do_vibrate {}", speed);
        for device in self.devices.iter() {
            self.action_sender
                .send(TkDeviceAction::Start(device.clone(), speed, priority, handle))
                .unwrap_or_else(|_| error!("queue full"));
        }
        self.event_sender
            .send(TkEvent::DeviceVibrated(self.devices.len() as i32, speed))
            .unwrap_or_else(|_| error!("queue full"));
    }

    fn do_stop(&self, priority: bool, handle: i32) {
        trace!("do_stop");
        for device in self.devices.iter() {
            self.action_sender
                .send(TkDeviceAction::End(device.clone(), priority, handle))
                .unwrap_or_else(|_| error!("queue full"));
        }
        self.event_sender
            .send(TkEvent::DeviceStopped())
            .unwrap_or_else(|_| error!("queue full"));
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
