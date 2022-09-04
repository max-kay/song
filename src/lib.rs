#![warn(missing_debug_implementations)]

use std::{any::Any, collections::HashMap, u8};

use ctrl_f::GeneratorManager;
use globals::{GENRATOR_MANAGER, TIME_MANAGER};
use serde::{Deserialize, Serialize};
use time::TimeManager;
use tracks::{MidiTrack, Track};
use wave::Wave;

pub mod ctrl_f;
pub mod effects;
pub mod error;
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
    pub fn new(name: String) -> Self {
        Self {
            name,
            tracks: HashMap::new(),
        }
    }

    pub fn add_midi_track(&mut self, mut track: MidiTrack) -> Result<(), Error> {
        for i in 0..=u8::MAX {
            if let std::collections::hash_map::Entry::Vacant(e) = self.tracks.entry(i) {
                match track.put_in_song(i) {
                    Ok(_) => (),
                    Err(err) => match err {
                        Error::Overwrite => continue,
                        _ => todo!(),
                    },
                };
                e.insert(Track::Midi(track));
                return Ok(());
            }
        }
        Err(Error::Overflow)
    }

    pub fn get_instr_as_any(&mut self, track_id: u8) -> &mut dyn Any {
        match self.tracks.get_mut(&track_id) {
            Some(track) => track.get_instr_as_any(),
            None => todo!(),
        }
    }

    pub fn get_wave(&self) -> Wave {
        let mut wave = Wave::new();
        for track in self.tracks.values() {
            wave.add_consuming(track.play(), 0);
        }
        wave
    }
}

impl From<SongData> for Song{
    fn from(data: SongData) -> Self {
        *GENRATOR_MANAGER.write().unwrap() = data.generator_manager;
        *TIME_MANAGER.write().unwrap() = data.time_manager;
        Self {
            name: data.name,
            tracks: data.tracks,
        }
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct SongData {
    name: String,
    tracks: HashMap<u8, Track>,
    time_manager: TimeManager,
    generator_manager: GeneratorManager,
}

impl From<&Song> for SongData {
    fn from(song: &Song) -> Self {
        Self {
            name: song.name.clone(),
            tracks: song.tracks.clone(),
            time_manager: TIME_MANAGER.read().unwrap().clone(),
            generator_manager: GENRATOR_MANAGER.read().unwrap().clone(),
        }
    }
}
