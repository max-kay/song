#![warn(missing_debug_implementations)]

use std::{cell::RefCell, rc::Rc};
use time::TimeKeeper;

pub mod auto;
pub mod consts;
pub mod effects;
pub mod instr;
pub mod io;
pub mod time;
pub mod tracks;
pub mod utils;
pub mod wave;

#[derive(Debug)]
pub struct Song<W: wave::Wave> {
    name: String,
    tracks: Vec<tracks::Track<W>>,
    time_manager: Rc<RefCell<time::TimeManager>>,
}

impl<W: 'static + wave::Wave> Song<W> {
    pub fn set_time_manager(&mut self) {
        for track in &mut self.tracks {
            track.set_time_manager(Rc::clone(&self.time_manager))
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }
    pub fn new(name: String) -> Self {
        Self {
            name,
            tracks: Vec::new(),
            time_manager: Rc::new(RefCell::new(time::TimeManager::default())),
        }
    }

    pub fn add_midi_track(&mut self, track: tracks::MidiTrack<W>) {
        self.tracks.push(tracks::Track::Midi(track))
    }

    pub fn get_wave(&self) -> W {
        let mut wave = W::new();
        for track in &self.tracks {
            wave.add_consuming(track.play(), 0);
        }
        wave
    }
}
