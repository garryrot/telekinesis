
use std::sync::{Arc, Mutex};
use itertools::Itertools;

use tracing::{
    instrument,
    info
};

use cxx::{CxxString, CxxVector};

use buttplug::core::message::ActuatorType;

use bp_scheduler::{
    settings::*, 
    speed::*
};

use ffi::SKSEModEvent;
use crate::{
    api::*,
    pattern::*,
    input::*,
    telekinesis::*,
    connection::*,
    settings::*,
};

mod api;
mod connection;
mod input;
mod logging;
mod pattern;
mod settings;
mod status;
pub mod telekinesis;

#[derive(Debug)]
pub struct TkApi {
    pub state: Arc<Mutex<Option<Telekinesis>>>,
}

/// Methods exposed as papyrus native functions
/// - Uses a an abstract query/command engine to reduce coupling between the mod
///    functionality and the (rather tedious) `Plugin.cxx <-> Cxx <-> RustFFI` Sandwich
///    basically, I don't want to change 5 method signatures whenever one of those methods changes
/// - All ffi methods except  are non-blocking, triggering an async action somewhere in the future
/// - All error conditions during the function call (i.e. mutex not available) will
///   be swallowed and logged to Telekinesis.log
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
        fn tk_update(&mut self, arg0: i32, arg1: i32) -> bool;
        fn tk_stop(&mut self, arg0: i32) -> bool;
        // blocking
        fn tk_qry_nxt_evt(&mut self) -> Vec<SKSEModEvent>;
    }
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

impl TkApi {
    #[instrument(skip(self))]
    fn tk_cmd(&mut self, cmd: &str) -> bool {
        self.exec_cmd_0(cmd)
    }

    #[instrument(skip(self))]
    fn tk_cmd_1(&mut self, cmd: &str, arg0: &str) -> bool {
        self.exec_cmd_1(cmd, arg0)
    }

    #[instrument(skip(self))]
    fn tk_cmd_2(&mut self, cmd: &str, arg0: &str, arg1: &str) -> bool {
        self.exec_cmd_2(cmd, arg0, arg1)
    }

    #[instrument(skip(self))]
    fn tk_qry_str(&mut self, qry: &str) -> String {
        self.exec_qry_str(qry)
    }

    #[instrument(skip(self))]
    fn tk_qry_str_1(&mut self, qry: &str, arg0: &str) -> String {
        self.exec_qry_str_1(qry, arg0)
    }

    #[instrument(skip(self))]
    fn tk_qry_lst(&mut self, qry: &str) -> Vec<String> {
        self.exec_qry_lst(qry)
    }

    #[instrument(skip(self))]
    fn tk_qry_lst_1(&mut self, qry: &str, arg0: &str) -> Vec<String> {
        self.exec_qry_lst_1(qry, arg0)
    }

    #[instrument(skip(self))]
    fn tk_qry_bool(&mut self, qry: &str) -> bool {
        self.exec_qry_bool(qry)
    }

    #[instrument(skip(self))]
    fn tk_qry_bool_1(&mut self, qry: &str, arg0: &str) -> bool {
        self.exec_qry_bool_1(qry, arg0)
    }

    #[instrument(skip(self))]
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

    #[instrument(skip(self))]
    fn tk_update(&mut self, arg0: i32, arg1: i32) -> bool {
        self.exec_update(arg0, arg1)
    }

    #[instrument(skip(self))]
    fn tk_stop(&mut self, arg0: i32) -> bool {
        self.exec_stop(arg0)
    }

