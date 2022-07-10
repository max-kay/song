pub mod midi;
use crate::instruments;
use crate::time;
use crate::wave;
pub use midi::MidiTrack;
use std::rc::Rc;

pub enum Track<'a, W: wave::Wave> {
    Midi(midi::MidiTrack<'a, W>),
}

impl<W: wave::Wave> time::TimeKeeper for Track<'_, W> {
    fn set_time_manager(&mut self, time_manager: &Rc<time::TimeManager>) {
        match self {
            Track::Midi(track) => track.set_time_manager(time_manager),
        }
    }
}

impl<'a, W: wave::Wave + 'static> Track<'a, W> {
    pub fn play(&self) -> W {
        match self {
            Track::Midi(track) => track.play(),
        }
    }

    //this lifetime may be a bad idea
    pub fn from_instrument<I: 'static + instruments::MidiInstrument<W>>(
        time_manager: Rc<time::TimeManager>,
        instrument: Box<I>,
    ) -> Self {
        Track::Midi(MidiTrack::from_instrument(
            instrument,
            Rc::clone(&time_manager),
        ))
    }
}
