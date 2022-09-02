use crate::wave::Wave;

pub mod midi;

pub use midi::MidiTrack;

#[derive(Debug)]
pub enum Track {
    Midi(midi::MidiTrack),
}

impl Track {
    pub fn play(&self) -> Wave {
        match self {
            Track::Midi(track) => track.play(),
        }
    }
}
