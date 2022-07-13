use super::{midi, MidiInstrument};
use crate::{auto, time, wave};
use std::{cell::RefCell, marker::PhantomData, rc::Rc};

#[derive(Debug)]
pub struct EmptyInstrument<W: wave::Wave> {
    phantom: PhantomData<W>,
}
impl<W: wave::Wave> EmptyInstrument<W> {
    pub fn new() -> Self {
        EmptyInstrument {
            phantom: PhantomData,
        }
    }
}

impl<W: wave::Wave> Default for EmptyInstrument<W> {
    fn default() -> Self {
        Self::new()
    }
}

impl<W: wave::Wave> time::TimeKeeper for EmptyInstrument<W> {
    fn set_time_manager(&mut self, _time_manager: Rc<RefCell<time::TimeManager>>) {}
}

impl<W: wave::Wave> auto::AutomationKeeper for EmptyInstrument<W> {
    fn set_automation_manager(
        &mut self,
        _automation_manager: Rc<std::cell::RefCell<auto::AutomationManager>>,
    ) {
    }
}

impl<W: wave::Wave> MidiInstrument<W> for EmptyInstrument<W> {
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
