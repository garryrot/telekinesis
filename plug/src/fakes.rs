use buttplug::core::connector::ButtplugConnectorResult;
use buttplug::core::message::{ActuatorType, ClientDeviceMessageAttributes};
use buttplug::server::device::configuration::{
    ServerDeviceMessageAttributesBuilder, ServerGenericDeviceMessageAttributes,
};
use lazy_static::__Deref;
use serde::Serialize;
use tokio::sync::mpsc::channel;
use tokio::{
    sync::mpsc::Sender,
    time::{sleep, Duration},
};

use buttplug::{
    core::message::{self, ButtplugMessage, DeviceList},
    core::message::{ButtplugMessageSpecVersion, ServerInfo},
    util::async_manager,
};
use serde_json::{self, Value};

use buttplug::core::{
    connector::{ButtplugConnector, ButtplugConnectorError},
    message::{
        ButtplugCurrentSpecClientMessage, ButtplugCurrentSpecServerMessage,
        ButtplugSpecV3ClientMessage, ButtplugSpecV3ServerMessage, DeviceAdded,
    },
};
use futures::{future::BoxFuture, lock::Mutex, FutureExt};
use std::ops::{DerefMut, RangeInclusive};
use std::vec;
use std::{collections::HashMap, sync::Arc};
use tracing::error;

#[derive(Clone)]
pub struct FakeConnectorCallRegistry {
    pub actions: Arc<Mutex<HashMap<u32, Box<Vec<ButtplugCurrentSpecClientMessage>>>>>,
}

pub struct FakeDeviceConnector {
    pub devices: Vec<DeviceAdded>,
    server_outbound_sender: Sender<ButtplugCurrentSpecServerMessage>,
    call_registry: FakeConnectorCallRegistry,
}

