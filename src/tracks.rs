use crate::{
    ctrl_f::{FunctionKeeper, IdMapOrErr},
    time::{TimeKeeper, TimeManager},
    wave::Wave,
};
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
    pub fn set_function_manager(&mut self) {
        match self {
            Track::Midi(track) => track.set_function_manager(),
        }
    }
}

impl<W: Wave> FunctionKeeper for Track<W> {
    unsafe fn new_id(&mut self) {
        match self {
            Track::Midi(track) => track.new_id(),
        }
    }

    fn get_id_map(&self) -> IdMapOrErr {
        match self {
            Track::Midi(track) => track.get_id_map(),
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
