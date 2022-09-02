use super::{midi, MidiInstrument};
use crate::wave::Wave;
use std::marker::PhantomData;

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
