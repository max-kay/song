use crate::{
    ctrl_f::{Control, ControlError},
    globals::{SAMPLE_RATE, TIME_MANAGER},
    time::TimeStamp,
    utils::oscs::Oscillator,
};
use std::f64::consts::TAU;

use super::CtrlFunction;

const FREQ_RANGE: (f64, f64) = (0.001, 20.0);

#[derive(Debug)]
pub struct Lfo {
    oscillator: Oscillator,
    freq: Control,
    modulation: Control,
    phase_shift: f64,
}

impl Lfo {
    pub fn new(
        oscillator: Oscillator,
        freq: f64,
        modulation: f64,
        phase_shift: f64,
    ) -> Result<Self, ControlError> {
        Ok(Self {
            oscillator,
            freq: Control::from_val_in_range(freq, FREQ_RANGE)
                .map_err(|err| err.set_origin("Lfo", "frequency"))?,
            modulation: Control::from_val_in_unit(modulation)
                .map_err(|err| err.set_origin("Lfo", "modulation"))?,
            phase_shift,
        })
    }
}

impl Lfo {
    pub fn set(&mut self, other: Lfo) -> Result<(), ControlError> {
        self.set_freq(other.freq)?;
        self.set_modulation(other.modulation)?;
        self.oscillator = other.oscillator;
        self.phase_shift = other.phase_shift;
        Ok(())
    }

    pub fn set_freq(&mut self, freq_ctrl: Control) -> Result<(), ControlError> {
        todo!()
    }

    pub fn set_modulation(&mut self, modulation_ctrl: Control) -> Result<(), ControlError> {
        todo!()
    }
}

impl Default for Lfo {
    fn default() -> Self {
        Self::new(Oscillator::ModSaw, 2.0, 0.0, 0.0).expect("error in Lfo::Default")
    }
}

impl CtrlFunction for Lfo {
    fn get_value(&self, time: TimeStamp) -> f64 {
        let phase = ((TIME_MANAGER.lock().unwrap().stamp_to_seconds(time)
            * TAU
            * self.freq.get_value(time)
            / (SAMPLE_RATE as f64))
            + self.phase_shift)
            % TAU;
        (self
            .oscillator
            .get_sample(phase, self.modulation.get_value(time))
            + 1.0)
            / 2.0
    }

    fn get_vec(&self, start: TimeStamp, samples: usize) -> Vec<f64> {
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
