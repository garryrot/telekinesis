use anyhow::Error;
use api::*;
use buttplug::client::ButtplugClientDevice;
use connection::{TkAction, TkConnectionEvent, TkConnectionStatus, TkDeviceStatus, TkStatus};
use pattern::{get_pattern_names, TkButtplugScheduler};
use settings::PATTERN_PATH;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{
    runtime::Runtime,
    sync::mpsc::{Sender, UnboundedReceiver, UnboundedSender},
};
use tracing::instrument;

use cxx::{CxxString, CxxVector};
use telekinesis::ERROR_HANDLE;

use crate::{
    input::{parse_list_string, read_input_string},
    settings::{TkConnectionType, TkSettings, SETTINGS_FILE, SETTINGS_PATH},
};

mod api;
mod connection;
mod fakes;
mod input;
mod logging;
mod papyrus;
mod pattern;
mod settings;
pub mod telekinesis;
mod tests;
mod util;

/// Methods exposed to as papyrus native functions
/// - All ffi methods are non-blocking, triggering an async action somewhere in the future
/// - All error conditions during the function call (i.e. mutex not available) will
///   be swallowed and logged to Telekinesis.log
/// - Uses a an abstract query/command engine to reduce coupling between the mod
///    functionality and the (rather tedious) `Plugin.cxx <-> Cxx <-> RustFFI` Sandwich
#[cxx::bridge]
mod ffi {
    extern "Rust" {
        type TkApi;
        fn tk_new() -> Box<TkApi>;
        fn tk_cmd(&mut self, cmd: &str) -> bool;
        fn tk_cmd_1(&mut self, cmd: &str, arg0: &str) -> bool;
        fn tk_cmd_2(&mut self, cmd: &str, arg0: &str, arg1: &str) -> bool;
        fn tk_qry_str(&mut self, qry: &str) -> String;
        fn tk_qry_str_1(&mut self, qry: &str, arg0: &str) -> String;
        fn tk_qry_lst(&mut self, qry: &str) -> Vec<String>;
        fn tk_qry_lst_1(&mut self, qry: &str, arg0: &str) -> Vec<String>;
        fn tk_qry_bool(&mut self, qry: &str) -> bool;
        fn tk_qry_bool_1(&mut self, qry: &str, arg0: &str) -> bool;
        fn tk_control(&mut self, qry: &str, arg0: i32, arg1: f32, arg2: &str, arg3: &CxxVector<CxxString>) -> i32;
        fn tk_stop(&mut self, arg0: i32) -> bool;
    }
}

type DeviceList = Vec<Arc<ButtplugClientDevice>>;

/// access to Telekinesis struct from within foreign rust modules and tests
/// TODO needed?
pub trait Tk {
    fn connect(settings: TkSettings) -> Result<Telekinesis, Error>;
    fn scan_for_devices(&self) -> bool;
    fn stop_scan(&self) -> bool;
    fn disconnect(&mut self);
    fn get_connection_status(&self) -> TkConnectionStatus;
    fn get_devices(&self) -> DeviceList;
    fn get_device(&self, device_name: &str) -> Option<Arc<ButtplugClientDevice>>;
    fn get_device_status(&self, device_name: &str) -> Option<TkDeviceStatus>;
    fn get_known_device_names(&self) -> Vec<String>;
    fn get_device_connection_status(&self, device_name: &str) -> TkConnectionStatus;
    fn get_device_capabilities(&self, device_name: &str) -> Vec<String>;
    fn vibrate(&mut self, speed: Speed, duration: TkDuration, events: Vec<String>) -> i32;
    fn vibrate_pattern(&mut self, pattern: TkPattern, events: Vec<String>) -> i32;
    fn stop(&mut self, handle: i32) -> bool;
    fn stop_all(&mut self) -> bool;
    fn get_next_event(&mut self) -> Option<TkConnectionEvent>;
    fn process_next_events(&mut self) -> Vec<TkConnectionEvent>;
    fn settings_set_enabled(&mut self, device_name: &str, enabled: bool);
    fn settings_set_events(&mut self, device_name: &str, events: Vec<String>);
    fn settings_get_events(&self, device_name: &str) -> Vec<String>;
    fn settings_get_enabled(&self, device_name: &str) -> bool;
}

pub struct Telekinesis {
    pub connection_status: Arc<Mutex<TkStatus>>,
    settings: TkSettings,
    runtime: Runtime,
    command_sender: Sender<TkAction>,
    scheduler: TkButtplugScheduler,
    connection_events: UnboundedReceiver<TkConnectionEvent>,
    event_sender: UnboundedSender<TkConnectionEvent>,
}

