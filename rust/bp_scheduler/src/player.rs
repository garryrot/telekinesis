use anyhow::{anyhow, Context};
use funscript::FScript;
use tokio::{io::split, runtime::Handle};
use tokio::task::JoinHandle;

use std::num::ParseIntError;
use std::{any, fmt, sync::Arc, time::Duration};
use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    time::{sleep, Instant},
};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, instrument, trace};

use crate::{
    actuator::Actuator,
    cancellable_wait,
    speed::Speed,
    worker::{ButtplugClientResult, WorkerTask},
};

/// Pattern executor that can be passed from the schedulers main-thread to a sub-thread
pub struct PatternPlayer {
    pub handle: i32,
    pub scalar_resolution_ms: i32,
    pub actuators: Vec<Arc<Actuator>>,
    pub result_sender: UnboundedSender<ButtplugClientResult>,
    pub result_receiver: UnboundedReceiver<ButtplugClientResult>,
    pub update_receiver: UnboundedReceiver<Speed>,
    pub cancellation_token: CancellationToken,
    pub worker_task_sender: UnboundedSender<WorkerTask>,
}

pub struct LinearRange {
    pub start_pos: f64,
    pub end_pos: f64,
    pub min_dur: f64,
    pub max_dur: f64,
    pub invert: bool
}

impl Default for LinearRange {
    fn default() -> Self {
        Self {
            start_pos: 0.0, 
            end_pos: 100.0,
            min_dur: 250.0,
            max_dur: 4000.0,
            invert: false
        }
    }
}

impl LinearRange {
    pub fn start_pos(&self) -> f64 {
        match self.invert {
            true => self.end_pos,
            false => self.start_pos
        }
    }
    pub fn end_pos(&self) -> f64 {
        match self.invert {
            true => self.start_pos,
            false => self.end_pos
        }
    }
}

impl PatternPlayer {
    pub async fn play_oscillate_linear(
        mut self,
        duration: Duration,
        speed: Speed,
        range: LinearRange
    ) -> ButtplugClientResult {
        let waiter = self.stop_after(duration);

        let get_t = |speed: Speed| {  
            let factor = (100 - speed.value) as f64 / 100.0;
            (range.min_dur + (range.max_dur - range.min_dur) * factor) as u32
        };

        let mut current_speed = speed;
        while !self.cancelled() {

            self.try_update(&mut current_speed);
            let mut wait_ms = get_t(current_speed);
            self.do_linear(range.end_pos(), wait_ms).await.unwrap();
            sleep(Duration::from_millis(wait_ms as u64)).await;

            if self.cancelled() {
                break;
            }

            self.try_update(&mut current_speed);
            wait_ms = get_t(current_speed);
            self.do_linear(range.start_pos(), wait_ms).await.unwrap();
            sleep(Duration::from_millis(wait_ms as u64)).await;
        }

        waiter.abort();
        Ok(())
    }

    /// Executes the linear 'fscript' for 'duration' and consumes the player
    #[instrument(skip(fscript))]
    pub async fn play_linear(
        mut self,
        duration: Duration,
        fscript: FScript,
        speed: Speed,
    ) -> ButtplugClientResult {
        info!("linear pattern started");
        let mut last_result = Ok(());
        if speed.as_float() <= 0.0
            || fscript.actions.is_empty()
            || fscript.actions.iter().all(|x| x.at == 0)
        {
            return last_result;
        }
        let mut current_speed = speed;
        let waiter = self.stop_after(duration);
        while !self.cancellation_token.is_cancelled() {
            let mut last_instant = Instant::now();
            let mut last_at = Duration::ZERO;
            let mut last_waiting_time = Duration::ZERO;
            for point in fscript.actions.iter() {
                if let Ok(update) = self.update_receiver.try_recv() {
                    if update.as_float() > 0.0 {
                        current_speed = update;
                    }
                }
                let waiting_time_us = Duration::from_millis(point.at as u64)
                    .saturating_sub(last_at)
                    .as_micros() as f64;
                let offset: Duration = last_instant.elapsed().saturating_sub(last_waiting_time);
                let factor = 1.0 / current_speed.as_float();
                let actual_waiting_time =
                    Duration::from_micros((waiting_time_us * factor) as u64).saturating_sub(offset);

                last_instant = Instant::now();
                last_at = Duration::from_millis(point.at as u64);
                last_waiting_time = actual_waiting_time;
                if actual_waiting_time == Duration::ZERO {
                    debug!("duration is zero, skipping");
                    continue;
                }

                let token = &self.cancellation_token.clone();
                if let Some(result) = tokio::select! {
                    _ = token.cancelled() => {
                        debug!("linear pattern cancelled");
                        break;
                    }
                    result = async {
                        let result = self.do_linear(Speed::from_fs(point).as_float(), actual_waiting_time.as_millis() as u32).await;
                        debug!("sleeping for {:?}...", actual_waiting_time);
                        sleep(actual_waiting_time).await;
                        result
                    } => {
                        Some(result)
                    }
                } {
                    last_result = result;
                }
            }
        }
        waiter.abort();
        info!("linear pattern done");
        last_result
    }