#[allow(dead_code)]
impl FakeConnectorCallRegistry {
    fn default() -> Self {
        Self {
            actions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn store_record<T>(&self, imp: &T, cmd: ButtplugSpecV3ClientMessage)
    where
        T: Serialize,
    {
        let mut calls = self.actions.try_lock().unwrap();
        let device_id = get_value(imp, "DeviceIndex").parse().unwrap();
        let mut bucket = match calls.deref().get(&device_id) {
            Some(some) => some.clone(),
            None => Box::new(vec![]),
        };
        bucket.deref_mut().push(cmd);
        calls.deref_mut().insert(device_id, bucket);
    }

    pub fn get_record(&self, device_id: u32) -> Vec<ButtplugSpecV3ClientMessage> {
        match self.actions.try_lock().unwrap().deref().get(&device_id) {
            Some(some) => *some.clone(),
            None => vec![],
        }
    }
}

// Connector that allows to instantiate various fake devices for testing purposes
#[allow(dead_code)]
impl FakeDeviceConnector {
    pub fn new(devices: Vec<DeviceAdded>) -> (Self, FakeConnectorCallRegistry) {
        let (server_outbound_sender, _) = channel(256);
        let connector = FakeDeviceConnector {
            devices: devices,
            server_outbound_sender: server_outbound_sender,
            call_registry: FakeConnectorCallRegistry::default(),
        };
        let calls = connector.get_call_registry();
        (connector, calls)
    }

    // A demo configuration that exposes various devices
    pub fn device_demo() -> (Self, FakeConnectorCallRegistry) {
        Self::new(vec![
            vibrator(1, "Vibator 1"),
            vibrator(2, "Vibrator 2"),
            vibrator(3, "Vibrator 3"),
            linear(4, "Linear 1"),
            linear(5, "Linear 2"),
            linear(6, "Linear 3"),
            rotate(7, "Rotator 1"),
        ])
    }

    pub fn get_call_registry(&self) -> FakeConnectorCallRegistry {
        self.call_registry.clone()
    }

    fn ok_response(&self, msg_id: u32) -> buttplug::core::connector::ButtplugConnectorResultFuture {
        let sender = self.server_outbound_sender.clone();
        async move {
            let mut response = ButtplugSpecV3ServerMessage::Ok(message::Ok::default());
            response.set_id(msg_id);
            sender
                .send(response)
                .await
                .map_err(|_| ButtplugConnectorError::ConnectorNotConnected)
        }
        .boxed()
    }
}

impl ButtplugConnector<ButtplugCurrentSpecClientMessage, ButtplugCurrentSpecServerMessage>
    for FakeDeviceConnector
{
    fn connect(
        &mut self,
        message_sender: tokio::sync::mpsc::Sender<ButtplugCurrentSpecServerMessage>,
    ) -> BoxFuture<'static, Result<(), ButtplugConnectorError>> {
        let devices = self.devices.clone();
        let send = message_sender.clone();
        self.server_outbound_sender = message_sender.clone();
        async move {
            async_manager::spawn(async move {
                for device in devices {
                    if send
                        .send(ButtplugSpecV3ServerMessage::DeviceAdded(device))
                        .await
                        .is_err()
                    {
                        panic!();
                    }
                }
                sleep(Duration::from_millis(1)).await;
            });
            Ok(())
        }
        .boxed()
    }

    fn disconnect(&self) -> buttplug::core::connector::ButtplugConnectorResultFuture {
        async move { ButtplugConnectorResult::Ok(()) }.boxed()
    }

    fn send(
        &self,
        msg: ButtplugCurrentSpecClientMessage,
    ) -> buttplug::core::connector::ButtplugConnectorResultFuture {
        let msg_id = msg.id();
        let msg_clone = msg.clone();
        let sender = self.server_outbound_sender.clone();
        match msg {
            ButtplugSpecV3ClientMessage::RequestServerInfo(_) => async move {
                sender
                    .send(ButtplugSpecV3ServerMessage::ServerInfo(ServerInfo::new(
                        "test server",
                        ButtplugMessageSpecVersion::Version3,
                        0,
                    )))
                    .await
                    .map_err(|_| ButtplugConnectorError::ConnectorNotConnected)
            }
            .boxed(),
            ButtplugSpecV3ClientMessage::RequestDeviceList(_) => async move {
                let mut response: ButtplugSpecV3ServerMessage =
                    ButtplugSpecV3ServerMessage::DeviceList(DeviceList::new(vec![]));
                response.set_id(msg_id);
                sender
                    .send(response)
                    .await
                    .map_err(|_| ButtplugConnectorError::ConnectorNotConnected)
            }
            .boxed(),
            ButtplugSpecV3ClientMessage::ScalarCmd(cmd) => {
                self.call_registry.store_record(&cmd, msg_clone);
                self.ok_response(msg_id)
            }
            ButtplugSpecV3ClientMessage::LinearCmd(cmd) => {
                self.call_registry.store_record(&cmd, msg_clone);
                self.ok_response(msg_id)
            }
            ButtplugSpecV3ClientMessage::RotateCmd(cmd) => {
                self.call_registry.store_record(&cmd, msg_clone);
                self.ok_response(msg_id)
            }
            ButtplugSpecV3ClientMessage::StopAllDevices(cmd) => {
                self.call_registry.store_record(&cmd, msg_clone);
                self.ok_response(msg_id)
            }
            ButtplugSpecV3ClientMessage::StartScanning(cmd) => {
                self.call_registry.store_record(&cmd, msg_clone);
                self.ok_response(msg_id)
            }
            ButtplugSpecV3ClientMessage::StopScanning(cmd) => {
                self.call_registry.store_record(&cmd, msg_clone);
                self.ok_response(msg_id)
            }
            _ => {
                error!("Unimplemented message type.");
                async move { ButtplugConnectorResult::Ok(()) }.boxed()
            }
        }
    }
}

fn get_value<T>(val: &T, key: &str) -> String
where
    T: Serialize,
{
    let value: Value = serde_json::from_str(&serde_json::to_string(val).unwrap()).unwrap();
    value[key].to_string().parse().unwrap()
}

#[allow(dead_code)]
pub fn vibrator(id: u32, name: &str) -> DeviceAdded {
    let attributes = ServerDeviceMessageAttributesBuilder::default()
        .scalar_cmd(&vec![ServerGenericDeviceMessageAttributes::new(
            &format!("Vibrator {}", id),
            &RangeInclusive::new(0, 10),
            ActuatorType::Vibrate,
        )])
        .finish();
    DeviceAdded::new(
        id,
        name,
        &None,
        &None,
        &ClientDeviceMessageAttributes::from(attributes),
    )
}

#[allow(dead_code)]
pub fn scalar(id: u32, name: &str, actuator: ActuatorType) -> DeviceAdded {
    let attributes = ServerDeviceMessageAttributesBuilder::default()
        .scalar_cmd(&vec![ServerGenericDeviceMessageAttributes::new(
            &format!("Vibrator {}", id),
            &RangeInclusive::new(0, 10),
            actuator,
        )])
        .finish();
    DeviceAdded::new(
        id,
        name,
        &None,
        &None,
        &ClientDeviceMessageAttributes::from(attributes),
    )
}

#[allow(dead_code)]
pub fn linear(id: u32, name: &str) -> DeviceAdded {
    let attributes = ServerDeviceMessageAttributesBuilder::default()
        .linear_cmd(&vec![ServerGenericDeviceMessageAttributes::new(
            &format!("Oscillator {}", id),
            &RangeInclusive::new(0, 10),
            ActuatorType::Oscillate,
        )])
        .finish();
    DeviceAdded::new(
        id,
        name,
        &None,
        &None,
        &ClientDeviceMessageAttributes::from(attributes),
    )
}

#[allow(dead_code)]
pub fn rotate(id: u32, name: &str) -> DeviceAdded {
    let attributes = ServerDeviceMessageAttributesBuilder::default()
        .rotate_cmd(&vec![ServerGenericDeviceMessageAttributes::new(
            &format!("Rotator {}", id),
            &RangeInclusive::new(0, 10),
            ActuatorType::Rotate,
        )])
        .finish();
    DeviceAdded::new(
        id,
        name,
        &None,
        &None,
        &ClientDeviceMessageAttributes::from(attributes),
    )
}

#[cfg(test)]
mod tests {
    use buttplug::{
        client::{
            ButtplugClient, ButtplugClientDevice, LinearCommand, RotateCommand, ScalarCommand,
        },
        core::message::ActuatorType,
    };
    use futures::Future;
    use tracing::Level;

