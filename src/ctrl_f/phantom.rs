use crate::{
    ctrl_f::{ControlError},
    time::TimeStamp,
};

use super::{CtrlFunction, };

#[derive(Debug)]
pub struct PhantomF();

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
}
