use std::{cell::RefCell, rc::Rc};

use serde::{Deserialize, Serialize};

use crate::{
    control::{Control, ControlError, FunctionKeeper},
    ctrl_f::IdMap,
    time::{TimeKeeper, TimeManager, TimeStamp},
    wave::Wave,
};

use super::Effect;

const VOL_RANGE: (f64, f64) = (0.0, 5.0);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Volume {
    volume: Control,
    on: bool,
}

impl Volume {
    pub fn new() -> Self {
        Self {
            volume: Control::from_val_in_range(1.0, VOL_RANGE).unwrap(),
            on: true,
        }
    }
}

impl Default for Volume {
    fn default() -> Self {
        Self::new()
    }
}

impl TimeKeeper for Volume {
    fn set_time_manager(&mut self, time_manager: Rc<RefCell<TimeManager>>) {
        self.volume.set_time_manager(time_manager)
    }
}

impl FunctionKeeper for Volume {
    fn heal_sources(&mut self, id_map: &IdMap) -> Result<(), ControlError> {
        self.volume
            .heal_sources(id_map)
            .map_err(|err| err.set_origin("Volume", "volume"))
    }

    fn test_sources(&self) -> Result<(), ControlError> {
        self.volume
            .test_sources()
            .map_err(|err| err.set_origin("Volume", "volume"))
    }

    fn set_ids(&mut self) {
        self.volume.set_ids()
    }

    fn get_ids(&self) -> Vec<usize> {
        self.volume.get_ids()
    }
}

#[typetag::serde]
impl Effect for Volume {
    fn apply(&self, wave: &mut Wave, time_triggered: TimeStamp) {
        if self.on {
            let vol = self.volume.get_vec(time_triggered, wave.len());
            wave.scale_by_vec(vol)
        }
    }

    fn set_defaults(&mut self) {
        self.volume.set_value(1.0).unwrap()
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
