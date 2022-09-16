use crate::{
    globals::TIME_MANAGER,
    network::{Receiver, Transform},
    time::ClockTick,
    utils,
    wave::Wave,
};
use serde::{Deserialize, Serialize};

const SMALLEST_GAIN_ALLOWED: f32 = 0.05;
const GAIN_RECEIVER: Receiver = Receiver::new(0.6, (0.0, 0.95), Transform::Linear);
const DELTA_T_RECEIVER: Receiver = Receiver::new(0.6, (0.001, 6.0), Transform::Linear);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delay {
    on: bool,
    gain: Receiver,
    delta_t: Receiver,
}

impl Delay {
    pub fn new() -> Self {
        Self {
            on: true,
            gain: GAIN_RECEIVER,
            delta_t: DELTA_T_RECEIVER,
        }
    }
}

impl Default for Delay {
    fn default() -> Self {
        Self::new()
    }
}

impl Delay {
    pub fn apply(&self, wave: &mut Wave, time_triggered: ClockTick) {
        let mut source = wave.clone();
        let mut current_time = time_triggered;
        let mut gain: f32 = self.gain.get_val(time_triggered);
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

    pub fn set_defaults(&mut self) {
        self.gain = GAIN_RECEIVER;
        self.delta_t = DELTA_T_RECEIVER;
    }

    pub fn on(&mut self) {
        self.on = true
    }

    pub fn off(&mut self) {
        self.on = false
    }

    pub fn toggle(&mut self) {
        self.on = !self.on
    }
}

impl Delay {
    pub fn extract(&self) -> Self {
        Self {
            on: self.on,
            gain: self.gain.extract(),
            delta_t: self.delta_t.extract(),
        }
    }

    pub fn set_id(&mut self, track_id: u8) {
        self.gain.set_id(track_id);
        self.delta_t.set_id(track_id);
    }
}
