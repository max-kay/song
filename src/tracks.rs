use crate::{time, wave};
use std::{cell::RefCell, rc::Rc};

pub mod midi;

pub use midi::MidiTrack;

#[derive(Debug)]
pub enum Track<W: wave::Wave> {
    Midi(midi::MidiTrack<W>),
}

impl<W: wave::Wave> time::TimeKeeper for Track<W> {
    fn set_time_manager(&mut self, time_manager: Rc<RefCell<time::TimeManager>>) {
        match self {
            Track::Midi(track) => track.set_time_manager(Rc::clone(&time_manager)),
        }
    }
}

impl<W: wave::Wave> Track<W> {
    pub fn set_automation_manager(&mut self) {
        match self {
            Track::Midi(track) => track.set_automation_manager(),
        }
    }
}

impl<W: wave::Wave + 'static> Track<W> {
    pub fn play(&self) -> W {
        match self {
            Track::Midi(track) => track.play(),
        }
    }
}
