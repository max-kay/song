use crate::{
    control::{ControlError, FunctionKeeper},
    time::TimeStamp,
};

use super::{CtrlFunction, IdMap};

#[derive(Debug)]
pub struct PhantomF();

impl FunctionKeeper for PhantomF {
    fn heal_sources(&mut self, _id_map: &IdMap) -> Result<(), ControlError> {
        Ok(())
    }

    fn test_sources(&self) -> Result<(), ControlError> {
        Err(ControlError::new_phantom_f_err())
    }

    fn set_ids(&mut self) {
        panic!(
            "no associated function, use test function before this function and handle the result"
        )
    }

    fn get_ids(&self) -> Vec<usize> {
        panic!(
            "no associated function, use test function before this function and handle the result"
        )
    }
}

impl CtrlFunction for PhantomF {
    fn get_value(&self, _time: TimeStamp) -> f64 {
        panic!(
            "no associated function, use test function before this function and handle the result"
        )
    }

    fn get_vec(&self, _start: TimeStamp, _samples: usize) -> Vec<f64> {
        panic!(
            "no associated function, use test_sources() before this function and handle the result"
        )
    }

    fn get_id(&self) -> usize {
        panic!(
            "no associated function, use test function before this function and handle the result"
        )
    }

    unsafe fn new_id_f(&mut self) {
        panic!(
            "no associated function, use test function before this function and handle the result"
        )
    }
}
