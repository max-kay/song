use crate::{
    gens::{
        point_defined::Interpolation, Constant, GenId, Generator, GeneratorManager, PointDefined,
        Specific, TI,
    },
    globals::{GENRATOR_MANAGER, RESOURCE_MANAGER, TIME_MANAGER},
    instr::MidiInstrument,
    resources::ResourceManager,
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
    resource_manager: ResourceManager,
}

impl SongBuilder {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            tracks: HashMap::new(),
            time_manager: TimeManager::default(),
            generator_manager: GeneratorManager::new(),
            resource_manager: ResourceManager::default(),
        }
    }

    pub fn add_track_data(&mut self, mut track: MidiTrackBuilder) -> Result<(), Error> {
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

                // mod_wheel
                match track.gen_data.entry(1) {
                    Entry::Occupied(e) => {
                        let id = GenId::Specific {
                            track_id,
                            kind: Specific::ModW,
                        };
                        let mut mod_wheel = Generator::PointDefined(PointDefined::from_xy_pairs(
                            e.remove(),
                            Interpolation::Step,
                        ));
                        mod_wheel.set_id(id);
                        *self.generator_manager.get_mut_or_new(id).unwrap() = mod_wheel
                    }
                    Entry::Vacant(_) => (),
                }

                // generators
                for (i, points) in track.gen_data.into_iter() {
                    let mut channel = Generator::PointDefined(PointDefined::from_xy_pairs(
                        points,
                        Interpolation::Step,
                    ));
                    let id = GenId::put_together(Some((track_id, TI::Track)), i);
                    channel.set_id(id);
                    *self.generator_manager.get_mut_or_new(id).unwrap() = channel;
                }

                // pitchbend
                let id = GenId::Specific {
                    track_id,
                    kind: Specific::Pitch,
                };
                if !track.pitch_bend.is_empty() {
                    let mut pitchbend = Generator::PointDefined(PointDefined::from_xy_pairs(
                        track.pitch_bend,
                        Interpolation::Step,
                    ));
                    pitchbend.set_id(id);
                    *self.generator_manager.get_mut_or_new(id).unwrap() = pitchbend;
                } else {
                    self.generator_manager
                        .get_mut_or_new(id)
                        .unwrap()
                        .set_id(id);
                }

                // // channel after touch
                // if !track.ch_after_touch.is_empty() {
                //     let ch_after_touch = Generator::PointDefined(PointDefined::from_xy_pairs(
                //         track.ch_after_touch,
                //         Interpolation::Step,
                //     )); // set id !!!
                //     let ch_after_touch_id = self
                //         .generator_manager
                //         .add_generator(ch_after_touch, id)
                //         .unwrap();
                //     decoded_track.add_ch_after_touch(ch_after_touch_id);
                // }

                let id = GenId::Specific {
                    track_id,
                    kind: Specific::Vel,
                };
                let mut vel = Constant::w_default();
                vel.set_id(id);
                *self.generator_manager.get_mut_or_new(id).unwrap() = vel;

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
            resource_manager: RESOURCE_MANAGER.read().unwrap().extract(),
        }
    }
}

impl TryFrom<SongBuilder> for Song {
    type Error = Box<dyn std::error::Error>;
    fn try_from(data: SongBuilder) -> Result<Self, Box<dyn std::error::Error>> {
        *GENRATOR_MANAGER.write().unwrap() = data.generator_manager;
        *TIME_MANAGER.write().unwrap() = data.time_manager;
        *RESOURCE_MANAGER.write().unwrap() = data.resource_manager;
        RESOURCE_MANAGER.write().unwrap().init()?;
        Ok(Self {
            name: data.name,
            tracks: data.tracks,
        })
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

