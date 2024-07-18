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
use serde::{Deserialize, Serialize};
use tokio::runtime::Handle;
use tracing::{debug, error, info};

// use crate::*;

/// Global commands on connection level, i.e. connection handling
/// or emergency stop
#[derive(Clone, Debug)]
pub enum ConnectionCommand {
    Scan,
    StopScan,
    StopAll,
    Disconect,
    GetBattery
}

#[derive(Clone, Debug)]
pub enum Task {
    Scalar(Speed),
    Pattern(Speed, ActuatorType, String),
    Linear(Speed, String),
    LinearStroke(Speed, String),
}

#[derive(Clone, Debug)]
pub enum TkConnectionEvent {
    Connected(String),
    ConnectionFailure(String),
    DeviceAdded(Arc<ButtplugClientDevice>, Option<f64>),
    DeviceRemoved(Arc<ButtplugClientDevice>),
    BatteryLevel(Arc<ButtplugClientDevice>, Option<f64>),
    ActionStarted(Task, Vec<Arc<Actuator>>, Vec<String>, i32),
    ActionDone(Task, Duration, i32),
    ActionError(Arc<Actuator>, String),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum TkConnectionType {
    InProcess,
    WebSocket(String),
    Test,
}

impl Display for TkConnectionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TkConnectionType::InProcess => write!(f, "In-Process"),
            TkConnectionType::WebSocket(host) => write!(f, "WebSocket {}", host),
            TkConnectionType::Test => write!(f, "Test"),
        }
    }
}

pub async fn handle_connection(
    event_sender: crossbeam_channel::Sender<TkConnectionEvent>,
    event_sender_internal: crossbeam_channel::Sender<TkConnectionEvent>,
    command_sender: tokio::sync::mpsc::Sender<ConnectionCommand>, // TODO: just use crossbeam?
    mut command_receiver: tokio::sync::mpsc::Receiver<ConnectionCommand>,
    client: ButtplugClient,
    connection_type: TkConnectionType,
) {
    let sender_interla_clone = event_sender_internal.clone();
    let mut buttplug_events = client.event_stream();
    let sender_clone = event_sender.clone();
    let try_send_events = move |event: TkConnectionEvent| {
        try_send_event(&sender_clone, event.clone());
        try_send_event(&event_sender_internal, event);
    };
    Handle::current().spawn(async move {
        debug!("starting connection thread...");
        loop {
            let next_cmd = command_receiver.recv().await;
            if let Some(cmd) = next_cmd {
                debug!("Executing command {:?}", cmd);
                match cmd {
                    ConnectionCommand::Scan => {
                        if let Err(err) = client.start_scanning().await {
                            let error = err.to_string();
                            error!("connection failure {}", error);
                            try_send_events(TkConnectionEvent::ConnectionFailure(err.to_string()));
                        } else {
                            let settings = connection_type.to_string();
                            info!(settings, "connection success");
                            try_send_events(TkConnectionEvent::Connected(settings.clone()));
                        }
                    }
                    ConnectionCommand::StopScan => {
                        if let Err(err) = client.stop_scanning().await {
                            let error = err.to_string();
                            error!(error, "failed stop scan");
                            let err = TkConnectionEvent::ConnectionFailure(error);
                            try_send_events(err);
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
                    ConnectionCommand::GetBattery => {
                        for device in client.devices() {
                            if device.connected() && device.has_battery_level() {
                                try_send_events(TkConnectionEvent::BatteryLevel(device.clone(),device.battery_level().await.ok()));
                            }
                        }
                    },
                }
            } else {
                break;
            }
        }
        info!("stream closed");
    });

    Handle::current().spawn(async move {
        debug!("starting battery thread");
        loop {
            tokio::time::sleep(Duration::from_secs(300)).await;
            let _ = command_sender.send(ConnectionCommand::GetBattery).await;
        }
    });

    while let Some(event) = buttplug_events.next().await {
        match event.clone() {
            ButtplugClientEvent::DeviceAdded(device) => {
                let name = device.name();
                let index = device.index();
                let actuators = get_actuators(vec![device.clone()]);
                info!(name, index, ?actuators, "device connected");
                let battery = if device.has_battery_level() {
                    device.battery_level().await.ok()
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
            Task::LinearStroke(speed, _) => write!(f, "Stroke({})", speed),
        }
    }
}
