use std::sync::{Arc, Mutex};

use itertools::Itertools;
use linear::LinearRange;
use tracing::{info, instrument};

use cxx::{CxxString, CxxVector};

use buttplug::core::message::ActuatorType;

use bp_scheduler::{
    client::{
        actions::*, connection::*, input::*, pattern::*, settings::*, telekinesis::*
    }, 
    settings::*, 
    speed::*
};

use api::*;
use ffi::SKSEModEvent;

mod api;
mod logging;

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

pub fn get_next_events_blocking(
    connection_events: &crossbeam_channel::Receiver<TkConnectionEvent>,
) -> Option<SKSEModEvent> {
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
                evt.num_arg = battery_level.unwrap_or(0.0);
                evt
            }
            TkConnectionEvent::DeviceRemoved(device) => {
                SKSEModEvent::from("Tele_DeviceRemoved", device.name())
            }
            TkConnectionEvent::ActionStarted(action, actuators, tags, handle) => {
                let str_arg = format!(
                    "{}{} on ({})",
                    action.name,
                    if !tags.is_empty() {
                        format!(" {}", tags.iter().join(","))
                    } else {
                        String::default()
                    },
                    actuators.iter().map(|x| x.identifier()).join(",")
                );
                SKSEModEvent::new("Tele_DeviceActionStarted", &str_arg, f64::from(handle))
            }
            TkConnectionEvent::ActionDone(action, duration, handle) => {
                let str_arg = format!("{} done after {:.1}s", action.name, duration.as_secs());
                SKSEModEvent::new("Tele_DeviceActionDone", &str_arg, f64::from(handle))
            }
            TkConnectionEvent::ActionError(_actuator, err) => {
                SKSEModEvent::new("Tele_DeviceError", &err, 0.0)
            }
            TkConnectionEvent::BatteryLevel(device, battery_level) => SKSEModEvent::new(
                "Tele_BatteryLevel",
                device.name(),
                battery_level.unwrap_or(0.0),
            ),
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
            let action = Action {
                name: "vibrate".into(),
                speed: Speed::new(100),
                control: Control::Scalar(vec![ScalarActuators::Vibrate]),
            };
            tk.dispatch_cmd(
                action,
                read_input_string(body_parts),
                Speed::new(speed.into()),
                get_duration_from_secs(time_sec),
            )
        },
        default: ERROR_HANDLE,
    })
    .def_control(ApiControl {
        name: "scalar",
        exec: |tk, speed, time_sec, actuator_type, body_parts: &CxxVector<CxxString>| {
            let action = Action {
                name: "scalar".into(),
                speed: Speed::new(100),
                control: Control::Scalar(vec![
                    ScalarActuators::Vibrate,
                    ScalarActuators::Constrict,
                    ScalarActuators::Oscillate,
                    ScalarActuators::Inflate,
                ]),
            };
            tk.dispatch_cmd(
                action,
                read_input_string(body_parts),
                Speed::new(speed.into()),
                get_duration_from_secs(time_sec),
            )
        },
        default: ERROR_HANDLE,
    })
    .def_control(ApiControl {
        name: "vibrate.pattern",
        exec: |tk, speed, time_sec, pattern_name, body_parts| {
            let action = Action {
                name: "vibrate.pattern".into(),
                speed: Speed::new(100),
                control: Control::ScalarPattern(
                    pattern_name.into(),
                    vec![
                        ScalarActuators::Vibrate,
                        ScalarActuators::Constrict,
                        ScalarActuators::Oscillate,
                        ScalarActuators::Inflate,
                    ],
                ),
            };
            tk.dispatch_cmd(
                action,
                read_input_string(body_parts),
                Speed::new(speed.into()),
                get_duration_from_secs(time_sec),
            )
        },
        default: ERROR_HANDLE,
    })
    .def_control(ApiControl {
        name: "linear.pattern",
        exec: |tk, speed, time_sec, pattern_name, body_parts| {
            let action = Action {
                name: "vibrate.pattern".into(),
                speed: Speed::new(100),
                control: Control::StrokePattern(pattern_name.into()),
            };
            tk.dispatch_cmd(
                action,
                read_input_string(body_parts),
                Speed::new(speed.into()),
                get_duration_from_secs(time_sec),
            )
        },
        default: ERROR_HANDLE,
    })
    .def_control(ApiControl {
        name: "linear.stroke",
        exec: |tk, speed, time_sec, pattern_name, body_parts| {
            let action = Action {
                name: "linear.stroke".into(),
                speed: Speed::new(100),
                control: Control::Stroke(StrokeRange {
                    min_ms: 100,
                    max_ms: 200,
                    min_pos: 0.0,
                    max_pos: 1.0,
                }),
            };
            tk.dispatch_cmd(
                action,
                read_input_string(body_parts),
                Speed::new(speed.into()),
                get_duration_from_secs(time_sec),
            )
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
            if let Some(actuator) = tk.status.get_actuator(actuator_id) {
                return actuator.device.has_battery_level();
            }
            false
        },
    })
    .def_qry_str1(ApiQryStr1 {
        name: "device.get_battery_level",
        exec: |tk, actuator_id| {
            if let Some(actuator_status) = tk.status.get_actuator_status(actuator_id) {
                return actuator_status
                    .battery_level
                    .map(|x| ((x * 100.0) as i32).to_string())
                    .unwrap_or("".to_owned());
            }
            "".into()
        },
        default: "",
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
        exec: |tk, actuator_id| match tk
            .settings
            .device_settings
            .try_get_actuator_settings(actuator_id)
        {
            ActuatorSettings::None => {
                if let Some(entry) = tk.status.get_actuator(actuator_id) {
                    return match entry.actuator {
                        ActuatorType::Position => "Linear".into(),
                        _ => "Scalar".into(),
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
            tk.settings.device_settings.set_enabled(actuator_id, true);
            true
        },
    })
    .def_cmd1(ApiCmd1 {
        name: "device.settings.disable",
        exec: |tk, actuator_id| {
            tk.settings.device_settings.set_enabled(actuator_id, false);
            true
        },
    })
    .def_qry_bool_1(ApiQryBool1 {
        name: "device.settings.enabled",
        exec: |tk, actuator_id| tk.settings.device_settings.get_enabled(actuator_id),
    })
    .def_cmd2(ApiCmd2 {
        name: "device.settings.events",
        exec: |tk, actuator_id, events| {
            tk.settings
                .device_settings
                .set_events(actuator_id, &parse_csv(events));
            true
        },
    })
    .def_qry_lst_1(ApiQryList1 {
        name: "device.settings.events",
        exec: |tk, actuator_id| tk.settings.device_settings.get_events(actuator_id),
    })
    .def_qry_str1(ApiQryStr1 {
        name: "device.scalar.min_speed",
        default: "",
        exec: |tk, actuator_id| {
            tk.settings
                .device_settings
                .update_scalar(actuator_id, |x| x.min_speed.to_string())
        },
    })
    .def_cmd2(ApiCmd2 {
        name: "device.scalar.min_speed",
        exec: |tk, actuator_id, percent| {
            tk.settings.device_settings.update_scalar(actuator_id, |x| {
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
                .device_settings
                .update_scalar(actuator_id, |x| x.max_speed.to_string())
        },
    })
    .def_cmd2(ApiCmd2 {
        name: "device.scalar.max_speed",
        exec: |tk, actuator_id, percent| {
            tk.settings.device_settings.update_scalar(actuator_id, |x| {
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
                .device_settings
                .update_scalar(actuator_id, |x| x.factor.to_string())
        },
    })
    .def_cmd2(ApiCmd2 {
        name: "device.scalar.factor",
        exec: |tk, actuator_id, factor| {
            tk.settings.device_settings.update_scalar(actuator_id, |x| {
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
                .device_settings
                .update_linear(actuator_id, |x| x.min_ms.to_string())
        },
    })
    .def_cmd2(ApiCmd2 {
        name: "device.linear.min_ms",
        exec: |tk, actuator_id, percent| {
            tk.settings.device_settings.update_linear(actuator_id, |x| {
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
                .device_settings
                .update_linear(actuator_id, |x| x.max_ms.to_string())
        },
    })
    .def_cmd2(ApiCmd2 {
        name: "device.linear.max_ms",
        exec: |tk, actuator_id, percent| {
            tk.settings
                .device_settings
                .update_linear(actuator_id, |x| x.max_ms = percent.parse().unwrap_or(100));
            true
        },
    })
    .def_qry_str1(ApiQryStr1 {
        name: "device.linear.min_pos",
        default: "",
        exec: |tk, actuator_id| {
            tk.settings
                .device_settings
                .update_linear(actuator_id, |x| x.min_pos.to_string())
        },
    })
    .def_cmd2(ApiCmd2 {
        name: "device.linear.min_pos",
        exec: |tk, actuator_id, percent| {
            tk.settings
                .device_settings
                .update_linear(actuator_id, |x| x.min_pos = percent.parse().unwrap_or(0.0));
            true
        },
    })
    .def_qry_str1(ApiQryStr1 {
        name: "device.linear.max_pos",
        default: "",
        exec: |tk, actuator_id| {
            tk.settings
                .device_settings
                .update_linear(actuator_id, |x| x.max_pos.to_string())
        },
    })
    .def_cmd2(ApiCmd2 {
        name: "device.linear.max_pos",
        exec: |tk, actuator_id, percent| {
            tk.settings.device_settings.update_linear(actuator_id, |x| {
                x.max_pos = percent.parse().unwrap_or(LinearRange::default().max_pos)
            });
            true
        },
    })
    .def_qry_bool_1(ApiQryBool1 {
        name: "device.linear.invert",
        exec: |tk, actuator_id| {
            tk.settings
                .device_settings
                .update_linear(actuator_id, |x| x.invert)
        },
    })
    .def_cmd1(ApiCmd1 {
        name: "device.linear.invert.enable",
        exec: |tk, actuator_id| {
            tk.settings
                .device_settings
                .update_linear(actuator_id, |x| x.invert = true);
            true
        },
    })
    .def_cmd1(ApiCmd1 {
        name: "device.linear.invert.disable",
        exec: |tk, actuator_id| {
            tk.settings
                .device_settings
                .update_linear(actuator_id, |x| x.invert = false);
            true
        },
    })
    // connection
    .def_qry_str1(ApiQryStr1 {
        name: "device.connection.status",
        default: "Not Connected",
        exec: |tk, actuator_id| {
            tk.status
                .get_actuator_connection_status(actuator_id)
                .to_string()
        },
    })
    .def_qry_str1(ApiQryStr1 {
        name: "device.connection.status",
        default: "Not Connected",
        exec: |tk, actuator_id| {
            tk.status
                .get_actuator_connection_status(actuator_id)
                .to_string()
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

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use bp_scheduler::{
        client::{
            actions::*, 
            connection::*, 
            telekinesis::*
        }, speed::Speed};
    use buttplug::core::message::ActuatorType;
    use funscript::FScript;

    use crate::get_next_events_blocking;

    /// Vibrate
    pub fn test_cmd(
        tk: &mut Telekinesis,
        task: Task,
        duration: Duration,
        body_parts: Vec<String>,
        fscript: Option<FScript>,
        actuator_types: &[ActuatorType],
    ) -> i32 {
        let speed: Speed = match task {
            Task::Scalar(speed) => speed,
            Task::Pattern(speed, _, _) => speed,
            Task::Linear(speed, _) => speed,
            Task::LinearStroke(speed, _) => speed,
        };
        tk.dispatch_cmd(Action {
            name: "something".into(),
            speed,
            control: Control::Scalar(vec![ ScalarActuators::Vibrate ]),
        }, body_parts, speed, duration )
    }

    /// Events
    #[test]
    fn process_next_events_after_action_returns_1() {
        let mut tk = Telekinesis::connect_with(
            || async move { in_process_connector() },
            None,
            TkConnectionType::Test,
        )
        .unwrap();
        test_cmd(
            &mut tk,
            Task::Scalar(Speed::new(22)),
            Duration::from_millis(1),
            vec![],
            None,
            &[ActuatorType::Vibrate],
        );
        get_next_events_blocking(&tk.connection_events);
    }

    #[test]
    fn process_next_events_works() {
        let mut tk = Telekinesis::connect_with(
            || async move { in_process_connector() },
            None,
            TkConnectionType::Test,
        )
        .unwrap();
        test_cmd(
            &mut tk,
            Task::Scalar(Speed::new(10)),
            Duration::from_millis(100),
            vec![],
            None,
            &[ActuatorType::Vibrate],
        );
        test_cmd(
            &mut tk,
            Task::Scalar(Speed::new(20)),
            Duration::from_millis(200),
            vec![],
            None,
            &[ActuatorType::Vibrate],
        );
        get_next_events_blocking(&tk.connection_events);
        get_next_events_blocking(&tk.connection_events);
    }
}
