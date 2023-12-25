use funscript::FScript;
use tokio::runtime::Handle;
use tokio::task::JoinHandle;

use std::{sync::Arc, time::Duration};
use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    time::{sleep, Instant},
};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, trace};

use crate::{cancellable_wait, actuator::Actuator, speed::Speed, worker::{WorkerTask, ButtplugClientResult}};

/// Pattern executor that can be passed to a sub-thread
pub struct PatternPlayer {
    pub actuators: Vec<Arc<Actuator>>,
    pub action_sender: UnboundedSender<WorkerTask>,
    pub result_sender: UnboundedSender<ButtplugClientResult>,
    pub result_receiver: UnboundedReceiver<ButtplugClientResult>,
    pub player_scalar_resolution_ms: i32,
    pub handle: i32,
    pub cancellation_token: CancellationToken,
}

impl PatternPlayer {
    /// Executes the linear 'fscript' for 'duration' and consumes the player
    pub async fn play_linear(
        mut self,
        duration: Duration,
        fscript: FScript,
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

    /// Executes the scalar 'fscript' for 'duration' and consumes the player
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

    /// Executes a constant movement with 'speed' for 'duration' and consumes the player
    pub async fn play_scalar(self, duration: Duration, speed: Speed) -> ButtplugClientResult {
        self.do_scalar(speed, true);
        cancellable_wait(duration, &self.cancellation_token).await;
        self.do_stop(true).await
    }

    fn do_update(&self, speed: Speed) {
        for actuator in self.actuators.iter() {
            trace!("do_update {} {:?}", speed, actuator);
            self.action_sender
                .send(WorkerTask::Update(actuator.clone(), speed))
                .unwrap_or_else(|_| error!("queue full"));
        }
    }

    fn do_scalar(&self, speed: Speed, is_not_pattern: bool) {
        for actuator in self.actuators.iter() {
            trace!("do_scalar {} {:?}", speed, actuator);
            self.action_sender
                .send(WorkerTask::Start(
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
                .send(WorkerTask::End(
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
                .send(WorkerTask::Move(
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
