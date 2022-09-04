use crate::{wave::Wave, Error};

pub mod midi;

pub use midi::MidiTrack;

#[derive(Debug)]
pub enum Track {
    Midi(midi::MidiTrack),
}

impl Track {
    pub fn put_in_song(&mut self, id: u8) -> Result<(), Error> {
        match self {
            Track::Midi(track) => track.put_in_song(id),
        }
    }

    pub fn play(&self) -> Wave {
        match self {
            Track::Midi(track) => track.play(),
        }
    }
}
