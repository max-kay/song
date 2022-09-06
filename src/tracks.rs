use std::any::Any;

use crate::{wave::Wave, Error};

pub mod midi;

pub use midi::MidiTrack;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Track {
    Midi(midi::MidiTrack),
}

impl Track {
    pub fn put_in_song(&mut self, id: u8) -> Result<(), Error> {
        match self {
            Track::Midi(track) => track.put_in_song(id),
        }
    }

    pub fn get_instr_as_any(&mut self) -> &mut dyn Any {
        match self {
            Track::Midi(track) => track.get_instr_as_any(),
        }
    }

    pub fn play(&self) -> Wave {
        match self {
            Track::Midi(track) => track.play(),
        }
    }
}
