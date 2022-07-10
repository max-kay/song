use std::marker::PhantomData;
use crate::wave;
use super::{MidiInstrument, midi};

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

impl<W: wave::Wave> MidiInstrument<W> for EmptyInstrument<W> {
    fn play_note(&self, _note: midi::Note) -> W {
        W::new()
    }
    fn play_notes(&self, _notes: &Vec<midi::Note>) -> W {
        W::new()
    }
    fn name(&self) -> &str {
        "empty"
    }
}

