use serde::{Deserialize, Serialize};

use crate::speed::Speed;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ScalarScaling {
    // Note: currently unused
    Linear,            // f(x) = x
    Quadratic,         // f(x) = x^2
    QuadraticFraction, // f(x) = x^(1/2)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ScalarRange {
    pub min_speed: i64,
    pub max_speed: i64,
    pub factor: f64,
    pub scaling: ScalarScaling,
}

impl Default for ScalarRange {
    fn default() -> Self {
        Self {
            min_speed: 0,
            max_speed: 100,
            factor: 1.0,
            scaling: ScalarScaling::Linear,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub enum ActuatorSettings {
    #[default]
    None,
    Scalar(ScalarRange),
    Linear(LinearRange),
}

impl ActuatorSettings {
    pub fn linear_or_max(&self) -> LinearRange {
        if let ActuatorSettings::Linear(settings) = self {
            return settings.clone();
        }
        LinearRange::max()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LinearSpeedScaling {
    Linear,         // f(x) = x
    Parabolic(i32), // f(x) = 1 - (1 - x)^n
}

impl LinearSpeedScaling {
    pub fn apply(&self, speed: Speed) -> Speed {
        match self {
            LinearSpeedScaling::Linear => speed,
            LinearSpeedScaling::Parabolic(n) => {
                let mut x = speed.as_float();
                x = 1.0 - (1.0 - x).powi(*n);
                Speed::from_float(x)
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LinearRange {
    pub min_ms: i64,
    pub max_ms: i64,
    pub min_pos: f64,
    pub max_pos: f64,
    pub invert: bool,
    pub scaling: LinearSpeedScaling,
}

impl LinearRange {
    pub fn max() -> Self {
        Self {
            min_ms: 50,
            max_ms: 10_000,
            min_pos: 0.0,
            max_pos: 1.0,
            invert: false,
            scaling: LinearSpeedScaling::Linear,
        }
    }
}
impl Default for LinearRange {
    fn default() -> Self {
        Self {
            min_ms: 250,
            max_ms: 3000,
            min_pos: 0.0,
            max_pos: 1.0,
            invert: false,
            scaling: LinearSpeedScaling::Linear,
        }
    }
}
