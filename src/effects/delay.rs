use std::marker::PhantomData;
use std::rc::Rc;
use super::{Effect, Control};

use crate::utils::seconds_to_samples;
use crate::wave;
use crate::time;

pub struct Delay<W: wave::Wave> {
    phantom: PhantomData<W>,
    time_keeper: Rc<time::TimeKeeper>,
    gain_ctrl: Control,
    delta_t_ctrl: Control,
}

impl<W: wave::Wave> Effect<W> for Delay<W> {
    fn apply(&self, wave: &mut W, time_triggered: time::TimeStamp) {
        let mut source = wave.clone();

        let mut current_time = time_triggered;
        let mut gain: f64 = self.gain_ctrl.get_value(time_triggered);
        let mut delta_t = self.delta_t_ctrl.get_value(time_triggered);
        while gain > 0.005 {
            // test this value
            source.scale(gain);
            wave.add(&source, seconds_to_samples(delta_t));
            current_time = self.time_keeper.add_seconds_to_stamp(current_time, delta_t);
            delta_t += self.delta_t_ctrl.get_value(current_time);
            gain *= self.gain_ctrl.get_value(current_time);
        }
    }
}