    /// Return type Vec cause cxx crate does not support Option
    /// and Result enforces try catch with some weird template
    /// I don't wanna get into
    fn tk_qry_nxt_evt(&mut self) -> Vec<SKSEModEvent> {
        let tele = &self.state();
        let mut receiver = None;
        if let Ok(mut guard) = tele.lock() {
            if let Some(tk) = guard.take() {
                let evt_receiver = tk.connection_events.clone();
                guard.replace(tk);
                receiver = Some(evt_receiver);
            }
        }
        match receiver {
            Some(receiver) => {
                if let Some(evt) = get_next_events_blocking(&receiver) {
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

pub fn get_next_events_blocking(connection_events: &crossbeam_channel::Receiver<TkConnectionEvent>,) -> Option<SKSEModEvent> {
    if let Ok(result) = connection_events.recv() {
        info!("Sending SKSE Event: {:?}", result);
        let event = match result {
            TkConnectionEvent::Connected(connector) => {
                SKSEModEvent::from("Tele_Connected", &connector)
            }
            TkConnectionEvent::ConnectionFailure(err) => {
                SKSEModEvent::from("Tele_ConnectionError", &err)
            }
            TkConnectionEvent::DeviceAdded(device, battery_level) => {
                let mut evt = SKSEModEvent::from("Tele_DeviceAdded", device.name());
                evt.num_arg = battery_level.unwrap_or(0).into();
                evt
            }
            TkConnectionEvent::DeviceRemoved(device) => {
                SKSEModEvent::from("Tele_DeviceRemoved", device.name())
            }
            TkConnectionEvent::ActionStarted(task, actuators, tags, handle) => {
                let str_arg = format!(
                    "{}{} on ({})",
                    task,
                    if !tags.is_empty() {
                        format!(" {}", tags.iter().join(","))
                    } else {
                        String::default()
                    },
                    actuators.iter().map(|x| x.identifier()).join(",")
                );
                SKSEModEvent::new("Tele_DeviceActionStarted", &str_arg, f64::from(handle))
            }
            TkConnectionEvent::ActionDone(task, duration, handle) => {
                let str_arg = format!("{} done after {:.1}s", task, duration.as_secs());
                SKSEModEvent::new("Tele_DeviceActionDone", &str_arg, f64::from(handle))
            }
            TkConnectionEvent::ActionError(_actuator, err) => {
                SKSEModEvent::new("Tele_DeviceError", &err, 0.0)
            }
            TkConnectionEvent::BatteryLevel(device, battery_level) => {
                SKSEModEvent::new("Tele_BatteryLevel", device.name(), battery_level.unwrap_or(0).into())
            },
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
        exec: |tk| tk.status.connection_status().to_string(),
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
        exec: |tk, speed, time_sec, _, body_parts| {
            let cmd = DeviceCommand::from_inputs(
                Task::Scalar(Speed::new(speed.into())),
                &[ActuatorType::Vibrate],
                time_sec,
                body_parts,
                None);
            tk.dispatch_cmd(cmd)
        },
        default: ERROR_HANDLE,
    })
    .def_control(ApiControl {
        name: "scalar",
        exec: |tk, speed, time_sec, actuator_type, body_parts: &CxxVector<CxxString>| {
            let cmd = DeviceCommand::from_inputs(
                Task::Scalar(Speed::new(speed.into())),
                &[read_scalar_actuator(actuator_type)],
                time_sec,
                body_parts,
                None);
            tk.dispatch_cmd(cmd)
        },
        default: ERROR_HANDLE,
    })
    .def_control(ApiControl {
        name: "vibrate.pattern",
        exec: |tk, speed, time_sec, pattern_name, body_parts| match read_pattern(
            &tk.settings.pattern_path,
            pattern_name,
            true,
        ) {
            Some(fscript) => {
                let cmd = DeviceCommand::from_inputs(
                    Task::Pattern(
                        Speed::new(speed.into()),
                        ActuatorType::Vibrate,
                        pattern_name.into(),
                    ),
                    &[ActuatorType::Vibrate],
                    time_sec,
                    body_parts,
                    Some(fscript));
                tk.dispatch_cmd(cmd)
            },
            None => ERROR_HANDLE,
        },
        default: ERROR_HANDLE,
    })
    .def_control(ApiControl {
        name: "linear.pattern",
        exec: |tk, speed, time_sec, pattern_name, body_parts| match read_pattern(
            &tk.settings.pattern_path,
            pattern_name,
            false,
        ) {
            Some(fscript) => {
                let cmd = DeviceCommand::from_inputs(
                    Task::Linear(Speed::new(speed.into()), pattern_name.into()),
                    &[ActuatorType::Position],
                    time_sec,
                    body_parts,
                    Some(fscript));
                tk.dispatch_cmd(cmd)
            },
            None => ERROR_HANDLE,
        },
        default: ERROR_HANDLE,
    })
    .def_control(ApiControl {
        name: "linear.oscillate",
        exec: |tk, speed, time_sec, pattern_name, body_parts| {
            let cmd = DeviceCommand::from_inputs(
                Task::LinearOscillate(Speed::new(speed.into()), pattern_name.into()),
                &[ActuatorType::Oscillate],
                time_sec,
                body_parts,
                None);
            tk.dispatch_cmd(cmd)
        },
        default: ERROR_HANDLE,
    })
    .def_update(ApiUpdate {
        exec: |tk, handle, speed| tk.update(handle, Speed::new(speed.into())),
    })
    .def_stop(ApiStop {
        exec: Telekinesis::stop,
    })
    .def_cmd(ApiCmd0 {
        name: "stop_all",
        exec: Telekinesis::stop_all,
    })
    // settings
    .def_cmd(ApiCmd0 {
        name: "settings.store",
        exec: |tk| tk.settings.try_write(SETTINGS_PATH, SETTINGS_FILE),
    })
    // devices settings
    .def_qry_lst(ApiQryList {
        name: "devices",
        exec: |tk| tk.status.get_known_actuator_ids(),
    })
    .def_qry_bool_1(ApiQryBool1 { 
        name: "device.has_battery_level", 
        exec: |tk, actuator_id| {
            if let Some(actuator) = tk.status.get_actuator(actuator_id)  {
                return actuator.device.has_battery_level();
            }
            false
        }
    })
    .def_qry_str1(ApiQryStr1 { 
        name: "device.get_battery_level", 
        exec: |tk, actuator_id| {
            if let Some(actuator_status) = tk.status.get_actuator_status(actuator_id)  {
                return actuator_status.battery_level.map(|x| x.to_string()).unwrap_or("".to_owned());
            }
            "".into()
        },
        default: ""
    })
    .def_qry_str1(ApiQryStr1 {
        name: "device.actuator",
        default: "Not Connected",
        exec: |tk, actuator_id| {
            if let Some(actuator) = tk.status.get_actuator(actuator_id) {
                return actuator.actuator.to_string();
            }
            String::default()
        },
    })
    .def_qry_str1(ApiQryStr1 {
        name: "device.actuator_type",
        default: "None",
        exec: |tk, actuator_id| match tk.settings.try_get_actuator_settings(actuator_id) {
            ActuatorSettings::None => {
                if let Some(entry) = tk.status.get_actuator(actuator_id) {
                    return match entry.actuator {
                        ActuatorType::Position => "Linear".into(),
                        _ => "Scalar".into()
                    };
                }
                "None".into()
            }
            ActuatorSettings::Scalar(_) => "Scalar".into(),
            ActuatorSettings::Linear(_) => "Linear".into(),
        },
    })
    .def_qry_str1(ApiQryStr1 {
        name: "device.actuator.index",
        default: "1",
        exec: |tk, actuator_id| {
            if let Some(actuator) = tk.status.get_actuator(actuator_id) {
                return (actuator.index_in_device + 1).to_string();
            }
            "1".into()
        },
    })
    .def_cmd1(ApiCmd1 {
        name: "device.settings.enable",
        exec: |tk, actuator_id| {
            tk.settings.set_enabled(actuator_id, true);
            true
        },
    })
    .def_cmd1(ApiCmd1 {
        name: "device.settings.disable",
        exec: |tk, actuator_id| {
            tk.settings.set_enabled(actuator_id, false);
            true
        },
    })
    .def_qry_bool_1(ApiQryBool1 {
        name: "device.settings.enabled",
        exec: |tk, actuator_id| tk.settings.get_enabled(actuator_id),
    })
    .def_cmd2(ApiCmd2 {
        name: "device.settings.events",
        exec: |tk, actuator_id, events| {
            tk.settings.set_events(actuator_id, &parse_csv(events));
            true
        },
    })
    .def_qry_lst_1(ApiQryList1 {
        name: "device.settings.events",
        exec: |tk, actuator_id| tk.settings.get_events(actuator_id),
    })
    .def_qry_str1(ApiQryStr1 {
        name: "device.scalar.min_speed",
        default: "",
        exec: |tk, actuator_id| {
            tk.settings
                .access_scalar(actuator_id, |x| x.min_speed.to_string())
        },
    })
    .def_cmd2(ApiCmd2 {
        name: "device.scalar.min_speed",
        exec: |tk, actuator_id, percent| {
            tk.settings.access_scalar(actuator_id, |x| {
                x.min_speed = percent.parse().unwrap_or(0);
            });
            true
        },
    })
    .def_qry_str1(ApiQryStr1 {
        name: "device.scalar.max_speed",
        default: "",
        exec: |tk, actuator_id| {
            tk.settings
                .access_scalar(actuator_id, |x| x.max_speed.to_string())
        },
    })
    .def_cmd2(ApiCmd2 {
        name: "device.scalar.max_speed",
        exec: |tk, actuator_id, percent| {
            tk.settings.access_scalar(actuator_id, |x| {
                x.max_speed = percent.parse().unwrap_or(100);
            });
            true
        },
    })
    .def_qry_str1(ApiQryStr1 {
        name: "device.scalar.factor",
        default: "",
        exec: |tk, actuator_id| {
            tk.settings
                .access_scalar(actuator_id, |x| x.factor.to_string())
        },
    })
    .def_cmd2(ApiCmd2 {
        name: "device.scalar.factor",
        exec: |tk, actuator_id, factor| {
            tk.settings.access_scalar(actuator_id, |x| {
                x.factor = factor.parse().unwrap_or(1.0);
            });
            true
        },
    })
    .def_qry_str1(ApiQryStr1 {
        name: "device.linear.min_ms",
        default: "",
        exec: |tk, actuator_id| {
            tk.settings
                .access_linear(actuator_id, |x| x.min_ms.to_string())
        },
    })
    .def_cmd2(ApiCmd2 {
        name: "device.linear.min_ms",
        exec: |tk, actuator_id, percent| {
            tk.settings.access_linear(actuator_id, |x| {
                x.min_ms = percent.parse().unwrap_or(0);
            });
            true
        },
    })
    .def_qry_str1(ApiQryStr1 {
        name: "device.linear.max_ms",
        default: "",
        exec: |tk, actuator_id| {
            tk.settings
                .access_linear(actuator_id, |x| x.max_ms.to_string())
        },
    })
    .def_cmd2(ApiCmd2 {
        name: "device.linear.max_ms",
        exec: |tk, actuator_id, percent| {
            tk.settings
                .access_linear(actuator_id, |x| x.max_ms = percent.parse().unwrap_or(100));
            true
        },
    })
    .def_qry_str1(ApiQryStr1 {
        name: "device.linear.min_pos",
        default: "",
        exec: |tk, actuator_id| {
            tk.settings
                .access_linear(actuator_id, |x| x.min_pos.to_string())
        },
    })
    .def_cmd2(ApiCmd2 {
        name: "device.linear.min_pos",
        exec: |tk, actuator_id, percent| {
            tk.settings
                .access_linear(actuator_id, |x| x.min_pos = percent.parse().unwrap_or(0.0));
            true
        },
    })
    .def_qry_str1(ApiQryStr1 {
        name: "device.linear.max_pos",
        default: "",
        exec: |tk, actuator_id| {
            tk.settings
                .access_linear(actuator_id, |x| x.max_pos.to_string())
        },
    })
    .def_cmd2(ApiCmd2 {
        name: "device.linear.max_pos",
        exec: |tk, actuator_id, percent| {
            tk.settings.access_linear(actuator_id, |x| {
                x.max_pos = percent.parse().unwrap_or(LinearRange::default().max_pos)
            });
            true
        },
    })
    .def_qry_bool_1(ApiQryBool1 {
        name: "device.linear.invert",
        exec: |tk, actuator_id| tk.settings.access_linear(actuator_id, |x| x.invert),
    })
    .def_cmd1(ApiCmd1 {
        name: "device.linear.invert.enable",
        exec: |tk, actuator_id| {
            tk.settings.access_linear(actuator_id, |x| x.invert = true);
            true
        },
    })
    .def_cmd1(ApiCmd1 {
        name: "device.linear.invert.disable",
        exec: |tk, actuator_id| {
            tk.settings.access_linear(actuator_id, |x| x.invert = false);
            true
        },
    })
    // connection
    .def_qry_str1(ApiQryStr1 {
        name: "device.connection.status",
        default: "Not Connected",
        exec: |tk, actuator_id| tk.status.get_actuator_connection_status(actuator_id).to_string(),
    })
    .def_qry_str1(ApiQryStr1 {
        name: "device.connection.status",
        default: "Not Connected",
        exec: |tk, actuator_id| tk.status.get_actuator_connection_status(actuator_id).to_string(),
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
