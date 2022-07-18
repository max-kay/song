use crate::{
    consts::SAMPLE_RATE,
    time::{TimeKeeper, TimeManager, TimeStamp},
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
    id: usize,
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
            id: super::get_ctrl_id(),
        })
    }
}

impl Lfo {
    pub fn set(&mut self, other: Lfo) -> Result<(), ControlError> {
        self.set_freq(other.freq)?;
        self.set_modulation(other.modulation)?;
        self.oscillator = other.oscillator;
        self.phase_shift = other.phase_shift;
        self.time_manager = other.time_manager;
        Ok(())
    }

    pub fn set_freq(&mut self, freq_ctrl: Control) -> Result<(), ControlError> {
        if let Err(err) = self.freq.try_set(freq_ctrl){
            return Err(err.set_origin("Lfo", "frequency"))
        }
        Ok(())
    }

    pub fn set_modulation(&mut self, modulation_ctrl: Control) -> Result<(), ControlError> {
        if let Err(err) = self.modulation.try_set(modulation_ctrl){
            return Err(err.set_origin("Lfo", "modulation"))
        }
        Ok(())
    }
}

impl TimeKeeper for Lfo {
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

    fn get_id(&self) -> usize {
        self.id
    }

    fn get_sub_ids(&self) -> Vec<usize> {
        let mut ids = Vec::new();
        ids.append(&mut self.freq.get_ids());
        ids.append(&mut self.modulation.get_ids());
        ids
    }
}