#[derive(Debug, Clone, Copy)]
pub struct Speed {
    pub value: u16,
}

#[derive(Clone, Debug)]
pub enum TkDuration {
    Infinite,
    Timed(Duration),
}

impl TkDuration {
    pub fn from_input_float(secs: f32) -> TkDuration {
        if secs > 0.0 {
            return TkDuration::Timed(Duration::from_millis((secs * 1000.0) as u64));
        } else {
            return TkDuration::Infinite;
        }
    }
    pub fn from_millis(ms: u64) -> TkDuration {
        TkDuration::Timed(Duration::from_millis(ms))
    }
    pub fn from_secs(s: u64) -> TkDuration {
        TkDuration::Timed(Duration::from_secs(s))
    }
}

#[derive(Clone, Debug)]
pub enum TkPattern {
    Linear(TkDuration, Speed),
    Funscript(TkDuration, String),
}

#[derive(Debug)]
pub struct TkApi {
    pub state: Arc<Mutex<Option<Telekinesis>>>,
}

fn tk_new() -> Box<TkApi> {
    Box::new(TkApi {
        state: Arc::new( Mutex::new( None ) )
    })
}

impl TkApi {
    #[instrument]
    fn tk_cmd(&mut self, cmd: &str) -> bool {
        self.exec_cmd_0(cmd)
    }

    #[instrument]
    fn tk_cmd_1(&mut self, cmd: &str, arg0: &str) -> bool {
        self.exec_cmd_1(cmd, arg0)
    }
    
    #[instrument]
    fn tk_cmd_2(&mut self, cmd: &str, arg0: &str, arg1: &str) -> bool {
        self.exec_cmd_2(cmd, arg0, arg1)
    }

    #[instrument]
    fn tk_qry_str(&mut self, qry: &str) -> String {
        self.exec_qry_str(qry)
    }

    #[instrument]
    fn tk_qry_str_1(&mut self, qry: &str, arg0: &str) -> String {
        self.exec_qry_str_1(qry, arg0)
    }

    #[instrument]
    fn tk_qry_lst(&mut self, qry: &str) -> Vec<String> {
        self.exec_qry_lst(qry)
    }

    #[instrument]
    fn tk_qry_lst_1(&mut self, qry: &str, arg0: &str) -> Vec<String> {
        self.exec_qry_lst_1(qry, arg0)
    }

    #[instrument]
    fn tk_qry_bool(&mut self, qry: &str) -> bool {
        self.exec_qry_bool(qry)
    }
    
    #[instrument]
    fn tk_qry_bool_1(&mut self, qry: &str, arg0: &str) -> bool {
        self.exec_qry_bool_1(qry, arg0)
    }

    #[instrument]
    fn tk_control(&mut self, qry: &str, arg0: i32, arg1: f32, arg2: &str, arg3: &CxxVector<CxxString>) -> i32 {
        self.exec_control(qry, arg0, arg1, arg2, arg3)
    }

    #[instrument]
    fn tk_stop(&mut self, arg0: i32) -> bool {
        self.exec_stop(arg0)
    }

    #[instrument]
    fn tk_destroy(&mut self) {
        self.destroy();
    }

}

impl Api<Telekinesis> for TkApi {
    fn init(&self) -> ApiInit<Telekinesis> {
        ApiInit {
            name: "connection.connect",
            exec: || {
                Telekinesis::connect(TkSettings::try_read_or_default(
                    SETTINGS_PATH,
                    SETTINGS_FILE,
                ))
            },
        }
    }

    fn cmd_0(&self) -> Vec<ApiCmd0<Telekinesis>> {
        vec![
            ApiCmd0 {
                name: "stop_all",
                exec: |tk| tk.stop_all(),
            },
            ApiCmd0 {
                name: "connection.start_scan",
                exec: |tk| tk.scan_for_devices(),
            },
            ApiCmd0 {
                name: "connection.stop_scan",
                exec: |tk| tk.stop_scan(),
            },
            ApiCmd0 {
                name: "settings.store",
                exec: |tk| tk.settings.try_write(SETTINGS_PATH, SETTINGS_FILE),
            },
            ApiCmd0 {
                name: "connection.inprocess",
                exec: |tk| {
                    tk.settings.connection = TkConnectionType::InProcess;
                    true
                },
            },
        ]
    }

