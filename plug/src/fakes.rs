use serde::Serialize;
use tokio::{time::{sleep, Duration}, sync::mpsc::Sender};
use std::ops::RangeInclusive;
use tokio::{sync::mpsc::channel};

use buttplug::{
    util::async_manager, 
    core::message::{
        ServerInfo, ButtplugMessageSpecVersion}, 
        core::message::{DeviceList, ButtplugMessage, self}, 
        server::{device::configuration::{
            ServerDeviceMessageAttributesBuilder,
            ServerGenericDeviceMessageAttributes
        }}
    };
use serde_json::{self, Value};

use std::{collections::HashMap, sync::Arc};
use buttplug::{core::{connector::{ButtplugConnector, ButtplugConnectorError, ButtplugConnectorResult}, message::{ButtplugCurrentSpecClientMessage, ButtplugCurrentSpecServerMessage, DeviceAdded, ClientDeviceMessageAttributes, ButtplugSpecV3ClientMessage, ButtplugSpecV3ServerMessage}}};
use futures::{FutureExt, future::BoxFuture, lock::Mutex};

#[derive(Clone)]
pub struct FakeConnectorCallRegistry
{
    pub actions: Arc<Mutex<HashMap<u32, ButtplugCurrentSpecClientMessage>>>
}

pub struct FakeDeviceConnector {
    pub devices: Vec<DeviceAdded>,
    server_outbound_sender: Sender<ButtplugCurrentSpecServerMessage>,
    call_registry: FakeConnectorCallRegistry
}

impl FakeConnectorCallRegistry {
    fn default() -> Self {
        Self {
            actions: Arc::new(Mutex::new(HashMap::new()))
        }
    }

    pub fn store_record<T>( &self, imp: &T, cmd: ButtplugSpecV3ClientMessage )
        where T : Serialize
    {
        let values = parse_message( imp );
        let mut calls = self.actions.try_lock().unwrap();
        (*calls).insert(values["DeviceIndex"].to_string().parse().unwrap(), cmd);
    }

    pub fn get_record( &self, device_id: u32 ) -> Option<ButtplugSpecV3ClientMessage> {
        let calls = self.actions.try_lock().unwrap();
        let content = (*calls).get( &device_id );
        if None == content {
            None
        }
        else {
            Some(content.unwrap().clone())
        }
    }

}

impl FakeDeviceConnector {
    pub fn new( devices: Vec<DeviceAdded> ) -> Self {
        let (server_outbound_sender, _) = channel(256);
        FakeDeviceConnector {
            devices: devices,
            server_outbound_sender: server_outbound_sender,
            call_registry: FakeConnectorCallRegistry::default()
        }
    }
    pub fn get_call_registry( &self ) -> FakeConnectorCallRegistry {
        self.call_registry.clone()
    }
}

