use crate::{
    consts::SAMPLE_RATE,
    time::{self, TimeManager, TimeStamp},
    utils::oscs::Oscillator,
};
use std::{cell::RefCell, f64::consts::TAU, rc::Rc};

use super::{Control, ControlError, CtrlFunction};

const FREQ_RANGE: (f64, f64) = (0.001, 20.0);

#[derive(Debug)]
pub struct Lfo {
    oscillator: Oscillator,
    freq: Control,
    modulation: Control,
    phase_shift: f64,
    time_manager: Rc<RefCell<TimeManager>>,
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
            freq: match Control::from_val_in_range(freq, FREQ_RANGE) {
                Ok(ctrl) => ctrl,
                Err(err) => return Err(err.set_origin("Lfo", "Frequency")),
            },
            modulation: match Control::from_val_in_unit(modulation) {
                Ok(ctrl) => ctrl,
                Err(err) => return Err(err.set_origin("Lfo", "Modulation")),
            },
            phase_shift,
            time_manager: Rc::new(RefCell::new(TimeManager::default())),
        })
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
    fn set_time_manager(&mut self, time_manager: Rc<RefCell<TimeManager>>) {
        self.time_manager = Rc::clone(&time_manager)
    }
}

impl Default for Lfo {
    fn default() -> Self {
        Self::new(Oscillator::ModSaw, 2.0, 0.0, 0.0).expect("error in Lfo::Default")
    }
}

impl CtrlFunction for Lfo {
    fn get_value(&self, time: TimeStamp) -> f64 {
        let phase =
            ((self.time_manager.borrow().stamp_to_seconds(time) * TAU * self.freq.get_value(time)
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
