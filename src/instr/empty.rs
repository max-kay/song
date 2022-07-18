use super::{midi, MidiInstrument};
use crate::{
    auto::{self, AutomationManager},
    time::{TimeKeeper, TimeManager},
    wave::Wave,
};
use std::{cell::RefCell, marker::PhantomData, rc::Rc};

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

impl<W: Wave> auto::AutomationKeeper for EmptyInstrument<W> {
    fn set_automation_manager(&mut self, _automation_manager: Rc<RefCell<AutomationManager>>) {}
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
