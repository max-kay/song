use super::{midi, MidiInstrument};
use crate::wave::Wave;

#[derive(Debug)]
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
