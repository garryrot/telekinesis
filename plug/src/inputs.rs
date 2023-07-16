use std::fmt::{Display, self};
use util::Narrow;
use cxx::{CxxVector, CxxString};

use crate::util;


#[derive(Debug, Clone, Copy)]
pub struct Speed {
    pub value: u16,
}

impl Speed {
    pub fn new(percentage: i64) -> Speed {
        Speed {
            value: percentage.narrow(0, 100) as u16,
        }
    }
    pub fn min() -> Speed {
        Speed { value: 0 }
    }
    pub fn max() -> Speed {
        Speed { value: 100 }
    }
    pub fn as_float(self) -> f64 {
        self.value as f64 / 100.0
    }
}

impl Display for Speed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

pub fn as_string_list(list: &CxxVector<CxxString>) -> Vec<String> {
    list.iter()
        .map(|d| d.to_string_lossy().into_owned())
        .collect()
}