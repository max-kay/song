pub mod midi;
use crate::time;
use crate::wave;
pub use midi::MidiTrack;
use std::rc::Rc;

pub enum Track<W: wave::Wave> {
    Midi(midi::MidiTrack<W>),
}

impl<W: wave::Wave + 'static> Track<W> {
    pub fn play(&self) -> W {
        match self {
            Track::Midi(track) => track.play(),
        }
    }
    pub fn new(time_keeper: Rc<time::TimeKeeper>) -> Self {
        todo!()
    }
    pub fn from_instrument(time_keeper: Rc<time::TimeKeeper>) -> Self {
        todo!();
    }
}