    use super::*;

    #[allow(dead_code)]
    fn enable_log() {
        tracing::subscriber::set_global_default(
            tracing_subscriber::fmt()
                .with_max_level(Level::DEBUG)
                .finish(),
        )
        .unwrap();
    }

    #[test]
    fn adding_test_devices_works() {
        async_manager::block_on(async {
            // arrange
            let buttplug = ButtplugClient::new("Foobar");
            let (connector, _) = FakeDeviceConnector::new(vec![
                vibrator(1, "eins"),
                vibrator(2, "zwei"),
                vibrator(3, "drei"),
            ]);

            // act
            buttplug.connect(connector).await.expect("connects");

            // assert
            assert_eq!(
                buttplug
                    .devices()
                    .iter()
                    .filter(|x| x.index() == 1)
                    .next()
                    .unwrap()
                    .name(),
                "eins"
            );
            assert_eq!(
                buttplug
                    .devices()
                    .iter()
                    .filter(|x| x.index() == 2)
                    .next()
                    .unwrap()
                    .name(),
                "zwei"
            );
            assert_eq!(
                buttplug
                    .devices()
                    .iter()
                    .filter(|x| x.index() == 3)
                    .next()
                    .unwrap()
                    .name(),
                "drei"
            );
            ()
        });
    }

    #[test]
    fn call_registry_stores_vibrate() {
        async_manager::block_on(async {
            // arrange
            let (connector, call_registry) =
                FakeDeviceConnector::new(vec![vibrator(1, "vibrator"), linear(2, "oscillator")]);

            // act
            execute_test(connector, |x| {
                x.scalar(&ScalarCommand::Scalar((1.0, ActuatorType::Vibrate)))
            })
            .await;

            // assert
            assert!(matches!(
                call_registry.get_record(1).first().unwrap(),
                ButtplugSpecV3ClientMessage::ScalarCmd(..)
            ));
        });
    }

    #[test]
    fn call_registry_stores_linear() {
        async_manager::block_on(async {
            // arrange
            let (connector, call_registry) =
                FakeDeviceConnector::new(vec![vibrator(1, "vibrator"), linear(2, "oscillator")]);

            // act
            execute_test(connector, |x| x.linear(&LinearCommand::Linear(88, 0.9))).await;

            // asert
            assert!(matches!(
                call_registry.get_record(2).first().unwrap(),
                ButtplugSpecV3ClientMessage::LinearCmd(..)
            ));
        });
    }

    #[test]
    fn call_registry_stores_rotate() {
        async_manager::block_on(async {
            // arrange
            let (connector, call_registry) =
                
                FakeDeviceConnector::new(vec![vibrator(1, "vibrator"), rotate(2, "rotator")]);

            // act
            execute_test(connector, |x| x.rotate(&RotateCommand::Rotate(0.9, true))).await;

            // asert
            assert!(matches!(
                call_registry.get_record(2)[0],
                ButtplugSpecV3ClientMessage::RotateCmd(..)
            ));
        });
    }

    async fn execute_test<Fut, F>(connector: FakeDeviceConnector, func: F) -> ButtplugClient
    where
        F: Fn(&Arc<ButtplugClientDevice>) -> Fut,
        Fut: Future,
    {
        let buttplug = ButtplugClient::new("Foobar");
        buttplug.connect(connector).await.expect("connects");
        for device in buttplug.devices().iter() {
            func(device).await;
        }
        buttplug
    }
}
