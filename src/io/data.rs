use crate::{
    gens::{
        point_defined::Interpolation, Constant, GenId, Generator, GeneratorManager, PointDefined,
        TI,
    },
    globals::{GENRATOR_MANAGER, TIME_MANAGER},
    instr::MidiInstrument,
    time::{ClockTick, TimeManager},
    tracks::{
        midi::{self, MidiTrack},
        Track,
    },
    utils::XYPairs,
    Error, Song,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{hash_map::Entry, HashMap},
    fs::File,
    path::Path,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct SongBuilder {
    name: String,
    tracks: HashMap<u8, Track>,
    time_manager: TimeManager,
    generator_manager: GeneratorManager,
}

impl SongBuilder {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            tracks: HashMap::new(),
            time_manager: TimeManager::default(),
            generator_manager: GeneratorManager::new(),
        }
    }

    pub fn add_track_data(&mut self, track: MidiTrackBuilder) -> Result<(), Error> {
        if track.notes.is_empty() {
            // TODO think about if this is a good idea
            return Ok(());
        }
        match self.tracks.entry(track.track_nr) {
            Entry::Occupied(_) => Err(Error::Overwrite),
            Entry::Vacant(e) => {
                let track_id = track.track_nr;
                let mut decoded_track = MidiTrack::new(track_id);
                self.generator_manager.new_track(track_id).expect("received an invalid track_id even though this should already be tested at this point");

                decoded_track.set_name(track.name);

                // notes
                decoded_track.add_notes(track.notes);

                // generators
                for (i, points) in track.gen_data.into_iter() {
                    let channel = Generator::PointDefined(PointDefined::from_xy_pairs(
                        points,
                        Interpolation::Step,
                    ));
                    *self
                        .generator_manager
                        .get_mut_or_new(GenId::put_together(Some((track_id, TI::Track)), i))
                        .unwrap() = channel;
                }

                // pitchbend
                if !track.pitch_bend.is_empty() {
                    let pitchbend = Generator::PointDefined(PointDefined::from_xy_pairs(
                        track.pitch_bend,
                        Interpolation::Step,
                    ));
                    *self
                        .generator_manager
                        .get_mut_or_new(GenId::Specific {
                            track_id,
                            kind: crate::gens::Specific::Pitch,
                        })
                        .unwrap() = pitchbend;
                }

                // // channel after touch
                // if !track.ch_after_touch.is_empty() {
                //     let ch_after_touch = Generator::PointDefined(PointDefined::from_xy_pairs(
                //         track.ch_after_touch,
                //         Interpolation::Step,
                //     ));
                //     let ch_after_touch_id = self
                //         .generator_manager
                //         .add_generator(ch_after_touch, id)
                //         .unwrap();
                //     decoded_track.add_ch_after_touch(ch_after_touch_id);
                // }

                *self
                    .generator_manager
                    .get_mut_or_new(GenId::Specific {
                        track_id,
                        kind: crate::gens::Specific::ModW,
                    })
                    .unwrap() = PointDefined::w_val(0.0).unwrap();

                *self
                    .generator_manager
                    .get_mut_or_new(GenId::Specific {
                        track_id,
                        kind: crate::gens::Specific::Vel,
                    })
                    .unwrap() = Constant::w_default();

                decoded_track.set_inst_unchecked(MidiInstrument::named_empty(&track.inst_name));

                e.insert(Track::Midi(decoded_track));
                Ok(())
            }
        }
    }

    pub fn set_time_manager(&mut self, tm: TimeManager) {
        self.time_manager = tm
    }
}

impl Default for SongBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl From<&Song> for SongBuilder {
    fn from(song: &Song) -> Self {
        Self {
            name: song.name.clone(),
            tracks: song.tracks.clone(),
            time_manager: TIME_MANAGER.read().unwrap().clone(),
            generator_manager: GENRATOR_MANAGER.read().unwrap().clone(),
        }
    }
}

impl From<SongBuilder> for Song {
    fn from(data: SongBuilder) -> Self {
        *GENRATOR_MANAGER.write().unwrap() = data.generator_manager;
        *TIME_MANAGER.write().unwrap() = data.time_manager;
        Self {
            name: data.name,
            tracks: data.tracks,
        }
    }
}

impl SongBuilder {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let rdr = File::open(path)?;
        Ok(ron::de::from_reader(rdr)?)
    }

    pub fn from_midi(path: impl AsRef<Path>) -> Result<SongBuilder, Box<dyn std::error::Error>> {
        super::parse_midi_file(path)
    }
}

#[derive(Debug)]
pub struct MidiTrackBuilder {
    pub(super) name: String,
    pub(super) inst_name: String,
    pub(super) track_nr: u8,
    pub(super) notes: Vec<midi::Note>,
    pub(super) gen_data: HashMap<u8, XYPairs<ClockTick, f32>>,
    pub(super) pitch_bend: XYPairs<ClockTick, f32>,
    pub(super) _ch_after_touch: XYPairs<ClockTick, f32>,
}

// #[derive(Debug, Serialize, Deserialize)]
// pub enum InstrData {
//     Empty(String),
//     Synthesizer(SynthesierBuilder),
// }
