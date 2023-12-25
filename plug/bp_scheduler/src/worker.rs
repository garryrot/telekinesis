use buttplug::client::LinearCommand;
use std::{collections::HashMap, sync::Arc};

use tokio::sync::mpsc::UnboundedReceiver;
use tracing::{info, trace};
use tokio::sync::mpsc::UnboundedSender;

use crate::{access::DeviceAccess, actuator::Actuator, speed::Speed, ButtplugClientResult};

/// Process the queue of all device actions from all player threads
///
/// This was introduced so that that the housekeeping and the decision which
/// thread gets priority on a device is always done in the same thread and
/// its not necessary to introduce Mutex/etc to handle multithreaded access
pub struct ButtplugWorker {
    pub task_receiver: UnboundedReceiver<WorkerTask>,
}

#[derive(Clone, Debug)]
pub enum WorkerTask {
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

impl ButtplugWorker {
    pub async fn run_worker_thread(&mut self) {
        let mut device_access = DeviceAccess::default();
        loop {
            if let Some(next_action) = self.task_receiver.recv().await {
                trace!("exec device action {:?}", next_action);
                match next_action {
                    WorkerTask::Start(actuator, speed, is_not_pattern, handle) => {
                        device_access
                            .start_scalar(&actuator, speed, is_not_pattern, handle)
                            .await;
                    }
                    WorkerTask::Update(actuator, speed) => {
                        device_access.update_scalar(&actuator, speed).await;
                    }
                    WorkerTask::End(actuator, is_not_pattern, handle, result_sender) => {
                        let result = device_access
                            .stop_scalar(&actuator, is_not_pattern, handle)
                            .await;
                        result_sender.send(result).unwrap();
                    }
                    WorkerTask::Move(actuator, position, duration_ms, finish, result_sender) => {
                        let cmd = LinearCommand::LinearMap(HashMap::from([(
                            actuator.index_in_device,
                            (duration_ms, position),
                        )]));
                        let result = actuator.device.linear(&cmd).await;
                        if finish {
                            result_sender.send(result).unwrap();
                        }
                    }
                    WorkerTask::StopAll => {
                        device_access.clear_all();
                        info!("stop all action");
                    }
                }
            }
        }
    }
}