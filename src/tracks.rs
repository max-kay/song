use crate::{
    control::{ControlError, FunctionKeeper},
    ctrl_f::FunctionOwner,
    time::{TimeKeeper, TimeManager},
    wave::Wave,
};
use std::{cell::RefCell, rc::Rc};

pub mod midi;

pub use midi::MidiTrack;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Track {
    Midi(midi::MidiTrack),
}

impl TimeKeeper for Track {
    fn set_time_manager(&mut self, time_manager: Rc<RefCell<TimeManager>>) {
        match self {
            Track::Midi(track) => track.set_time_manager(Rc::clone(&time_manager)),
        }
    }
}

impl Track {
    pub fn set_function_manager(&mut self) {
        match self {
            Track::Midi(track) => track.set_function_manager(),
        }
    }
}

impl Track {
    pub fn new_ids(&mut self) -> Result<(), ControlError> {
        match self {
            Track::Midi(track) => {
                unsafe { track.new_ids() };
                track.set_ids();
                track.test_sources()
            }
        }
    }
}

impl Track {
    pub fn play(&self) -> Wave {
        match self {
            Track::Midi(track) => track.play(),
        }
    }
}
