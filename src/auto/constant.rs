use crate::time;
use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Default)]
pub struct Constant(pub super::CtrlVal);

impl Constant {
    pub fn set(&mut self, value: super::CtrlVal) {
        self.0 = value
    }
}

impl time::TimeKeeper for Constant {
    fn set_time_manager(&mut self, _time_manager: Rc<RefCell<time::TimeManager>>) {}
}

impl super::CtrlFunction for Constant {
    fn get_value(&self, _time: time::TimeStamp) -> super::CtrlVal {
        self.0
    }

    fn get_vec(&self, _start: time::TimeStamp, samples: usize) -> Vec<super::CtrlVal> {
        vec![self.0; samples]
    }

    fn trigger(&self, samples: usize) -> Vec<super::CtrlVal> {
        vec![self.0; samples]
    }
}
