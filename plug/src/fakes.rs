use buttplug::core::connector::ButtplugConnectorResult;
use buttplug::core::message::{ActuatorType, ClientDeviceMessageAttributes};
use buttplug::server::device::configuration::{
    ServerDeviceMessageAttributesBuilder, ServerGenericDeviceMessageAttributes,
};
use serde::Serialize;
use tokio::sync::mpsc::channel;
use tokio::{sync::mpsc::Sender, time::sleep};

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
        ButtplugSpecV3ServerMessage, DeviceAdded,
    },
};
use futures::{future::BoxFuture, FutureExt};
use std::ops::{DerefMut, RangeInclusive};
use std::sync::Mutex;
use std::time::Duration;
use std::{collections::HashMap, sync::Arc};
use std::{thread, vec};
use tracing::error;

use crate::util::assert_timeout;
use crate::Telekinesis;
use std::time::Instant;

#[derive(Clone)]
pub struct FakeConnectorCallRegistry {
    pub actions: Arc<Mutex<HashMap<u32, Box<Vec<FakeMessage>>>>>,
}

#[derive(Clone)]
pub struct FakeMessage {
    pub message: ButtplugCurrentSpecClientMessage,
    pub time: Instant,
}

impl FakeMessage {
    pub fn new(msg: ButtplugCurrentSpecClientMessage) -> Self {
        FakeMessage {
            message: msg,
            time: Instant::now(),
        }
    }

    pub fn vibration_started(&self) -> bool {
        match self.message.clone() {
            message::ButtplugSpecV3ClientMessage::ScalarCmd(cmd) => {
                cmd.scalars().iter().any(|v| v.scalar() > 0.0)
            }
            _ => panic!("Message is not scalar cmd"),
        }
    }

    #[allow(dead_code)]
    pub fn vibration_started_strength(&self, speed: f64) -> bool {
        match self.message.clone() {
            message::ButtplugSpecV3ClientMessage::ScalarCmd(cmd) => {
                cmd.scalars().iter().any(|v| v.scalar() == speed)
            }
            _ => panic!("Message is not scalar cmd"),
        }
    }

    #[allow(dead_code)]
    pub fn get_scalar_strength(&self) -> f64 {
        match self.message.clone() {
            message::ButtplugSpecV3ClientMessage::ScalarCmd(cmd) => {
                cmd.scalars().iter().next().unwrap().scalar()
            }
            _ => panic!("Message is not scalar cmd"),
        }
    }

    pub fn vibration_stopped(&self) -> bool {
        match self.message.clone() {
            message::ButtplugSpecV3ClientMessage::ScalarCmd(cmd) => {
                cmd.scalars().iter().all(|v| v.scalar() == 0.0)
            }
            _ => panic!("Message is not scalar cmd"),
        }
    }
}

