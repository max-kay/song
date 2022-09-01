use crate::{
    control::{ControlError, FunctionKeeper},
    ctrl_f::FunctionOwner,
    wave::Wave,
};

pub mod midi;

pub use midi::MidiTrack;

#[derive(Debug)]
pub enum Track<W: Wave> {
    Midi(midi::MidiTrack<W>),
}

impl<W: Wave> Track<W> {
    pub fn set_function_manager(&mut self) {
        match self {
            Track::Midi(track) => track.set_function_manager(),
        }
    }
}

impl<W: Wave> Track<W> {
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

impl<W: Wave + 'static> Track<W> {
    pub fn play(&self) -> W {
        match self {
            Track::Midi(track) => track.play(),
        }
    }
}
