use std::any::Any;

use serde::{Serialize, Deserialize};

use super::{midi, MidiInstrument};
use crate::{wave::Wave, Error};

#[derive(Debug, Serialize, Deserialize)]
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
    fn put_in_song(&mut self, _id: u8) -> Result<(), Error> {
        Ok(())
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}