#[allow(dead_code)]
impl FakeConnectorCallRegistry {
    fn default() -> Self {
        Self {
            actions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn store_record<T>(&self, imp: &T, cmd: FakeMessage)
    where
        T: Serialize,
    {
        let mut calls = self.actions.try_lock().unwrap();
        let device_id = get_value(imp, "DeviceIndex").parse().unwrap();
        let mut bucket = match calls.get(&device_id) {
            Some(some) => some.clone(),
            None => Box::new(vec![]),
        };
        // let box_copy = *bucket.clone();
        bucket.deref_mut().push(cmd);
        calls.deref_mut().insert(device_id, bucket);
    }

    pub fn get_device(&self, device_id: u32) -> Vec<FakeMessage> {
        match self.actions.lock().unwrap().get(&device_id) {
            Some(some) => *some.clone(),
            None => vec![],
        }
    }

    pub fn assert_started(&self, device_id: u32) {
        assert_timeout!(self.get_device(device_id).len() == 1, "Device has vibrated");
        self.get_device(device_id)[0].vibration_started();
    }

    pub fn assert_vibrated(&self, device_id: u32) {
        assert_timeout!(self.get_device(device_id).len() >= 2, "Device has vibrated");
        self.get_device(device_id)[0].vibration_started();
        self.get_device(device_id)[1].vibration_stopped();
    }

    pub fn assert_not_vibrated(&self, device_id: u32) {
        thread::sleep(Duration::from_millis(100));
        assert_eq!(
            self.get_device(device_id).len(),
            0,
            "Device has not vibrated"
        );
    }
}

pub struct FakeDeviceConnector {
    pub devices: Vec<DeviceAdded>,
    server_outbound_sender: Sender<ButtplugCurrentSpecServerMessage>,
    call_registry: FakeConnectorCallRegistry,
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
                // assure that other thread has registered listener when the test devices
                // are added. Quick and dirty but its just test code anyways
                sleep(Duration::from_millis(100)).await;
                for device in devices {
                    if send
                        .send(ButtplugSpecV3ServerMessage::DeviceAdded(device))
                        .await
                        .is_err()
                    {
                        panic!();
                    }
                }
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
            ButtplugCurrentSpecClientMessage::RequestServerInfo(_) => async move {
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
            ButtplugCurrentSpecClientMessage::RequestDeviceList(_) => async move {
                let mut response: ButtplugSpecV3ServerMessage =
                    ButtplugSpecV3ServerMessage::DeviceList(DeviceList::new(vec![]));
                response.set_id(msg_id);
                sender
                    .send(response)
                    .await
                    .map_err(|_| ButtplugConnectorError::ConnectorNotConnected)
            }
            .boxed(),
            ButtplugCurrentSpecClientMessage::ScalarCmd(cmd) => {
                self.call_registry
                    .store_record(&cmd, FakeMessage::new(msg_clone));
                self.ok_response(msg_id)
            }
            ButtplugCurrentSpecClientMessage::LinearCmd(cmd) => {
                self.call_registry
                    .store_record(&cmd, FakeMessage::new(msg_clone));
                self.ok_response(msg_id)
            }
            ButtplugCurrentSpecClientMessage::RotateCmd(cmd) => {
                self.call_registry
                    .store_record(&cmd, FakeMessage::new(msg_clone));
                self.ok_response(msg_id)
            }
            ButtplugCurrentSpecClientMessage::StopAllDevices(_) => {
                // doesn't work cause no id
                // self.call_registry
                //     .store_record(&cmd, FakeMessage::new(msg_clone));
                self.ok_response(msg_id)
            }
            ButtplugCurrentSpecClientMessage::StartScanning(cmd) => {
                self.call_registry
                    .store_record(&cmd, FakeMessage::new(msg_clone));
                self.ok_response(msg_id)
            }
            ButtplugCurrentSpecClientMessage::StopScanning(cmd) => {
                self.call_registry
                    .store_record(&cmd, FakeMessage::new(msg_clone));
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

impl Telekinesis {
    /// should only be used by tests or fake backends
    pub fn await_connect(&self, devices: usize) {
        assert_timeout!(
            self.connection_status.lock().unwrap().device_status.len() == devices,
            "Awaiting connect"
        );
    }
}


#[cfg(test)]
pub mod tests {

    pub struct ButtplugTestClient {
        pub client: ButtplugClient, 
        pub call_registry: FakeConnectorCallRegistry,
        pub created_devices: Vec<Arc<ButtplugClientDevice>>,
    }    

    pub async fn get_test_client(devices: Vec<DeviceAdded>) -> ButtplugTestClient {
        let devices_len = devices.len();
        let mut created_devices = vec![];
    
        let (connector, call_registry) = FakeDeviceConnector::new(devices);
        let client = ButtplugClient::new("FakeClient");
        client.connect(connector).await.unwrap();
    
        while created_devices.len() < devices_len {
            let event = client.event_stream().next().await.unwrap();
            match event {
                buttplug::client::ButtplugClientEvent::DeviceAdded(device) => {
                    created_devices.push(device)
                }
                _ => {}
            }
        }
    
        ButtplugTestClient {
            client: client,
            call_registry,
            created_devices,
        }
    }

    use buttplug::{
        client::{
            ButtplugClient, ButtplugClientDevice, LinearCommand, RotateCommand, ScalarCommand,
        },
        core::message::ActuatorType,
    };
    use futures::{Future, StreamExt};
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

    pub fn await_devices(buttplug: &ButtplugClient, devices: usize) {
        assert_timeout!(
            buttplug.devices().len() == devices,
            "Awaiting devices connected"
        );
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
            await_devices(&buttplug, 3);

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
                call_registry.get_device(1).first().unwrap().message,
                ButtplugCurrentSpecClientMessage::ScalarCmd(..)
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
                call_registry.get_device(2).first().unwrap().message,
                ButtplugCurrentSpecClientMessage::LinearCmd(..)
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
                call_registry.get_device(2)[0].message,
                ButtplugCurrentSpecClientMessage::RotateCmd(..)
            ));
        });
    }

    async fn execute_test<Fut, F>(connector: FakeDeviceConnector, func: F) -> ButtplugClient
    where
        F: Fn(&Arc<ButtplugClientDevice>) -> Fut,
        Fut: Future,
    {
        let device_count = connector.devices.len();
        let buttplug = ButtplugClient::new("Foobar");
        buttplug.connect(connector).await.expect("connects");
        await_devices(&buttplug, device_count);
        for device in buttplug.devices().iter() {
            func(device).await;
        }
        buttplug
    }
}
