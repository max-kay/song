use serde::{Deserialize, Serialize};

use super::{Control, Effect};
use crate::{
    control::{ControlError, FunctionKeeper},
    ctrl_f::IdMap,
    time::{TimeKeeper, TimeManager, TimeStamp},
    utils,
    wave::Wave,
};
use std::{cell::RefCell, rc::Rc};

const SMALLEST_GAIN_ALLOWED: f64 = 0.05;
const GAIN_RANGE: (f64, f64) = (0.0, 0.95);
const DELTA_T_RANGE: (f64, f64) = (0.001, 6.0);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delay {
    #[serde(skip)]
    time_manager: Rc<RefCell<TimeManager>>,
    on: bool,
    gain: Control,
    delta_t: Control,
}

impl Delay {
    pub fn new() -> Self {
        Self {
            time_manager: Rc::new(RefCell::new(TimeManager::default())),
            on: true,
            gain: Control::from_val_in_range(0.6, GAIN_RANGE).unwrap(),
            delta_t: Control::from_val_in_range(0.6, DELTA_T_RANGE).unwrap(),
        }
    }
}

impl Default for Delay {
    fn default() -> Self {
        Self::new()
    }
}

impl TimeKeeper for Delay {
    fn set_time_manager(&mut self, time_manager: Rc<RefCell<TimeManager>>) {
        self.time_manager = Rc::clone(&time_manager);
        self.gain.set_time_manager(Rc::clone(&time_manager));
        self.delta_t.set_time_manager(Rc::clone(&time_manager));
    }
}

impl FunctionKeeper for Delay {
    fn heal_sources(&mut self, id_map: &IdMap) -> Result<(), ControlError> {
        self.delta_t
            .heal_sources(id_map)
            .map_err(|err| err.set_origin("Delay", "delta_t"))?;
        self.gain
            .heal_sources(id_map)
            .map_err(|err| err.set_origin("Delay", "gain"))
    }

    fn test_sources(&self) -> Result<(), ControlError> {
        self.delta_t
            .test_sources()
            .map_err(|err| err.set_origin("Delay", "delta_t"))?;
        self.gain
            .test_sources()
            .map_err(|err| err.set_origin("Delay", "gain"))
    }

    fn set_ids(&mut self) {
        self.delta_t.set_ids();
        self.gain.set_ids()
    }

    fn get_ids(&self) -> Vec<usize> {
        let mut ids = self.delta_t.get_ids();
        ids.append(&mut self.gain.get_ids());
        ids
    }
}

#[typetag::serde]
impl Effect for Delay {
    fn apply(&self, wave: &mut Wave, time_triggered: TimeStamp) {
        let mut source = wave.clone();
        let mut current_time = time_triggered;
        let mut gain: f64 = self.gain.get_value(time_triggered);
        let mut delta_t = self.delta_t.get_value(time_triggered);
        while gain > SMALLEST_GAIN_ALLOWED {
            source.scale(gain);
            wave.add(&source, utils::seconds_to_samples(delta_t));
            current_time = self
                .time_manager
                .borrow()
                .add_seconds_to_stamp(current_time, delta_t);
            delta_t += self.delta_t.get_value(current_time);
            gain *= self.gain.get_value(current_time);
        }
    }

    fn set_defaults(&mut self) {
        self.gain = Control::from_val_in_range(0.6, GAIN_RANGE).unwrap();
        self.delta_t = Control::from_val_in_range(0.6, DELTA_T_RANGE).unwrap();
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
