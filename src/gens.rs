use crate::{time::ClockTick, Error};
use std::{
    collections::{hash_map::Entry, HashMap},
    fmt::Debug,
};

pub mod constant;
pub mod envelope;
pub mod lfo;
pub mod point_defined;

pub use constant::Constant;
pub use envelope::Envelope;
pub use lfo::Lfo;
pub use point_defined::PointDefined;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GenId {
    Global(u8),

    Track { track_id: u8, key: u8 },

    Instr { track_id: u8, key: u8 },
    InstrExtracted { key: u8 },

    Specific { track_id: u8, kind: Specific },
    SpecificExtracted { kind: Specific },

    Unbound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Specific {
    Vel,
    ModW,
    Pitch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TI {
    Track,
    Instr,
}

impl GenId {
    pub fn extract(&self) -> Result<Self, Error> {
        match self {
            GenId::Global(_) => Err(Error::Type),

            GenId::Track {
                track_id: _,
                key: _,
            } => Err(Error::Type),

            GenId::Instr { track_id: _, key } => Ok(Self::InstrExtracted { key: *key }),
            GenId::InstrExtracted { key: _ } => Ok(*self),

            GenId::Specific { track_id: _, kind } => Ok(Self::SpecificExtracted { kind: *kind }),
            GenId::SpecificExtracted { kind: _ } => Ok(*self),

            GenId::Unbound => Err(Error::Unbound),
        }
    }

    pub fn set_id(&mut self, track_id: u8) {
        match self {
            GenId::Track { track_id: _, key } => {
                *self = GenId::Track {
                    track_id,
                    key: *key,
                }
            }

            GenId::Instr { track_id: _, key } => {
                *self = GenId::Instr {
                    track_id,
                    key: *key,
                }
            }
            GenId::InstrExtracted { key } => {
                *self = GenId::Instr {
                    track_id,
                    key: *key,
                }
            }

            GenId::Specific { track_id: _, kind } => {
                *self = GenId::Specific {
                    track_id,
                    kind: *kind,
                }
            }
            GenId::SpecificExtracted { kind } => {
                *self = GenId::Specific {
                    track_id,
                    kind: *kind,
                }
            }

            GenId::Global(_) => (),
            GenId::Unbound => (),
        }
    }
}

impl GenId {
    pub fn put_together(id: Option<(u8, TI)>, key: u8) -> Self {
        if let Some((track_id, kind)) = id {
            match kind {
                TI::Track => Self::Track { track_id, key },
                TI::Instr => Self::Instr { track_id, key },
            }
        } else {
            Self::Global(key)
        }
    }
}

impl Default for GenId {
    fn default() -> Self {
        Self::Unbound
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Generator {
    Empty,
    Constant(Constant),
    Lfo(Lfo),
    PointDefined(PointDefined),
    Envelope(Envelope),
}

impl Generator {
    pub fn get_sub_ids(&self) -> Vec<GenId> {
        match self {
            Generator::Constant(f) => f.get_sub_ids(),
            Generator::Lfo(f) => f.get_sub_ids(),
            Generator::PointDefined(_) => Vec::new(),
            Generator::Envelope(f) => f.get_sub_ids(),
            Generator::Empty => Vec::new(),
        }
    }

    pub fn set_id(&mut self, id: GenId) {
        match self {
            Generator::Constant(gen) => gen.set_id(id),
            Generator::Lfo(gen) => gen.set_id(id),
            Generator::PointDefined(gen) => gen.set_id(id),
            Generator::Envelope(gen) => gen.set_id(id),
            Generator::Empty => (),
        }
    }
}

impl Generator {
    fn get_val(&self, time: ClockTick) -> Result<f32, Error> {
        match self {
            Generator::Constant(f) => Ok(f.get_val()),
            Generator::Lfo(f) => Ok(f.get_val(time)),
            Generator::PointDefined(f) => Ok(f.get_val(time)),
            Generator::Envelope(_) => Err(Error::Type),
            Generator::Empty => Err(Error::Existence),
        }
    }

    fn get_vec(&self, start: ClockTick, samples: usize) -> Vec<f32> {
        match self {
            Generator::Constant(f) => f.get_vec(samples),
            Generator::Lfo(f) => f.get_vec(start, samples),
            Generator::PointDefined(f) => f.get_vec(start, samples),
            Generator::Envelope(f) => f.get_vec(start, samples),
            Generator::Empty => todo!(),
        }
    }

    fn get_envelope(&self, note_on: ClockTick, sus_samples: usize) -> Result<Vec<f32>, Error> {
        match self {
            Generator::Envelope(envelope) => Ok(envelope.get_envelope(note_on, sus_samples)),
            _ => Err(Error::Type),
        }
    }

    fn set_const(&mut self, val: f32) -> Result<(), Error> {
        match self {
            Generator::Constant(con) => {
                con.set(val);
                Ok(())
            }
            _ => Err(Error::Type),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratorSave {
    id: Option<(u8, TI)>,
    map: HashMap<u8, Generator>,
}

impl GeneratorSave {
    pub fn new(id: Option<(u8, TI)>) -> Self {
        Self {
            id,
            map: HashMap::new(),
        }
    }

    pub fn get(&self, key: u8) -> Result<&Generator, Error> {
        match self.map.get(&key) {
            Some(gen) => Ok(gen),
            None => Err(Error::Existence),
        }
    }

    pub fn get_mut(&mut self, key: u8) -> Result<&mut Generator, Error> {
        match self.map.get_mut(&key) {
            Some(gen) => Ok(&mut *gen),
            None => Err(Error::Existence),
        }
    }

    pub fn get_mut_or_new(&mut self, key: u8) -> &mut Generator {
        if let Entry::Vacant(e) = self.map.entry(key) {
            e.insert(Generator::Empty);
        };
        self.get_mut(key).unwrap()
    }

    pub fn get_sub_ids(&self, key: u8) -> Result<Vec<GenId>, Error> {
        match self.map.get(&key) {
            Some(gen) => Ok(gen.get_sub_ids()),
            None => Err(Error::Existence),
        }
    }

    pub fn add_generator(&mut self, mut gen: Generator) -> Result<GenId, Error> {
        for key in 0..=u8::MAX {
            if let Entry::Vacant(e) = self.map.entry(key) {
                let id = GenId::put_together(self.id, key);
                gen.set_id(id);
                e.insert(gen);
                return Ok(id);
            }
        }
        Err(Error::Overflow)
    }
}

impl GeneratorSave {
    pub fn get_val(&self, key: &u8, time: ClockTick) -> Result<f32, Error> {
        match self.map.get(key) {
            Some(gen) => gen.get_val(time),
            None => Err(Error::Existence),
        }
    }

    pub fn get_vec(&self, key: &u8, start: ClockTick, samples: usize) -> Result<Vec<f32>, Error> {
        match self.map.get(key) {
            Some(gen) => Ok(gen.get_vec(start, samples)),
            None => Err(Error::Existence),
        }
    }

    pub fn set_const(&mut self, key: &u8, val: f32) -> Result<(), Error> {
        match self.map.entry(*key) {
            Entry::Occupied(mut gen) => gen.get_mut().set_const(val),
            Entry::Vacant(_) => Err(Error::Existence),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenSaveBuilder {
    map: HashMap<u8, Generator>,
}

impl GenSaveBuilder {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn insert_gen(&mut self, gen: Generator) -> Result<u8, Error> {
        for i in 0..=u8::MAX {
            match self.map.entry(i) {
                Entry::Occupied(_) => continue,
                Entry::Vacant(e) => {
                    e.insert(gen);
                    return Ok(i);
                }
            }
        }
        Err(Error::Overflow)
    }

    pub fn as_generator_save(self, track_id: u8, kind: TI) -> GeneratorSave {
        GeneratorSave {
            id: Some((track_id, kind)),
            map: self.map,
        }
    }
}

impl Default for GenSaveBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl From<&GeneratorSave> for GenSaveBuilder {
    fn from(save: &GeneratorSave) -> Self {
        GenSaveBuilder {
            map: save.map.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackGManager {
    track_id: u8,
    pub pitchbend: Generator,
    pub velocity: Generator,
    pub mod_wheel: Generator,
    // pub channel_after_touch: Option<PointDefined>,
    pub track: GeneratorSave,
    pub instr: GeneratorSave,
}

impl TrackGManager {
    pub fn new(id: u8) -> Self {
        Self {
            track_id: id,
            pitchbend: PointDefined::new_val(0.5).unwrap().wrap(),
            velocity: Constant::new().wrap(),
            mod_wheel: PointDefined::new_val(0.0).unwrap().wrap(),
            // channel_after_touch: None,
            track: GeneratorSave::new(Some((id, TI::Track))),
            instr: GeneratorSave::new(Some((id, TI::Instr))),
        }
    }

    pub fn get_specific(&self, kind: Specific) -> &Generator {
        match kind {
            Specific::Vel => &self.velocity,
            Specific::ModW => &self.mod_wheel,
            Specific::Pitch => &self.pitchbend,
        }
    }

    pub fn get_specific_mut(&mut self, kind: Specific) -> &mut Generator {
        match kind {
            Specific::Vel => &mut self.velocity,
            Specific::ModW => &mut self.mod_wheel,
            Specific::Pitch => &mut self.pitchbend,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeneratorManager {
    globals: GeneratorSave,
    tracks: HashMap<u8, TrackGManager>,
}

impl GeneratorManager {
    pub fn new() -> Self {
        Self {
            globals: GeneratorSave::new(None),
            tracks: HashMap::new(),
        }
    }
}

impl Default for GeneratorManager {
    fn default() -> Self {
        Self::new()
    }
}

impl GeneratorManager {
    fn get(&self, id: GenId) -> Result<&Generator, Error> {
        match id {
            GenId::Global(key) => self.globals.get(key),
            GenId::Track { track_id, key } => match self.tracks.get(&track_id) {
                Some(tmg) => tmg.track.get(key),
                None => Err(Error::Existence),
            },
            GenId::Instr { track_id, key } => match self.tracks.get(&track_id) {
                Some(tmg) => tmg.instr.get(key),
                None => Err(Error::Existence),
            },
            GenId::InstrExtracted { key: _ } => Err(Error::Unbound),
            GenId::Specific { track_id, kind } => match self.tracks.get(&track_id) {
                Some(tgm) => Ok(tgm.get_specific(kind)),
                None => Err(Error::Existence),
            },
            GenId::SpecificExtracted { kind: _ } => Err(Error::Unbound),
            GenId::Unbound => Err(Error::Unbound),
        }
    }

    fn get_mut(&mut self, id: GenId) -> Result<&mut Generator, Error> {
        match id {
            GenId::Global(key) => self.globals.get_mut(key),
            GenId::Track { track_id, key } => match self.tracks.get_mut(&track_id) {
                Some(tgm) => tgm.track.get_mut(key),
                None => Err(Error::Existence),
            },
            GenId::Instr { track_id, key } => match self.tracks.get_mut(&track_id) {
                Some(tgm) => tgm.instr.get_mut(key),
                None => Err(Error::Existence),
            },
            GenId::InstrExtracted { key: _ } => Err(Error::Unbound),
            GenId::Specific { track_id, kind } => match self.tracks.get_mut(&track_id) {
                Some(tgm) => Ok(tgm.get_specific_mut(kind)),
                None => Err(Error::Existence),
            },
            GenId::SpecificExtracted { kind: _ } => Err(Error::Unbound),
            GenId::Unbound => Err(Error::Unbound),
        }
    }

    pub fn get_mut_or_new(&mut self, id: GenId) -> Result<&mut Generator, Error> {
        match id {
            GenId::Global(key) => Ok(self.globals.get_mut_or_new(key)),
            GenId::Track { track_id, key } => match self.tracks.get_mut(&track_id) {
                Some(tgm) => Ok(tgm.track.get_mut_or_new(key)),
                None => Err(Error::Existence),
            },
            GenId::Instr { track_id, key } => match self.tracks.get_mut(&track_id) {
                Some(tgm) => Ok(tgm.instr.get_mut_or_new(key)),
                None => Err(Error::Existence),
            },
            GenId::InstrExtracted { key: _ } => Err(Error::Unbound),
            GenId::Specific { track_id, kind } => match self.tracks.get_mut(&track_id) {
                Some(tgm) => Ok(tgm.get_specific_mut(kind)),
                None => Err(Error::Existence),
            },
            GenId::SpecificExtracted { kind: _ } => Err(Error::Unbound),
            GenId::Unbound => Err(Error::Unbound),
        }
    }
}

impl GeneratorManager {
    pub fn get_sub_ids(&self, id: GenId) -> Result<Vec<GenId>, Error> {
        Ok(self.get(id)?.get_sub_ids())
    }
}

impl GeneratorManager {
    pub fn get_val(&self, id: GenId, time: ClockTick) -> Result<f32, Error> {
        self.get(id)?.get_val(time)
    }

    pub fn get_vec(&self, id: GenId, start: ClockTick, samples: usize) -> Result<Vec<f32>, Error> {
        Ok(self.get(id)?.get_vec(start, samples))
    }

    pub fn get_envelope(
        &self,
        id: GenId,
        note_on: ClockTick,
        sus_samples: usize,
    ) -> Result<Vec<f32>, Error> {
        match id {
            GenId::Instr { track_id, key } => match self.tracks.get(&track_id) {
                Some(tgm) => tgm.instr.get(key)?.get_envelope(note_on, sus_samples),
                None => Err(Error::Existence),
            },
            _ => Err(Error::Type),
        }
    }

    pub fn set_const(&mut self, id: GenId, val: f32) -> Result<(), Error> {
        self.get_mut(id)?.set_const(val)
    }
}

impl GeneratorManager {
    pub fn new_track(&mut self, id: u8) -> Result<(), Error> {
        if let Entry::Vacant(e) = self.tracks.entry(id) {
            e.insert(TrackGManager::new(id));
            return Ok(());
        }
        Err(Error::Overwrite)
    }

    pub fn add_track_generator(
        &mut self,
        gen: Generator,
        track_id: u8,
        kind: TI,
    ) -> Result<GenId, Error> {
        match self.tracks.get_mut(&track_id) {
            Some(tgm) => match kind {
                TI::Track => tgm.track.add_generator(gen),
                TI::Instr => tgm.instr.add_generator(gen),
            },
            None => Err(Error::Existence),
        }
    }

    pub fn get_instr_save(&self, track_id: u8) -> Result<&GeneratorSave, Error> {
        match self.tracks.get(&track_id) {
            Some(tgm) => Ok(&tgm.instr),
            None => Err(Error::Existence),
        }
    }

    pub fn get_mut_instr_save(&mut self, track_id: u8) -> Result<&mut GeneratorSave, Error> {
        match self.tracks.get_mut(&track_id) {
            Some(tgm) => Ok(&mut tgm.instr),
            None => Err(Error::Existence),
        }
    }
}
