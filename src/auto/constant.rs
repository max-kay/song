use crate::time::{TimeKeeper, TimeManager, TimeStamp};
use std::{cell::RefCell, rc::Rc};

use super::CtrlFunction;

#[derive(Debug, Default)]
pub struct Constant {
    val: f64,
    id: usize,
}

impl Constant {
    pub fn new() -> Self {
        Self {
            val: 0.0,
            id: super::get_ctrl_id(),
        }
    }
    pub fn set(&mut self, value: f64) {
        assert!((0.0..=1.0).contains(&value));
        self.val = value
    }
}

impl TimeKeeper for Constant {
    fn set_time_manager(&mut self, _time_manager: Rc<RefCell<TimeManager>>) {}
}

impl CtrlFunction for Constant {
    fn get_value(&self, _time: TimeStamp) -> f64 {
        self.val
    }

    fn get_vec(&self, _start: TimeStamp, samples: usize) -> Vec<f64> {
        vec![self.val; samples]
    }

    fn get_id(&self) -> usize {
        self.id
    }

    fn get_sub_ids(&self) -> Vec<usize> {
        Vec::new()
    }
}
