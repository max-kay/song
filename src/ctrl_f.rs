use crate::{time::TimeStamp, Error};
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

#[derive(Debug, Clone, Copy)]
pub enum GenId {
    Unbound,
    Global(u8),
    Track { track: u8, key: u8 },
    Instr { track: u8, key: u8 },
}

impl GenId {
    pub fn decompose(&self) -> Result<(SaveId, u8), Error> {
        match self {
            GenId::Unbound => Err(Error::Unbound),
            GenId::Global(key) => Ok((SaveId::Global, *key)),
            GenId::Track { track, key } => Ok((SaveId::Track(*track), *key)),
            GenId::Instr { track, key } => Ok((SaveId::Instr(*track), *key)),
        }
    }
}

impl Default for GenId {
    fn default() -> Self {
        Self::Unbound
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SaveId {
    Unbound,
    Global,
    Track(u8),
    Instr(u8),
}

impl SaveId {
    pub fn add_key(&self, key: u8) -> GenId {
        match self {
            SaveId::Unbound => GenId::Unbound,
            SaveId::Global => GenId::Global(key),
            SaveId::Track(track) => GenId::Track { track: *track, key },
            SaveId::Instr(track) => GenId::Instr { track: *track, key },
        }
    }
}

impl Default for SaveId {
    fn default() -> Self {
        Self::Unbound
    }
}

#[derive(Debug)]
pub enum Generator {
    Constant(Constant),
    Lfo(Lfo),
    PointDefined(PointDefined),
    Envelope(Envelope),
}

impl Generator {
    fn get_val(&self, time: TimeStamp) -> Result<f64, Error> {
        match self {
            Generator::Constant(f) => Ok(f.get_val()),
            Generator::Lfo(f) => Ok(f.get_val(time)),
            Generator::PointDefined(f) => Ok(f.get_val(time)),
            Generator::Envelope(_) => Err(Error::Type),
        }
    }

    fn get_vec(&self, start: TimeStamp, samples: usize) -> Vec<f64> {
        match self {
            Generator::Constant(f) => f.get_vec(samples),
            Generator::Lfo(f) => f.get_vec(start, samples),
            Generator::PointDefined(f) => f.get_vec(start, samples),
            Generator::Envelope(f) => f.get_vec(start, samples),
        }
    }

    fn get_envelope(&self, sus_samples: usize, time: TimeStamp) -> Result<Vec<f64>, Error> {
        match self {
            Generator::Envelope(envelope) => Ok(envelope.get_envelope(sus_samples, time)),
            _ => Err(Error::Type),
        }
    }

    fn set_const(&mut self, val: f64) -> Result<(), Error> {
        match self {
            Generator::Constant(con) => {
                con.set(val);
                Ok(())
            },
            _ => Err(Error::Type),
        }
    }
}

#[derive(Debug, Default)]
pub struct GeneratorSave {
    map: HashMap<u8, Generator>,
}

impl GeneratorSave {
    pub fn new() -> Self {
        Self {
            map: HashMap::default(),
        }
    }

    pub fn add_generator(&mut self, gen: Generator) -> Result<u8, Error> {
        for i in 0..=u8::MAX {
            if let Entry::Vacant(e) = self.map.entry(i) {
                e.insert(gen);
                return Ok(i);
            }
        }
        Err(Error::Overflow)
    }
}

impl GeneratorSave {
    pub fn get_val(&self, key: &u8, time: TimeStamp) -> Result<f64, Error> {
        match self.map.get(key) {
            Some(gen) => gen.get_val(time),
            None => Err(Error::Existance),
        }
    }

    pub fn get_vec(&self, key: &u8, start: TimeStamp, samples: usize) -> Result<Vec<f64>, Error> {
        match self.map.get(key) {
            Some(gen) => Ok(gen.get_vec(start, samples)),
            None => Err(Error::Existance),
        }
    }

    pub fn get_envelope(
        &self,
        key: &u8,
        note_on: TimeStamp,
        sus_samples: usize,
    ) -> Result<Vec<f64>, Error> {
        match self.map.get(key) {
            Some(gen) => gen.get_envelope(sus_samples, note_on),
            None => Err(Error::Existance),
        }
    }

    pub fn set_const(&mut self, key: &u8, val: f64) -> Result<(), Error> {
        match self.map.entry(*key) {
            Entry::Occupied(mut gen) => gen.get_mut().set_const(val),
            Entry::Vacant(_) => Err(Error::Existance),
        }
    }
}

#[derive(Debug)]
pub struct TrackGManager {
    pub track_id: u8,
    pub locals: GeneratorSave,
    pub instr: GeneratorSave,
}

impl TrackGManager {
    pub fn new(id: u8) -> Self {
        Self {
            track_id: id,
            locals: Default::default(),
            instr: Default::default(),
        }
    }
}

#[derive(Debug, Default)]
pub struct GeneratorManager {
    globals: GeneratorSave,
    tracks: HashMap<u8, TrackGManager>,
}

impl GeneratorManager {
    fn get(&self, id: SaveId) -> Result<&GeneratorSave, Error> {
        match id {
            SaveId::Unbound => Err(Error::Unbound),
            SaveId::Global => Ok(&self.globals),
            SaveId::Track(track) => match self.tracks.get(&track) {
                Some(tgm) => Ok(&tgm.locals),
                None => Err(Error::Existance),
            },
            SaveId::Instr(track) => match self.tracks.get(&track) {
                Some(tgm) => Ok(&tgm.instr),
                None => Err(Error::Existance),
            },
        }
    }
    fn get_mut(&mut self, id: SaveId) -> Result<&mut GeneratorSave, Error> {
        match id {
            SaveId::Unbound => Err(Error::Unbound),
            SaveId::Global => Ok(&mut self.globals),
            SaveId::Track(track) => match self.tracks.get_mut(&track) {
                Some(tgm) => Ok(&mut tgm.locals),
                None => Err(Error::Existance),
            },
            SaveId::Instr(track) => match self.tracks.get_mut(&track) {
                Some(tgm) => Ok(&mut tgm.instr),
                None => Err(Error::Existance),
            },
        }
    }
}

impl GeneratorManager {
    pub fn get_val(&self, id: GenId, time: TimeStamp) -> Result<f64, Error> {
        let (id, key) = id.decompose()?;
        let save = self.get(id)?;
        save.get_val(&key, time)
    }

    pub fn get_vec(&self, id: GenId, start: TimeStamp, samples: usize) -> Result<Vec<f64>, Error> {
        let (id, key) = id.decompose()?;
        let save = self.get(id)?;
        save.get_vec(&key, start, samples)
    }

    pub fn get_envelope(
        &self,
        id: GenId,
        note_on: TimeStamp,
        sus_samples: usize,
    ) -> Result<Vec<f64>, Error> {
        let (id, key) = id.decompose()?;
        let save = self.get(id)?;
        save.get_envelope(&key, note_on, sus_samples)
    }

    pub fn set_const(&mut self, id: GenId, val: f64) -> Result<(), Error> {
        let (id, key) = id.decompose()?;
        let save = self.get_mut(id)?;
        save.set_const(&key, val)
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

    pub fn add_generator(&mut self, gen: Generator, id: SaveId) -> Result<GenId, Error> {
        let save = self.get_mut(id)?;
        Ok(id.add_key(save.add_generator(gen)?))
    }
}