    fn cmd_1(&self) -> Vec<ApiCmd1<Telekinesis>> {
        vec![
            ApiCmd1 {
                name: "connection.websocket",
                exec: |tk, value| {
                    tk.settings.connection = TkConnectionType::WebSocket(String::from(value));
                    true
                },
            },
            ApiCmd1 {
                name: "device.settings.enable",
                exec: |tk, device_name| {
                    tk.settings_set_enabled(device_name, true);
                    true
                },
            },
            ApiCmd1 {
                name: "device.settings.disable",
                exec: |tk, device_name| {
                    tk.settings_set_enabled(device_name, false);
                    true
                },
            },
        ]
    }

    fn cmd_2(&self) -> Vec<ApiCmd2<Telekinesis>> {
        vec![ApiCmd2 {
            name: "device.settings.events",
            exec: |tk, device_name, events| {
                tk.settings_set_events(device_name, parse_list_string(events));
                true
            },
        }]
    }

    fn qry_str(&self) -> Vec<ApiQryStr<Telekinesis>> {
        vec![ApiQryStr {
            name: "connection.status",
            default: "Not Connected",
            exec: |tk| {
                if let Ok(status) = tk.connection_status.try_lock() {
                    return status.connection_status.serialize_papyrus();
                }
                TkConnectionStatus::NotConnected.serialize_papyrus()
            },
        }]
    }

    fn qry_str_1(&self) -> Vec<ApiQryStr1<Telekinesis>> {
        vec![ApiQryStr1 {
            name: "device.connection.status",
            default: "Not Connected",
            exec: |tk, device_name| {
                if let Some(a) = tk.get_device_status(device_name) {
                    return a.status.serialize_papyrus();
                }
                TkConnectionStatus::NotConnected.serialize_papyrus()
            },
        }]
    }

    fn qry_lst(&self) -> Vec<ApiQryList<Telekinesis>> {
        vec![
            ApiQryList {
                name: "devices",
                exec: |tk| tk.get_known_device_names(),
            },
            ApiQryList {
                name: "events",
                exec: |tk| {
                    tk.process_next_events()
                        .iter()
                        .map(|evt| evt.serialize_papyrus())
                        .collect::<Vec<String>>()
                },
            },
            ApiQryList {
                name: "patterns.vibrator",
                exec: |_| get_pattern_names(PATTERN_PATH, true),
            },
            ApiQryList {
                name: "patterns.stroker",
                exec: |_| get_pattern_names(PATTERN_PATH, false),
            },
        ]
    }

    fn qry_lst_1(&self) -> Vec<ApiQryList1<Telekinesis>> {
        vec![
            ApiQryList1 {
                name: "device.settings.events",
                exec: |tk, device_name| tk.settings_get_events(device_name),
            },
            ApiQryList1 {
                name: "device.capabilities",
                exec: |tk, device_name| tk.get_device_capabilities(device_name),
            },
        ]
    }

    fn qry_bool(&self) -> Vec<ApiQryBool<Telekinesis>> {
        vec![]
    }

    fn qry_bool_1(&self) -> Vec<ApiQryBool1<Telekinesis>> {
        vec![ApiQryBool1 {
            name: "device.settings.enabled",
            exec: |tk, device_name| tk.settings_get_enabled(device_name),
        }]
    }

    fn control(&self) -> Vec<ApiControl<Telekinesis>> {
        vec![
            ApiControl {
                name: "vibrate",
                exec: |tk, speed, time_sec, _pattern_name, events| {
                    tk.vibrate(
                        Speed::new(speed.into()),
                        TkDuration::from_input_float(time_sec),
                        read_input_string(&events),
                    )
                },
                default: ERROR_HANDLE,
            },
            ApiControl {
                name: "vibrate.pattern",
                exec: |tk, _speed, time_sec, pattern_name, events| {
                    tk.vibrate_pattern(
                        TkPattern::Funscript(
                            TkDuration::from_input_float(time_sec),
                            String::from(pattern_name),
                        ),
                        read_input_string(&events),
                    )
                },
                default: ERROR_HANDLE,
            },
        ]
    }

    fn stop(&self) -> ApiStop<Telekinesis> {
        ApiStop { exec: |tk: &mut Telekinesis, handle| tk.stop(handle) }
    }

    fn state(&mut self) -> Arc<Mutex<Option<Telekinesis>>> {
        self.state.clone()
    }

    fn destroy(&mut self) -> ApiCmd0<Telekinesis> {
        ApiCmd0 {
            name: "connection.disconnect",
            exec: |tk| { tk.disconnect(); true },
        }
    }
}
