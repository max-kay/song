use crate::{
    ctrl_f::Error,
    globals::{SAMPLE_RATE, TIME_MANAGER},
    network::{Reciever, Transform},
    time::TimeStamp,
    utils::oscs::Oscillator,
};
use std::f64::consts::TAU;

use super::Generator;

const FREQ_RECIEVER: Reciever = Reciever::new(2.0, (0.001, 20.0), Transform::Linear);
const MOD_RECIEVER: Reciever = Reciever::new(0.5, (0.0, 1.0), Transform::Linear);

#[derive(Debug)]
pub struct Lfo {
    oscillator: Oscillator,
    freq: Reciever,
    modulation: Reciever,
    phase_shift: f64,
}

impl Lfo {
    pub fn new() -> Self {
        Self {
            oscillator: Oscillator::Sine,
            freq: FREQ_RECIEVER,
            modulation: MOD_RECIEVER,
            phase_shift: 0.0,
        }
    }

    pub fn w_default() -> Generator {
        Generator::Lfo(Self::default())
    }
}

impl Lfo {
    pub fn set(&mut self, other: Lfo) -> Result<(), Error> {
        todo!()
    }

    pub fn set_freq(&mut self, freq_ctrl: Reciever) -> Result<(), Error> {
        todo!()
    }

    pub fn set_modulation(&mut self, modulation_ctrl: Reciever) -> Result<(), Error> {
        todo!()
    }
}

impl Default for Lfo {
    fn default() -> Self {
        Self::new()
    }
}

impl Lfo {
    pub fn get_val(&self, time: TimeStamp) -> f64 {
        let phase =
            ((TIME_MANAGER.lock().unwrap().stamp_to_seconds(time) * TAU * self.freq.get_val(time)
                / (SAMPLE_RATE as f64))
                + self.phase_shift)
                % TAU;
        (self
            .oscillator
            .get_sample(phase, self.modulation.get_val(time))
            + 1.0)
            / 2.0
    }

    pub fn get_vec(&self, start: TimeStamp, samples: usize) -> Vec<f64> {
        self.oscillator
            .play_shifted(
                &self.freq.get_vec(start, samples),
                &self.modulation.get_vec(start, samples),
                samples,
                self.phase_shift,
            )
            .into_iter()
            .map(|x| (x + 1.0) / 2.0)
            .collect()
    }
}
