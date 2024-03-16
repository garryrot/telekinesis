use std::{
    fmt::{self, Display},
    sync::Arc,
    time::Duration,
};

use bp_scheduler::{
    actuator::{get_actuators, Actuator},
    speed::Speed,
};
use buttplug::{
    client::{ButtplugClient, ButtplugClientDevice, ButtplugClientEvent},
    core::message::ActuatorType,
};
use crossbeam_channel::Sender;
use futures::StreamExt;
use tokio::runtime::Handle;
use tracing::{debug, error, info};

use crate::*;

/// Global commands on connection level, i.e. connection handling
/// or emergency stop
#[derive(Clone, Debug)]
pub enum ConnectionCommand {
    Scan,
    StopScan,
    StopAll,
    Disconect,
}

#[derive(Clone, Debug)]
pub enum Task {
    Scalar(Speed),
    Pattern(Speed, ActuatorType, String),
    Linear(Speed, String),
    LinearOscillate(Speed, String),
}

#[derive(Clone, Debug)]
pub enum TkConnectionEvent {
    Connected(String),
    ConnectionFailure(String),
    DeviceAdded(Arc<ButtplugClientDevice>, Option<i32>),
    DeviceRemoved(Arc<ButtplugClientDevice>),
    BatteryLevel(Arc<ButtplugClientDevice>, Option<i32>),
    ActionStarted(Task, Vec<Arc<Actuator>>, Vec<String>, i32),
    ActionDone(Task, Duration, i32),
    ActionError(Arc<Actuator>, String),
}

pub async fn handle_connection(
    event_sender: crossbeam_channel::Sender<TkConnectionEvent>,
    event_sender_internal: crossbeam_channel::Sender<TkConnectionEvent>,
    mut command_receiver: tokio::sync::mpsc::Receiver<ConnectionCommand>,
    client: ButtplugClient,
    connection_type: TkConnectionType,
) {
    let sender_interla_clone = event_sender_internal.clone();
    let mut buttplug_events = client.event_stream();
    let sender_clone = event_sender.clone();
    Handle::current().spawn(async move {
        debug!("starting...");

        loop {
            let next_cmd = command_receiver.recv().await;
            if let Some(cmd) = next_cmd {
                debug!("Executing command {:?}", cmd);
                match cmd {
                    ConnectionCommand::Scan => {
                        if let Err(err) = client.start_scanning().await {
                            let error = err.to_string();
                            error!("connection failure {}", error);
                            let failure = TkConnectionEvent::ConnectionFailure(err.to_string());
                            try_send_event(&sender_clone, failure.clone());
                            try_send_event(&event_sender_internal, failure);
                        } else {
                            let settings = connection_type.to_string();
                            info!(settings, "connection success");

                            let connected = TkConnectionEvent::Connected(settings.clone());
                            try_send_event(&sender_clone, connected.clone());
                            try_send_event(&event_sender_internal, connected);
                        }
                    }
                    ConnectionCommand::StopScan => {
                        if let Err(err) = client.stop_scanning().await {
                            let error = err.to_string();
                            error!(error, "failed stop scan");
                            let err = TkConnectionEvent::ConnectionFailure(error);
                            try_send_event(&sender_clone, err.clone());
                            try_send_event(&event_sender_internal, err);
                        }
                    }
                    ConnectionCommand::Disconect => {
                        client
                            .disconnect()
                            .await
                            .unwrap_or_else(|_| error!("failed to disconnect"));
                        break;
                    }
                    ConnectionCommand::StopAll => {
                        client
                            .stop_all_devices()
                            .await
                            .unwrap_or_else(|_| error!("failed to stop all devices"));
                    }
                }
            } else {
                break;
            }
        }
        info!("stream closed");
    });

    while let Some(event) = buttplug_events.next().await {
        match event.clone() {
            ButtplugClientEvent::DeviceAdded(device) => {
                let name = device.name();
                let index = device.index();
                let actuators = get_actuators(vec![device.clone()]);
                info!(name, index, ?actuators, "device connected");

                let battery = if device.has_battery_level() {
                    device.battery_level().await.ok().map( |x| (x * 100.0) as i32 )
                } else {
                    None
                };
                let added = TkConnectionEvent::DeviceAdded(device, battery);
                try_send_event(&sender_interla_clone, added.clone());
                try_send_event(&event_sender, added);
            }
            ButtplugClientEvent::DeviceRemoved(device) => {
                let name = device.name();
                let index = device.index();
                info!(name, index, "device disconnected");

                let removed = TkConnectionEvent::DeviceRemoved(device);
                try_send_event(&sender_interla_clone, removed.clone());
                try_send_event(&event_sender, removed);
            }
            ButtplugClientEvent::Error(err) => {
                error!(?err, "client error event");
            }
            _ => {}
        };
    }
}

fn try_send_event(sender: &Sender<TkConnectionEvent>, evt: TkConnectionEvent) {
    sender
        .try_send(evt)
        .unwrap_or_else(|_| error!("event sender full"));
}

impl Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Task::Scalar(speed) => write!(f, "Constant({}%)", speed),
            Task::Pattern(speed, actuator, pattern) => {
                write!(f, "Pattern({}, {}, {})", speed, actuator, pattern)
            }
            Task::Linear(speed, pattern) => write!(f, "Linear({}, {})", speed, pattern),
            Task::LinearOscillate(speed, _) => write!(f, "Stroke({})", speed),
        }
    }
}
