use serde::{Deserialize, Serialize};

use super::{midi, MidiInstrument};
use crate::{
    control::{ControlError, FunctionKeeper},
    ctrl_f::{FunctionManager, FunctionMngrKeeper, FunctionOwner, IdMap, IdMapOrErr},
    time::{TimeKeeper, TimeManager},
    wave::Wave,
};
use std::{cell::RefCell,  rc::Rc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmptyInstrument;
impl EmptyInstrument {
    pub fn new() -> Self {
        EmptyInstrument
    }
}

impl Default for EmptyInstrument {
    fn default() -> Self {
        Self::new()
    }
}

impl TimeKeeper for EmptyInstrument {
    fn set_time_manager(&mut self, _time_manager: Rc<RefCell<TimeManager>>) {}
}

impl FunctionKeeper for EmptyInstrument {
    fn get_ids(&self) -> Vec<usize> {
        Vec::new()
    }

    fn heal_sources(&mut self, _id_map: &IdMap) -> Result<(), ControlError> {
        Ok(())
    }

    fn test_sources(&self) -> Result<(), ControlError> {
        Ok(())
    }

    fn set_ids(&mut self) {}
}

impl FunctionOwner for EmptyInstrument {
    unsafe fn new_ids(&mut self) {}

    fn get_id_map(&self) -> IdMapOrErr {
        Ok(IdMap::new())
    }
}

impl FunctionMngrKeeper for EmptyInstrument {
    fn set_fuction_manager(&mut self, _function_manager: Rc<RefCell<FunctionManager>>) {}
}

#[typetag::serde]
impl MidiInstrument for EmptyInstrument {
    fn play_note(&self, _note: midi::Note) -> Wave {
        Wave::new()
    }
    fn play_notes(&self, _notes: &[midi::Note]) -> Wave {
        Wave::new()
    }
    fn name(&self) -> &str {
        "empty"
    }
}
