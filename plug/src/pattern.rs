use std::{sync::Arc, time::Duration};

use buttplug::client::ButtplugClientDevice;
use tokio::{
    sync::mpsc::UnboundedSender,
    time::{sleep, Instant},
};
use tracing::{error, info, trace};

use crate::{commands::TkDeviceAction, event::TkEvent, inputs::Speed, TkPattern};

pub struct TkPatternPlayer {
    pub devices: Vec<Arc<ButtplugClientDevice>>,
    pub action_sender: UnboundedSender<TkDeviceAction>,
    pub event_sender: UnboundedSender<TkEvent>,
    pub resolution_ms: i32,
}

impl TkPatternPlayer {
    pub async fn play(self, pattern: TkPattern) {
        info!("Playing pattern {:?}", pattern);
        match pattern {
            TkPattern::Linear(duration, speed) => {
                // vibrate with speed
                self.do_vibrate(speed);
                sleep(duration).await;
                self.do_stop();
                info!("Linear finished");
            }
            TkPattern::Funscript(duration, pattern) => {
                match funscript::load_funscript(&pattern) {
                    Ok(funscript) => {
                        let actions = funscript.actions;
                        if actions.len() == 0 {
                            return;
                        }

                        let mut dropped = 0;
                        let mut ignored = 0;
                        let now = Instant::now();
                        self.do_vibrate(Speed::from_fs(&actions[0]));

                        let mut i = 1;
                        let mut last_pos = 0;
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
                                sleep(Duration::from_micros(next_timer_us - elapsed_us)).await;
                            }
                            if last_pos != point.pos {
                                self.do_update(Speed::from_fs(point));
                                last_pos = point.pos;
                            } else {
                                ignored += 1;
                            }
                            i += 1;
                        }
                        self.do_stop();
                        info!(
                            "Pattern finished in {:?} dropped={} ignored={}",
                            now.elapsed(),
                            dropped,
                            ignored
                        );
                    }
                    Err(err) => error!("Error loading funscript pattern={} err={}", pattern, err),
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

    fn do_vibrate(&self, speed: Speed) {
        trace!("do_vibrate {}", speed);
        for device in self.devices.iter() {
            self.action_sender
                .send(TkDeviceAction::Vibrate(device.clone(), speed))
                .unwrap_or_else(|_| error!("queue full"));
        }
        self.event_sender
            .send(TkEvent::DeviceVibrated(self.devices.len() as i32, speed))
            .unwrap_or_else(|_| error!("queue full"));
    }

    fn do_stop(&self) {
        trace!("do_vibrate");
        for device in self.devices.iter() {
            self.action_sender
                .send(TkDeviceAction::Stop(device.clone()))
                .unwrap_or_else(|_| error!("queue full"));
        }
        self.event_sender
            .send(TkEvent::DeviceStopped(self.devices.len() as i32))
            .unwrap_or_else(|_| error!("queue full"));
    }
}
