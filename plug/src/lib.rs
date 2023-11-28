use anyhow::Error;
use api::*;
use buttplug::client::ButtplugClientDevice;
use connection::{TkAction, TkConnectionEvent, TkConnectionStatus, TkDeviceStatus, TkStatus};
use pattern::{get_pattern_names, TkButtplugScheduler, TkPattern};
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
    pub fn as_us(&self) -> u64 {
        match self {
            TkDuration::Infinite => u64::MAX,
            TkDuration::Timed(duration) => duration.as_micros() as u64,
        }
    }
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

pub fn build_api() -> ApiBuilder<Telekinesis> {
    ApiBuilder::new(
        ApiInit {
            name: "connect",
            exec: || {
                Telekinesis::connect(TkSettings::try_read_or_default(
                    SETTINGS_PATH,
                    SETTINGS_FILE,
                ))
        },
    })
    // connection
    .def_cmd(ApiCmd0 {
        name: "connection.inprocess",
        exec: |tk| {
            tk.settings.connection = TkConnectionType::InProcess;
            true
        },
    })
    .def_cmd1(ApiCmd1 {
        name: "connection.websocket",
        exec: |tk, value| {
            tk.settings.connection = TkConnectionType::WebSocket(String::from(value));
            true
        },
    })
    .def_qry_str(ApiQryStr {
        name: "connection.status",
        default: "Not Connected",
        exec: |tk| {
            if let Ok(status) = tk.connection_status.try_lock() {
                return status.connection_status.serialize_papyrus();
            }
            TkConnectionStatus::NotConnected.serialize_papyrus()
        },
    })

    // scan
    .def_cmd(ApiCmd0 {
        name: "start_scan",
        exec: |tk| tk.scan_for_devices(),
    })
    .def_cmd(ApiCmd0 {
        name: "stop_scan",
        exec: |tk| tk.stop_scan(),
    })

    // status
    .def_qry_lst(ApiQryList { 
        name: "events",
        exec: |tk| {
            tk.process_next_events()
                .iter()
                .map(|evt| evt.serialize_papyrus())
                .collect::<Vec<String>>()
        },
    })

    // controls
    .def_control(ApiControl {
        name: "vibrate",
        exec: |tk, speed, time_sec, _pattern_name, events| {
            tk.vibrate(
                Speed::new(speed.into()),
                TkDuration::from_input_float(time_sec),
                read_input_string(&events),
            )
        },
        default: ERROR_HANDLE,
    })
    .def_control(ApiControl {
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
    })
    .def_stop(
        ApiStop { exec: |tk: &mut Telekinesis, handle| tk.stop(handle) }
    )
    .def_cmd(ApiCmd0 {
        name: "stop_all",
        exec: |tk| tk.stop_all(),
    })

    // settings
    .def_cmd(ApiCmd0 {
        name: "settings.store",
        exec: |tk| tk.settings.try_write(SETTINGS_PATH, SETTINGS_FILE),
    })

    // devices settings
    .def_qry_lst(ApiQryList {
        name: "devices",
        exec: |tk| tk.get_known_device_names(),
    })
    .def_qry_lst_1(ApiQryList1 {
        name: "device.capabilities",
        exec: |tk, device_name| tk.get_device_capabilities(device_name),
    })
    .def_cmd1(ApiCmd1 {
        name: "device.settings.enable",
        exec: |tk, device_name| {
            tk.settings_set_enabled(device_name, true);
            true
        },
    })
    .def_cmd1(ApiCmd1 {
        name: "device.settings.disable",
        exec: |tk, device_name| {
            tk.settings_set_enabled(device_name, false);
            true
        },
    })
    .def_qry_bool_1(ApiQryBool1 {
        name: "device.settings.enabled",
        exec: |tk, device_name| tk.settings_get_enabled(device_name),
    })
    .def_cmd2(ApiCmd2 {
        name: "device.settings.events",
        exec: |tk, device_name, events| {
            tk.settings_set_events(device_name, parse_list_string(events));
            true
        },
    })
    .def_qry_lst_1(ApiQryList1 {
        name: "device.settings.events",
        exec: |tk, device_name| tk.settings_get_events(device_name),
    })
    .def_qry_str1(ApiQryStr1 {
        name: "device.connection.status",
        default: "Not Connected",
        exec: |tk, device_name| {
            if let Some(a) = tk.get_device_status(device_name) {
                return a.status.serialize_papyrus();
            }
            TkConnectionStatus::NotConnected.serialize_papyrus()
        },
    })

    // patterns
    .def_qry_lst(ApiQryList {
        name: "patterns.vibrator",
        exec: |_| get_pattern_names(PATTERN_PATH, true),
    })
    .def_qry_lst(ApiQryList {
        name: "patterns.stroker",
        exec: |_| get_pattern_names(PATTERN_PATH, false),
    })
}

#[derive(Debug)]
pub struct TkApi {
    pub state: Arc<Mutex<Option<Telekinesis>>>,
}

impl Api<Telekinesis> for TkApi {
    fn state(&mut self) -> Arc<Mutex<Option<Telekinesis>>> {
        self.state.clone()
    }

    fn fns(&self) -> ApiBuilder<Telekinesis> {
        build_api()
    }
     fn destroy(&mut self) -> ApiCmd0<Telekinesis> {
         ApiCmd0 {
             name: "disconnect",
             exec: |tk| { tk.disconnect(); true },
         }
     }
}
