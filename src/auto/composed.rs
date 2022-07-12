use crate::time;
use std::{cell::RefCell, rc::Rc};

pub struct Composed(Vec<super::Control>);

impl time::TimeKeeper for Composed {
    fn set_time_manager(&mut self, time_manager: Rc<RefCell<time::TimeManager>>) {
        for time_function in &mut self.0 {
            time_function.set_time_manager(Rc::clone(&time_manager))
        }
    }
}

impl super::CtrlFunction for Composed {
    fn get_value(&self, time: time::TimeStamp) -> super::CtrlVal {
        let mut val = 1_f64;
        for control in &self.0 {
            val *= control.get_value(time)
        }
        val
    }

    fn get_vec(&self, start: time::TimeStamp, samples: usize) -> Vec<super::CtrlVal> {
        let mut vec = vec![1_f64; samples];
        for control in &self.0 {
            vec = vec
                .into_iter()
                .zip(control.get_vec(start, samples).into_iter())
                .map(|(x1, x2)| x1 * x2)
                .collect();
        }
        vec
    }

    fn trigger(&self, samples: usize) -> Vec<super::CtrlVal> {
        self.get_vec(time::TimeStamp::zero(), samples)
    }
}
