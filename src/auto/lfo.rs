use crate::{
    consts::SAMPLE_RATE,
    time::{self, TimeStamp},
    utils::oscs::Oscillator,
};
use std::{cell::RefCell, f64::consts::TAU, rc::Rc};

use super::CtrlFunction;

#[derive(Debug)]
pub struct Lfo {
    oscillator: Oscillator,
    freq: super::Control,
    modulation: super::Control,
    phase_shift: f64,
    time_manager: Rc<RefCell<time::TimeManager>>,
}

impl Lfo {
    pub fn new() -> Self {
        Self {
            oscillator: Oscillator::ModSaw,
            freq: super::Control::from_values(0.2, (0.001, 20.0)),
            modulation: super::Control::from_val_in_unit(0.5),
            phase_shift: 0.0,
            time_manager: Rc::new(RefCell::new(time::TimeManager::default())),
        }
    }
}

impl Lfo {
    pub fn set(&mut self, other: Lfo) {
        self.oscillator = other.oscillator;
        self.freq = other.freq;
        self.modulation = other.modulation;
        self.phase_shift = other.phase_shift;
        self.time_manager = other.time_manager;
    }
}

impl time::TimeKeeper for Lfo {
    fn set_time_manager(&mut self, time_manager: Rc<RefCell<time::TimeManager>>) {
        self.time_manager = Rc::clone(&time_manager)
    }
}

impl Default for Lfo {
    fn default() -> Self {
        Self::new()
    }
}

impl CtrlFunction for Lfo {
    fn get_value(&self, time: TimeStamp) -> f64 {
        let phase =
            ((self.time_manager.borrow().stamp_to_seconds(time) * TAU * self.freq.get_value(time)
                / (SAMPLE_RATE as f64))
                + self.phase_shift)
                % TAU;
        self.oscillator
            .get_sample(phase, self.modulation.get_value(time))
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
            .collect()
    }
}
