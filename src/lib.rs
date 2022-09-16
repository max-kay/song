#![warn(missing_debug_implementations)]

use io::data::SongBuilder;
use std::{collections::HashMap, fs::File, path::Path, u8};
use tracks::{MidiTrack, Track};
use wave::Wave;

pub mod effects;
pub mod error;
pub mod gens;
pub mod globals;
pub mod instr;
pub mod io;
pub mod network;
pub mod time;
pub mod tracks;
pub mod utils;
pub mod wave;

pub use error::Error;

#[derive(Debug)]
pub struct Song {
    name: String,
    tracks: HashMap<u8, Track>,
}

impl Song {
    pub fn get_name(&self) -> &str {
        &self.name
    }
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            tracks: HashMap::new(),
        }
    }

    pub fn get_wave(&self) -> Wave {
        let mut wave = Wave::new();
        for track in self.tracks.values() {
            wave.add(&track.play(), 0);
        }
        wave
    }

    pub fn mut_midi_tracks(&mut self) -> Vec<&mut MidiTrack> {
        let mut out = Vec::new();
        for track in self.tracks.values_mut() {
            match track {
                Track::Midi(track) => out.push(track),
            }
        }
        out
    }
}

impl Song {
    pub fn save_to(&self, path: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
        let data: SongBuilder = self.into();
        let file = File::create(path)?;
        ron::ser::to_writer_pretty(file, &data, Default::default())?;
        // serde_json::to_writer_pretty(file, &data)?;
        Ok(())
    }

    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        SongBuilder::from_path(path).map(|data| data.into())
    }

    pub fn from_midi(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        SongBuilder::from_midi(path).map(|data| data.into())
    }
}