impl ButtplugConnector<ButtplugCurrentSpecClientMessage, ButtplugCurrentSpecServerMessage>
  for FakeDeviceConnector {
    fn connect( &mut self, message_sender: tokio::sync::mpsc::Sender<ButtplugCurrentSpecServerMessage>, )
            -> BoxFuture<'static, Result<(), ButtplugConnectorError>>  { 

        let devices = self.devices.clone();
        let send = message_sender.clone();
        self.server_outbound_sender = message_sender.clone();
        async move {
            async_manager::spawn(async move {
                for device in devices {
                    if send.send(ButtplugSpecV3ServerMessage::DeviceAdded(device)).await.is_err() {
                        panic!();
                    }
                }
                sleep(Duration::from_millis(1)).await;
            });
            Ok(())
        }.boxed()
    }

    fn disconnect(&self) -> buttplug::core::connector::ButtplugConnectorResultFuture {
        todo!();
    }

    fn send(&self, msg: ButtplugCurrentSpecClientMessage) -> buttplug::core::connector::ButtplugConnectorResultFuture {
        let msg_id = msg.id();
        let msg_clone = msg.clone();
        let sender = self.server_outbound_sender.clone();
        match msg 
        {
            ButtplugSpecV3ClientMessage::RequestServerInfo(_) => {
                async move { 
                    sender.send(
                        ButtplugSpecV3ServerMessage::ServerInfo(
                            ServerInfo::new("test server", ButtplugMessageSpecVersion::Version3, 0)
                        )
                    ).await.map_err(|_| ButtplugConnectorError::ConnectorNotConnected)
                }.boxed()
            },
            ButtplugSpecV3ClientMessage::RequestDeviceList(_) => {
                async move { 
                    let mut response: ButtplugSpecV3ServerMessage = ButtplugSpecV3ServerMessage::DeviceList(DeviceList::new(vec![]));
                    response.set_id(msg_id);  
                    sender.send(response).await.map_err(|_| ButtplugConnectorError::ConnectorNotConnected)
                }.boxed()
            },
            ButtplugSpecV3ClientMessage::ScalarCmd(cmd) => {
                self.call_registry.store_record(&cmd, msg_clone);
                async move {
                    let mut response = ButtplugSpecV3ServerMessage::Ok(message::Ok::default());
                    response.set_id(msg_id);
                    sender.send(response).await.map_err(|_| ButtplugConnectorError::ConnectorNotConnected)
                }.boxed()
            },
            // ButtplugSpecV3ClientMessage::ScalarCmd(_) => {},
            // ButtplugSpecV3ClientMessage::LinearCmd(_) => {},
            // ButtplugSpecV3ClientMessage::RotateCmd(_) => {},
            // ButtplugSpecV3ClientMessage::StopAllDevices(_) => {},
            // ButtplugSpecV3ClientMessage::StartScanning(_) => {},
            // ButtplugSpecV3ClientMessage::StopScanning(_) => {}
            _ => { 
                async move { 
                    ButtplugConnectorResult::Ok(())
                }.boxed()
            }
        }
    }
}

fn parse_message<T>( val: &T ) -> Value
    where T : Serialize {
    serde_json::from_str(&serde_json::to_string(val).unwrap()).unwrap()
}

#[cfg(test)]
mod tests {
    use buttplug::{client::{ButtplugClient, ScalarCommand}, core::message::{ActuatorType}};
    use tracing::Level;

    use super::*;
    
    fn vibrator( id: u32, name: &str) -> DeviceAdded {
        let attributes = ServerDeviceMessageAttributesBuilder::default()
            .scalar_cmd(& vec![
                ServerGenericDeviceMessageAttributes::new(
                    &format!("Vibrator {}", id ), 
                    &RangeInclusive::new(0,10), 
                    ActuatorType::Vibrate)
            ])
            .finish();
        let client_device_attributes = ClientDeviceMessageAttributes::from(attributes);
        DeviceAdded::new(
            id,
            name, 
            &None, 
            &None, 
            &client_device_attributes)
    }

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
            let connector = FakeDeviceConnector::new(vec![
                vibrator(1, "eins"),
                vibrator(2, "zwei"),
                vibrator(3, "drei"),
            ]);
            
            // act
            buttplug.connect(connector).await.expect("connects");

            // assert
            assert_eq!( buttplug.devices().iter().filter( |x| x.index() == 1 ).next().unwrap().name(), "eins");
            assert_eq!( buttplug.devices().iter().filter( |x| x.index() == 2 ).next().unwrap().name(), "zwei");
            assert_eq!( buttplug.devices().iter().filter( |x| x.index() == 3 ).next().unwrap().name(), "drei");
            ()
        });
    }

    #[test]
    fn call_registry_stored_messages() {
        async_manager::block_on(async {
            // arrange
            let connector = FakeDeviceConnector::new(vec![
                vibrator(1, "eins")
            ]);
            let call_registry = connector.get_call_registry();
    
            // act
            let buttplug = ButtplugClient::new("Foobar");
            buttplug.connect(connector).await.expect("connects");
            let _ = buttplug.devices()[0].scalar(&ScalarCommand::Scalar((1.0, ActuatorType::Vibrate))).await;
    
            // assert
            let record = call_registry.get_record(1);
            assert!( record.is_some() );
            assert!( matches!( record.unwrap(), ButtplugSpecV3ClientMessage::ScalarCmd(..)) );
        });
    }
}
