use serde::{Serialize, Deserialize};

use super::Effect;
use crate::{
    globals::TIME_MANAGER,
    network::{Reciever, Transform},
    time::TimeStamp,
    utils,
    wave::Wave,
};

const SMALLEST_GAIN_ALLOWED: f64 = 0.05;
const GAIN_RECIEVER: Reciever = Reciever::new(0.6, (0.0, 0.95), Transform::Linear);
const DELTA_T_RECIEVER: Reciever = Reciever::new(0.6, (0.001, 6.0), Transform::Linear);

#[derive(Debug, Serialize, Deserialize)]
pub struct Delay {
    on: bool,
    gain: Reciever,
    delta_t: Reciever,
}

impl Delay {
    pub fn new() -> Self {
        Self {
            on: true,
            gain: GAIN_RECIEVER,
            delta_t: DELTA_T_RECIEVER,
        }
    }
}

impl Default for Delay {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for Delay {
    fn apply(&self, wave: &mut Wave, time_triggered: TimeStamp) {
        let mut source = wave.clone();
        let mut current_time = time_triggered;
        let mut gain: f64 = self.gain.get_val(time_triggered);
        let mut delta_t = self.delta_t.get_val(time_triggered);
        while gain > SMALLEST_GAIN_ALLOWED {
            source.scale(gain);
            wave.add(&source, utils::seconds_to_samples(delta_t));
            current_time = TIME_MANAGER
                .read()
                .unwrap()
                .add_seconds_to_stamp(current_time, delta_t);
            delta_t += self.delta_t.get_val(current_time);
            gain *= self.gain.get_val(current_time);
        }
    }

    fn set_defaults(&mut self) {
        self.gain = GAIN_RECIEVER;
        self.delta_t = DELTA_T_RECIEVER;
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
