#![warn(missing_debug_implementations)]

use tracks::Track;
use wave::Wave;

pub mod ctrl_f;
pub mod effects;
pub mod globals;
pub mod instr;
pub mod io;
pub mod network;
pub mod time;
pub mod tracks;
pub mod utils;
pub mod wave;

#[derive(Debug)]
pub struct Song {
    name: String,
    tracks: Vec<Track>,
}

impl Song {
    pub fn get_name(&self) -> &str {
        &self.name
    }
    pub fn new(name: String) -> Self {
        Self {
            name,
            tracks: Vec::new(),
        }
    }

    pub fn add_midi_track(&mut self, track: tracks::MidiTrack) {
        self.tracks.push(Track::Midi(track))
    }

    pub fn get_wave(&self) -> Wave {
        let mut wave = Wave::new();
        for track in &self.tracks {
            wave.add_consuming(track.play(), 0);
        }
        wave
    }
}
