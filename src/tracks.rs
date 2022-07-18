use crate::{time::{TimeManager, TimeKeeper}, wave::{ Wave}};
use std::{cell::RefCell, rc::Rc};

pub mod midi;

pub use midi::MidiTrack;

#[derive(Debug)]
pub enum Track<W: Wave> {
    Midi(midi::MidiTrack<W>),
}

impl<W: Wave> TimeKeeper for Track<W> {
    fn set_time_manager(&mut self, time_manager: Rc<RefCell<TimeManager>>) {
        match self {
            Track::Midi(track) => track.set_time_manager(Rc::clone(&time_manager)),
        }
    }
}

impl<W: Wave> Track<W> {
    pub fn set_automation_manager(&mut self) {
        match self {
            Track::Midi(track) => track.set_automation_manager(),
        }
    }
}

impl<W: Wave + 'static> Track<W> {
    pub fn play(&self) -> W {
        match self {
            Track::Midi(track) => track.play(),
        }
    }
}
