use crate::{
    control::ControlError,
    time::{TimeKeeper, TimeManager, TimeStamp},
    utils::{self, get_ctrl_id},
};
use std::{cell::RefCell, rc::Rc};

use super::{CtrlFunction, IdMap, SourceKeeper};

#[derive(Debug, Default)]
pub struct Constant {
    val: f64,
    id: usize,
}

impl Constant {
    pub fn new() -> Self {
        Self {
            val: 0.0,
            id: get_ctrl_id(),
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

impl SourceKeeper for Constant {
    fn heal_sources(&mut self, _id_map: &IdMap) -> Result<(), ControlError> {
        Ok(())
    }

    fn test_sources(&self) -> Result<(), ControlError> {
        Ok(())
    }

    fn set_ids(&mut self) {}

    fn get_ids(&self) -> Vec<usize> {
        vec![self.get_id()]
    }
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

    unsafe fn new_id_f(&mut self) {
        self.id = utils::get_ctrl_id();
    }
}
