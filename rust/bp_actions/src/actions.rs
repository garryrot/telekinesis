// actions/*.json

use bp_scheduler::{actuator::Actuator, speed::Speed};
use buttplug::core::message::ActuatorType;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Actions(Vec<Action>);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StrokeRange {
    pub min_ms: i64,
    pub max_ms: i64,
    pub min_pos: f64,
    pub max_pos: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Action {
    pub name: String,
    pub speed: Speed,
    pub control: Control,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Control {
    Scalar(Vec<ScalarActuators>),
    ScalarPattern(String, Vec<ScalarActuators>),
    Stroke(StrokeRange),
    StrokePattern(String),
}

impl Control {
    pub fn get_actuators(&self) -> Vec<ActuatorType> {
        match self {
            Control::Scalar(y) => y.iter().map(|x| x.clone().into()).collect(),
            Control::ScalarPattern(_, y) => y.iter().map(|x| x.clone().into()).collect(),
            Control::Stroke(range) => vec![ActuatorType::Position],
            Control::StrokePattern(_) => vec![ActuatorType::Position],
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ScalarActuators {
    Vibrate,
    Oscillate,
    Constrict,
    Inflate,
}

impl From<ScalarActuators> for buttplug::core::message::ActuatorType {
    fn from(val: ScalarActuators) -> Self {
        match val {
            ScalarActuators::Vibrate => ActuatorType::Vibrate,
            ScalarActuators::Oscillate => ActuatorType::Oscillate,
            ScalarActuators::Constrict => ActuatorType::Constrict,
            ScalarActuators::Inflate => ActuatorType::Inflate,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum BodyParts {
    All,
    Tags(Vec<String>),
}

#[cfg(test)]
mod tests {
    use super::*;

    // pub fn read_config( path: String ) -> Actions {
    // }

    pub fn build_config() {
        let default_actions = Actions(vec![
            Action {
                name: "vibrate".into(),
                speed: Speed::new(100),
                control: Control::Scalar(vec![ScalarActuators::Vibrate]),
            },
            Action {
                name: "constrict".into(),
                speed: Speed::new(100),
                control: Control::Scalar(vec![ScalarActuators::Constrict]),
            },
            Action {
                name: "inflate".into(),
                speed: Speed::new(100),
                control: Control::Scalar(vec![ScalarActuators::Constrict]),
            },
            Action {
                name: "scalar".into(),
                speed: Speed::new(100),
                control: Control::Scalar(vec![
                    ScalarActuators::Vibrate,
                    ScalarActuators::Constrict,
                    ScalarActuators::Oscillate,
                    ScalarActuators::Inflate,
                ]),
            },
        ]);

        serde_json::to_string_pretty(&default_actions).unwrap();
    }
}
