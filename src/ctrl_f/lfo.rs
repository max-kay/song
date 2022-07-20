use crate::{
    consts::SAMPLE_RATE,
    control::{Control, ControlError},
    time::{TimeKeeper, TimeManager, TimeStamp},
    utils::{self, oscs::Oscillator},
};
use std::{cell::RefCell, f64::consts::TAU, rc::Rc};

use super::{CtrlFunction, IdMap, SourceKeeper};

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
            freq: Control::from_val_in_range(freq, FREQ_RANGE)
                .map_err(|err| err.set_origin("Lfo", "frequency"))?,
            modulation: Control::from_val_in_unit(modulation)
                .map_err(|err| err.set_origin("Lfo", "modulation"))?,
            phase_shift,
            time_manager: Rc::new(RefCell::new(TimeManager::default())),
            id: utils::get_ctrl_id(),
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
        self.freq
            .try_set_checked(freq_ctrl, self.id)
            .map_err(|err| err.set_origin("Lfo", "frequency"))
    }

    pub fn set_modulation(&mut self, modulation_ctrl: Control) -> Result<(), ControlError> {
        self.modulation
            .try_set(modulation_ctrl)
            .map_err(|err| err.set_origin("Lfo", "modulation"))
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

impl SourceKeeper for Lfo {
    fn heal_sources(&mut self, id_map: &IdMap) -> Result<(), ControlError> {
        self.freq
            .heal_sources(id_map)
            .map_err(|err| err.set_origin("Lfo", "frequency"))?;
        self.modulation
            .heal_sources(id_map)
            .map_err(|err| err.set_origin("Lfo", "frequency"))
    }

    fn test_sources(&self) -> Result<(), ControlError> {
        self.freq
            .test_sources()
            .map_err(|err| err.set_origin("Lfo", "frequency"))?;
        self.modulation.test_sources()
    }

    fn set_ids(&mut self) {
        self.freq.set_ids();
        self.modulation.set_ids();
    }

    fn get_ids(&self) -> Vec<usize> {
        let mut ids = vec![self.get_id()];
        ids.append(&mut self.freq.get_ids());
        ids.append(&mut self.modulation.get_ids());
        ids
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

    unsafe fn new_id_f(&mut self) {
        self.id = utils::get_ctrl_id()
    }

    // fn get_sub_ids(&self) -> Vec<usize> {
    //     let mut ids = self.freq.get_ids();
    //     ids.append(&mut self.modulation.get_ids());
    //     ids
    // }
}
