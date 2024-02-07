use funscript::FScript;
use tokio::runtime::Handle;
use tokio::task::JoinHandle;

use std::{fmt, sync::Arc, time::Duration};
use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    time::{sleep, Instant},
};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, instrument, trace};

use crate::{
    actuator::Actuator, cancellable_wait, settings::{ActuatorSettings, LinearRange}, speed::Speed, worker::{ButtplugClientResult, WorkerTask}
};

/// Pattern executor that can be passed from the schedulers main-thread to a sub-thread
pub struct PatternPlayer {
    pub handle: i32,
    pub scalar_resolution_ms: i32,
    pub actuators: Vec<Arc<Actuator>>,
    pub settings: Vec<ActuatorSettings>,
    pub result_sender: UnboundedSender<ButtplugClientResult>,
    pub result_receiver: UnboundedReceiver<ButtplugClientResult>,
    pub update_receiver: UnboundedReceiver<Speed>,
    pub cancellation_token: CancellationToken,
    pub worker_task_sender: UnboundedSender<WorkerTask>,
}

impl PatternPlayer {
    pub async fn play_oscillate_linear(
        mut self,
        duration: Duration,
        speed: Speed,
        settings: LinearRange
    ) -> ButtplugClientResult {
        debug!(?settings, "oscillation started");
        let waiter = self.stop_after(duration);
        let mut current_speed = speed;
        while !self.cancelled() {
            self.try_update(&mut current_speed);
            self.do_stroke(true, current_speed, &settings).await.unwrap();
            if self.cancelled() {
                break;
            }
            self.try_update(&mut current_speed);
            self.do_stroke(false, current_speed, &settings).await.unwrap();
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
    ) -> ButtplugClientResult {
        info!("linear pattern started");
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
                            self.do_linear(point_as_float, waiting_time.as_millis() as u32).await
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

            let speed = Speed::from_fs(current).multiply(&current_speed);
            if !started {
                self.do_scalar(speed, true);
                started = true;
            } else {
                self.do_update(speed, true);
            }
            if let Some(waiting_time) =
                Duration::from_millis(next.at as u64).checked_sub(loop_started.elapsed())
            {
                debug!(?speed, ?waiting_time, "vibrating");
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
        for (i, actuator) in self.actuators.iter().enumerate() {
            trace!("do_update {} {:?}", speed, actuator);
            self.worker_task_sender
                .send(WorkerTask::Update(
                    actuator.clone(),
                    apply_scalar_settings(speed, &self.settings[ i ]),
                    is_pattern,
                    self.handle,
                ))
                .unwrap_or_else(|err| error!("queue err {:?}", err));
        }
    }

    #[instrument(skip(self))]
    fn do_scalar(&self, speed: Speed, is_pattern: bool) {
        for (i, actuator) in self.actuators.iter().enumerate() {
            trace!("do_scalar");
            self.worker_task_sender
                .send(WorkerTask::Start(
                    actuator.clone(),
                    apply_scalar_settings(speed, &self.settings[ i ]),
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
    async fn do_linear(&mut self, mut pos: f64, duration_ms: u32) -> ButtplugClientResult {
        for (i, actuator) in self.actuators.iter().enumerate() {
            let settings = &self.settings[ i ].linear_or_max();
            if settings.invert {
                pos = 1.0 - pos;
            }
            debug!(?duration_ms, ?pos, ?settings, "linear");
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
        sleep(Duration::from_millis(duration_ms as u64)).await;
        self.result_receiver.recv().await.unwrap()
    }

    #[instrument(skip(self))]
    async fn do_stroke(&mut self, start: bool, speed: Speed, settings: &LinearRange) -> ButtplugClientResult {
        let mut wait_ms = 0;
        for (i, actuator) in self.actuators.iter().enumerate() {
            let actual_settings = settings.merge(&self.settings[ i ].linear_or_max());
            wait_ms = actual_settings.get_duration_ms(speed);
            let target_pos = actual_settings.get_pos(start);
            debug!(?wait_ms, ?target_pos, ?settings, "stroke");
            self.worker_task_sender
                .send(WorkerTask::Move(
                    actuator.clone(),
                    target_pos,
                    wait_ms,
                    true,
                    self.result_sender.clone(),
                ))
                .unwrap_or_else(|err| error!("queue err {:?}", err));
        }
        // breaks with multiple devices that have different settings
        sleep(Duration::from_millis(wait_ms as u64)).await;
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

impl LinearRange {
    fn merge(&self, settings: &LinearRange) -> LinearRange {   
        LinearRange {
            min_ms: if self.min_ms < settings.min_ms { settings.min_ms } else { self.min_ms },
            max_ms: if self.max_ms > settings.max_ms { settings.max_ms } else { self.max_ms },
            min_pos: if self.min_pos < settings.min_pos { settings.min_pos } else { self.min_pos },
            max_pos: if self.max_pos > settings.max_pos { settings.max_pos } else { self.max_pos },
            invert: if settings.invert { ! self.invert } else { self.invert },
        }
    }
    pub fn get_pos(&self, move_up: bool) -> f64 {
        match move_up {
            true => if self.invert { 1.0 - self.max_pos } else { self.max_pos },
            false => if self.invert { 1.0 - self.min_pos } else { self.min_pos }
        }
    }
    pub fn get_duration_ms(&self, speed: Speed) -> u32 {
        let factor = (100 - speed.value) as f64 / 100.0;
        let ms = self.min_ms as f64 + (self.max_ms - self.min_ms) as f64 * factor;
        ms as u32
    }
}

fn apply_scalar_settings(speed: Speed, settings: &ActuatorSettings) -> Speed {
    if speed.value == 0 {
        return speed;
    }
    match settings {
        ActuatorSettings::Scalar(settings) => {
            trace!("applying {settings:?}");
            let speed = Speed::from_float(speed.as_float() * settings.factor);
            if speed.value < settings.min_speed as u16 {
                Speed::new(settings.min_speed)
            } else if speed.value > settings.max_speed as u16 {
                Speed::new(settings.max_speed)
            } else {
                speed
            }
        },
        _ => speed,
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
