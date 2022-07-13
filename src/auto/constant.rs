use crate::time;
use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Default)]
pub struct Constant(pub f64);

impl Constant {
    pub fn set(&mut self, value: f64) {
        self.0 = value
    }
}

impl time::TimeKeeper for Constant {
    fn set_time_manager(&mut self, _time_manager: Rc<RefCell<time::TimeManager>>) {}
}

impl super::CtrlFunction for Constant {
    fn get_value(&self, _time: time::TimeStamp) -> f64 {
        self.0
    }

    fn get_vec(&self, _start: time::TimeStamp, samples: usize) -> Vec<f64> {
        vec![self.0; samples]
    }
}
