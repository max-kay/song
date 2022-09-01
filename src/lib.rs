#![warn(missing_debug_implementations)]

use serde::{
    de::{Deserialize, Deserializer, Visitor},
    ser::{Serialize, SerializeStruct, Serializer},
};
use std::{cell::RefCell, rc::Rc, fmt};
use time::{TimeKeeper, TimeManager};
use tracks::Track;
use wave::Wave;

pub mod consts;
pub mod control;
pub mod ctrl_f;
pub mod effects;
pub mod instr;
pub mod io;
pub mod time;
pub mod tracks;
pub mod utils;
pub mod wave;

#[derive(Debug)]
pub struct Song {
    name: String,
    tracks: Vec<Track>,
    time_manager: Rc<RefCell<TimeManager>>,
}

impl Song {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            tracks: Vec::new(),
            time_manager: Rc::new(RefCell::new(TimeManager::default())),
        }
    }

    pub fn add_midi_track(&mut self, track: tracks::MidiTrack) {
        self.tracks.push(Track::Midi(track))
    }

    pub fn set_time_manager(&mut self) {
        for track in &mut self.tracks {
            track.set_time_manager(Rc::clone(&self.time_manager))
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_wave(&self) -> Wave {
        let mut wave = Wave::new();
        for track in &self.tracks {
            wave.add_consuming(track.play(), 0);
        }
        wave
    }
}

impl Serialize for Song {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Song", 3)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("tracks", &self.tracks)?;
        state.serialize_field("time_manager", &self.time_manager.borrow().clone())?;
        state.end()
    }
}

struct SongVisitor;
impl<'de> Visitor<'de> for SongVisitor {
    type Value = Song;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a Song")
    }


}

impl<'de> Deserialize<'de> for Song {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        todo!()
    }
}
