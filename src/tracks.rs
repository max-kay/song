use crate::wave::Wave;

pub mod midi;

pub use midi::MidiTrack;

#[derive(Debug)]
pub enum Track<W: Wave> {
    Midi(midi::MidiTrack<W>),
}

impl<W: Wave + 'static> Track<W> {
    pub fn play(&self) -> W {
        match self {
            Track::Midi(track) => track.play(),
        }
    }
}
