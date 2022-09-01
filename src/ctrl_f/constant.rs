use crate::{
    control::ControlError,
    time::TimeStamp,
    utils::{self, get_f_id},
};

use super::{CtrlFunction, FunctionKeeper, FunctionOwner, IdMap, IdMapOrErr};

#[derive(Debug, Default)]
pub struct Constant {
    val: f64,
    id: usize,
}

impl Constant {
    pub fn new() -> Self {
        Self {
            val: 0.0,
            id: get_f_id(),
        }
    }
    pub fn set(&mut self, value: f64) {
        assert!((0.0..=1.0).contains(&value));
        self.val = value
    }
}

impl FunctionKeeper for Constant {
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

impl FunctionOwner for Constant {
    unsafe fn new_ids(&mut self) {}

    fn get_id_map(&self) -> IdMapOrErr {
        Ok(IdMap::new())
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
        self.id = utils::get_f_id();
    }
}
