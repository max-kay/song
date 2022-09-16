use crate::wave::Wave;
use serde::{Deserialize, Serialize};

pub mod midi;
pub use midi::MidiTrack;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Track {
    Midi(midi::MidiTrack),
}

impl Track {
    pub fn play(&self) -> Wave {
        match self {
            Track::Midi(track) => track.play(),
        }
    }

    pub fn get_name(&self) -> &str {
        match self {
            Track::Midi(track) => track.get_name(),
        }
    }
}
