use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ScalarScaling {
    Linear,           // f(x) = x
    Quadratic,        // f(x) = x^2 
    QuadraticFraction // f(x) = x^(1/2)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ScalarSettings {
    pub min_speed: i64,
    pub max_speed: i64,
    pub scaling: ScalarScaling
}

impl Default for ScalarSettings {
    fn default() -> Self {
        Self {
            min_speed: 0,
            max_speed: 100,
            scaling: ScalarScaling::Linear
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub enum ActuatorSettings {
    #[default]
    None,
    Scalar(ScalarSettings),
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
pub struct LinearRange {
    pub min_ms: i64,
    pub max_ms: i64,
    pub min_pos: f64,
    pub max_pos: f64,
    pub invert: bool,
}

impl LinearRange {
    pub fn max() -> Self {
        Self {
            min_ms: 50,
            max_ms: 10_000,
            min_pos: 0.0,
            max_pos: 1.0,
            invert: false,
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
        }
    }
}