    /// Executes the scalar 'fscript' for 'duration' and consumes the player
    #[instrument(skip(fscript))]
    pub async fn play_scalar_pattern(
        mut self,
        duration: Duration,
        fscript: FScript,
        speed: Speed,
    ) -> ButtplugClientResult {
        if fscript.actions.is_empty() || fscript.actions.iter().all(|x| x.at == 0) {
            return Ok(());
        }
        info!("scalar pattern started");
        let waiter = self.stop_after(duration);
        let action_len = fscript.actions.len();
        let mut started = false;
        let mut loop_started = Instant::now();
        let mut i: usize = 0;
        let mut current_speed = speed;
        loop {
            let mut j = 1;
            while j + i < action_len - 1
                && (fscript.actions[i + j].at - fscript.actions[i].at) < self.scalar_resolution_ms
            {
                j += 1;
            }
            let current = &fscript.actions[i % action_len];
            let next = &fscript.actions[(i + j) % action_len];
            if let Ok(update) = self.update_receiver.try_recv() {
                current_speed = update;
            }

            if !started {
                self.do_scalar(Speed::from_fs(current).multiply(&current_speed), true);
                started = true;
            } else {
                self.do_update(Speed::from_fs(current).multiply(&current_speed), true);
            }
            if let Some(waiting_time) =
                Duration::from_millis(next.at as u64).checked_sub(loop_started.elapsed())
            {
                if !(cancellable_wait(waiting_time, &self.cancellation_token).await) {
                    debug!("scalar pattern cancelled");
                    break;
                }
            }
            i += j;
            if (i % action_len) == 0 {
                loop_started = Instant::now();
            }
        }
        waiter.abort();
        let result = self.do_stop(true).await;
        info!("scalar pattern done");
        result
    }

    /// Executes a constant movement with 'speed' for 'duration' and consumes the player
    #[instrument]
    pub async fn play_scalar(mut self, duration: Duration, speed: Speed) -> ButtplugClientResult {
        info!("scalar started");
        let waiter = self.stop_after(duration);
        self.do_scalar(speed, false);
        loop {
            tokio::select! {
                _ = self.cancellation_token.cancelled() => {
                    break;
                }
                update = self.update_receiver.recv() => {
                    if let Some(speed) = update {
                        self.do_update(speed, false);
                    }
                }
            };
        }
        waiter.abort();
        let result = self.do_stop(false).await;
        info!("scalar done");
        result
    }

    fn do_update(&self, speed: Speed, is_pattern: bool) {
        for actuator in self.actuators.iter() {
            trace!("do_update {} {:?}", speed, actuator);
            self.worker_task_sender
                .send(WorkerTask::Update(
                    actuator.clone(),
                    speed,
                    is_pattern,
                    self.handle,
                ))
                .unwrap_or_else(|err| error!("queue err {:?}", err));
        }
    }

    #[instrument(skip(self))]
    fn do_scalar(&self, speed: Speed, is_pattern: bool) {
        for actuator in self.actuators.iter() {
            trace!("do_scalar");
            self.worker_task_sender
                .send(WorkerTask::Start(
                    actuator.clone(),
                    speed,
                    is_pattern,
                    self.handle,
                ))
                .unwrap_or_else(|err| error!("queue err {:?}", err));
        }
    }

    #[instrument(skip(self))]
    async fn do_stop(mut self, is_pattern: bool) -> ButtplugClientResult {
        for actuator in self.actuators.iter() {
            trace!("do_stop");
            self.worker_task_sender
                .send(WorkerTask::End(
                    actuator.clone(),
                    is_pattern,
                    self.handle,
                    self.result_sender.clone(),
                ))
                .unwrap_or_else(|err| error!("queue err {:?}", err));
        }
        let mut last_result = Ok(());
        for _ in self.actuators.iter() {
            last_result = self.result_receiver.recv().await.unwrap();
        }
        last_result
    }

    #[instrument(skip(self))]
    async fn do_linear(&mut self, pos: f64, duration_ms: u32) -> ButtplugClientResult {
        for actuator in &self.actuators {
            trace!("do_linear");
            self.worker_task_sender
                .send(WorkerTask::Move(
                    actuator.clone(),
                    pos,
                    duration_ms,
                    true,
                    self.result_sender.clone(),
                ))
                .unwrap_or_else(|err| error!("queue err {:?}", err));
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

    fn try_update(&mut self, speed: &mut Speed) {
        if let Ok(update) = self.update_receiver.try_recv() {
            *speed = update;
        }
    }

    fn cancelled(&self) -> bool {
        self.cancellation_token.is_cancelled()
    }
}

impl fmt::Debug for PatternPlayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PatternPlayer")
            .field("actuators", &self.actuators)
            .field("handle", &self.handle)
            .field("resolution", &self.scalar_resolution_ms)
            .finish()
    }
}
