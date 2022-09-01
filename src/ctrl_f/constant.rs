use crate::{time::TimeStamp, };

use super::CtrlFunction;

#[derive(Debug, Default)]
pub struct Constant {
    val: f64,
}

impl Constant {
    pub fn new() -> Self {
        Self {
            val: 0.0,
        }
    }
    pub fn set(&mut self, value: f64) {
        assert!((0.0..=1.0).contains(&value));
        self.val = value
    }
}

impl CtrlFunction for Constant {
    fn get_value(&self, _time: TimeStamp) -> f64 {
        self.val
    }

    fn get_vec(&self, _start: TimeStamp, samples: usize) -> Vec<f64> {
        vec![self.val; samples]
    }
}
