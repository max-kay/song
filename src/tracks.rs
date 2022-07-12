use crate::{time, wave};
use std::{cell::RefCell, rc::Rc};

pub mod midi;

pub use midi::MidiTrack;

#[derive(Debug)]
pub enum Track<'a, W: wave::Wave> {
    Midi(midi::MidiTrack<'a, W>),
}

impl<W: wave::Wave> time::TimeKeeper for Track<'_, W> {
    fn set_time_manager(&mut self, time_manager: Rc<RefCell<time::TimeManager>>) {
        match self {
            Track::Midi(track) => track.set_time_manager(Rc::clone(&time_manager)),
        }
    }
}

impl<W: wave::Wave> Track<'_, W> {
    pub fn set_automation_manager(&mut self) {
        match self {
            Track::Midi(track) => track.set_automation_manager(),
        }
    }
}

impl<W: wave::Wave + 'static> Track<'_, W> {
    pub fn play(&self) -> W {
        match self {
            Track::Midi(track) => track.play(),
        }
    }
}
