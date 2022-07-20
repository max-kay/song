use super::{midi, MidiInstrument};
use crate::{
    control::{ControlError, SourceKeeper},
    ctrl_f::{FunctionKeeper, FunctionManager, FunctionMngrKeeper, IdMap, IdMapOrErr},
    time::{TimeKeeper, TimeManager},
    wave::Wave,
};
use std::{cell::RefCell, collections::HashMap, marker::PhantomData, rc::Rc};

#[derive(Debug)]
pub struct EmptyInstrument<W: Wave> {
    phantom: PhantomData<W>,
}
impl<W: Wave> EmptyInstrument<W> {
    pub fn new() -> Self {
        EmptyInstrument {
            phantom: PhantomData,
        }
    }
}

impl<W: Wave> Default for EmptyInstrument<W> {
    fn default() -> Self {
        Self::new()
    }
}

impl<W: Wave> TimeKeeper for EmptyInstrument<W> {
    fn set_time_manager(&mut self, _time_manager: Rc<RefCell<TimeManager>>) {}
}

impl<W: Wave> SourceKeeper for EmptyInstrument<W> {
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

impl<W: Wave> FunctionKeeper for EmptyInstrument<W> {
    unsafe fn new_id(&mut self) {}

    fn get_id_map(&self) -> IdMapOrErr {
        Ok(HashMap::new())
    }
}

impl<W: Wave> FunctionMngrKeeper for EmptyInstrument<W> {
    fn set_fuction_manager(&mut self, _function_manager: Rc<RefCell<FunctionManager>>) {}
}

impl<W: Wave> MidiInstrument<W> for EmptyInstrument<W> {
    fn play_note(&self, _note: midi::Note) -> W {
        W::new()
    }
    fn play_notes(&self, _notes: &[midi::Note]) -> W {
        W::new()
    }
    fn name(&self) -> &str {
        "empty"
    }
}
