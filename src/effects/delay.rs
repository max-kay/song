use super::{Control, Effect};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::Rc;
use std::vec;

use crate::utils::seconds_to_samples;
use crate::wave;
use crate::{auto, time};

pub struct Delay<W: wave::Wave> {
    phantom: PhantomData<W>,
    time_manager: Rc<time::TimeManager>,
    on: bool,
}

impl<W: wave::Wave> time::TimeKeeper for Delay<W> {
    fn set_time_manager(&mut self, time_manager: &Rc<time::TimeManager>) {
        self.time_manager = Rc::clone(time_manager)
    }
}

impl<W: wave::Wave> Effect<W> for Delay<W> {
    fn apply(
        &self,
        wave: &mut W,
        controls: &HashMap<&str, Control>,
        time_triggered: time::TimeStamp,
    ) {
        let mut source = wave.clone();
        let gain_ctrl = controls
            .get("gain")
            .expect("mismatch between controls and value names");
        let delta_t_ctrl = controls
            .get("delta_t")
            .expect("mismatch between controls and value names");
        let mut current_time = time_triggered;
        let mut gain: f64 = gain_ctrl.get_value(time_triggered);
        let mut delta_t = delta_t_ctrl.get_value(time_triggered);
        while gain > 0.005 {
            // test this value
            source.scale(gain);
            wave.add(&source, seconds_to_samples(delta_t));
            current_time = self
                .time_manager
                .add_seconds_to_stamp(current_time, delta_t);
            delta_t += delta_t_ctrl.get_value(current_time);
            gain *= gain_ctrl.get_value(current_time);
        }
    }

    fn controls(&self) -> Vec<&str> {
        vec!["gain", "delta_t"]
    }

    fn default_controls(&self) -> std::collections::HashMap<&str, Control> {
        HashMap::from([
            (
                "gain",
                Control::from_values(0.7, 1.0),
            ),
            (
                "delta_t",
                Control::from_values(0.1, 1.0),
            ),
        ])
    }

    fn on(&mut self) {
        self.on = true
    }

    fn off(&mut self) {
        self.on = false
    }

    fn toggle(&mut self) {
        self.on = !self.on
    }
}
