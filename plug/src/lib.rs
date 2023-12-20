use api::*;
use connection::{TkCommand, TkConnectionEvent, TkConnectionStatus, TkStatus};
use ffi::SKSEModEvent;
use input::get_duration_from_secs;
use itertools::Itertools;
use pattern::{Speed, TkButtplugScheduler, TkPattern};
use std::sync::{Arc, Mutex};
use tokio::{runtime::Runtime, sync::mpsc::Sender};
use tracing::instrument;

use cxx::{CxxString, CxxVector};
use telekinesis::{get_pattern_names, read_pattern, ERROR_HANDLE};

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
mod util;

/// Methods exposed to as papyrus native functions
/// - All ffi methods are non-blocking, triggering an async action somewhere in the future
/// - All error conditions during the function call (i.e. mutex not available) will
///   be swallowed and logged to Telekinesis.log
/// - Uses a an abstract query/command engine to reduce coupling between the mod
///    functionality and the (rather tedious) `Plugin.cxx <-> Cxx <-> RustFFI` Sandwich
#[cxx::bridge]
mod ffi {
    #[derive(Debug)]
    pub struct SKSEModEvent {
        pub event_name: String,
        pub str_arg: String,
        pub num_arg: f64,
    }

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
        fn tk_control(
            &mut self,
            qry: &str,
            arg0: i32,
            arg1: f32,
            arg2: &str,
            arg3: &CxxVector<CxxString>,
        ) -> i32;
        fn tk_stop(&mut self, arg0: i32) -> bool;
        // blocking
        fn tk_qry_nxt_evt(&mut self) -> Vec<SKSEModEvent>;
    }
}

pub struct Telekinesis {
    pub connection_status: Arc<Mutex<TkStatus>>,
    settings: TkSettings,
    runtime: Runtime,
    command_sender: Sender<TkCommand>,
    scheduler: TkButtplugScheduler,
    connection_events: crossbeam_channel::Receiver<TkConnectionEvent>,
    event_sender: crossbeam_channel::Sender<TkConnectionEvent>,
}

impl SKSEModEvent {
    pub fn new(event_name: &str, str_arg: &str, num_arg: f64) -> SKSEModEvent {
        SKSEModEvent {
            event_name: String::from(event_name),
            str_arg: String::from(str_arg),
            num_arg,
        }
    }

    pub fn from(event_name: &str, str_arg: &str) -> SKSEModEvent {
        SKSEModEvent {
            event_name: String::from(event_name),
            str_arg: String::from(str_arg),
            num_arg: 0.0,
        }
    }
}

fn tk_new() -> Box<TkApi> {
    Box::new(TkApi {
        state: Arc::new(Mutex::new(None)),
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
    fn tk_control(
        &mut self,
        qry: &str,
        arg0: i32,
        arg1: f32,
        arg2: &str,
        arg3: &CxxVector<CxxString>,
    ) -> i32 {
        self.exec_control(qry, arg0, arg1, arg2, arg3)
    }

    #[instrument]
    fn tk_stop(&mut self, arg0: i32) -> bool {
        self.exec_stop(arg0)
    }

    /// Return type Vec cause cxx does not support Option
    /// and Result enforces try catch
    #[instrument]
    fn tk_qry_nxt_evt(&mut self) -> Vec<SKSEModEvent> {
        let tele = &self.state();
        let mut receiver = None;
        if let Ok(mut guard) = tele.lock() {
            if let Some(tk) = guard.take() {
                let evt_receiver = tk.connection_events.clone();
                guard.replace(tk);
                receiver = Some(evt_receiver)
            }
        }
        match receiver {
            Some(receiver) => {
                if let Some(evt) = get_next_events_blocking(receiver) {
                    return vec![evt];
                }
                vec![]
            }
            None => vec![],
        }
    }

    #[instrument]
    fn tk_destroy(&mut self) {
        self.destroy();
    }
}

pub fn get_next_events_blocking(
    connection_events: crossbeam_channel::Receiver<TkConnectionEvent>,
) -> Option<SKSEModEvent> {
    if let Ok(result) = connection_events.recv() {
        let event = match result {
            TkConnectionEvent::Connected(connector) => {
                SKSEModEvent::from("Tele_Connected", &connector)
            }
            TkConnectionEvent::ConnectionFailure(err) => {
                SKSEModEvent::from("Tele_ConnectionError", &err)
            }
            TkConnectionEvent::DeviceAdded(device) => {
                SKSEModEvent::from("Tele_DeviceAdded", device.name())
            }
            TkConnectionEvent::DeviceRemoved(device) => {
                SKSEModEvent::from("Tele_DeviceRemoved", device.name())
            }
            TkConnectionEvent::ActionStarted(task, actuators, tags, handle) => {
                let str_arg = format!(
                    "{} {} on ({})",
                    task,
                    tags.iter().join(","),
                    actuators.iter().map(|x| x.identifier()).join(",")
                );
                SKSEModEvent::new("Tele_DeviceActionStarted", &str_arg, handle as f64)
            }
            TkConnectionEvent::ActionDone(task, duration, handle) => {
                let str_arg = format!("{} done after {:.1}s", task, duration.as_secs());
                SKSEModEvent::new("Tele_DeviceActionDone", &str_arg, handle as f64)
            }
            TkConnectionEvent::ActionError(_, err) => {
                SKSEModEvent::new("Tele_DeviceError", &err, 0.0)
            }
        };
        return Some(event);
    }
    None
}

pub fn build_api() -> ApiBuilder<Telekinesis> {
    ApiBuilder::new(ApiInit {
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
    // controls
    .def_control(ApiControl {
        name: "vibrate",
        exec: |tk, speed, time_sec, _pattern_name, events| {
            tk.vibrate(
                Speed::new(speed.into()),
                get_duration_from_secs(time_sec),
                read_input_string(events),
            )
        },
        default: ERROR_HANDLE,
    })
    .def_control(ApiControl {
        name: "vibrate.pattern",
        exec: |tk, _speed, time_sec, pattern_name, events| match read_pattern(
            &tk.settings.pattern_path,
            pattern_name,
            true,
        ) {
            Some(fscript) => tk.vibrate_pattern(
                TkPattern::Funscript(get_duration_from_secs(time_sec), Arc::new(fscript)),
                read_input_string(events),
                String::from(pattern_name),
            ),
            None => ERROR_HANDLE,
        },
        default: ERROR_HANDLE,
    })
    .def_stop(ApiStop {
        exec: |tk: &mut Telekinesis, handle| tk.stop(handle),
    })
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
        exec: |tk| get_pattern_names(&tk.settings.pattern_path, true),
    })
    .def_qry_lst(ApiQryList {
        name: "patterns.stroker",
        exec: |tk| get_pattern_names(&tk.settings.pattern_path, false),
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
            exec: |tk| {
                tk.disconnect();
                true
            },
        }
    }
}
