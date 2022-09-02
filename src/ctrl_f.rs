use crate::time::TimeStamp;
use std::{collections::HashMap, fmt::Debug};

pub mod constant;
pub mod envelope;
pub mod lfo;
pub mod point_defined;

pub use constant::Constant;
pub use envelope::Envelope;
pub use lfo::Lfo;
pub use point_defined::PointDefined;

#[derive(Debug)]
pub enum Generator {
    Constant(Constant),
    Lfo(Lfo),
    PointDefined(PointDefined),
    Envelope(Envelope),
}

#[derive(Debug)]
pub struct Error;

impl Generator {
    fn get_val(&self, time: TimeStamp) -> Result<f64, Error> {
        match self {
            Generator::Constant(f) => Ok(f.get_val()),
            Generator::Lfo(f) => Ok(f.get_val(time)),
            Generator::PointDefined(f) => Ok(f.get_val(time)),
            Generator::Envelope(_) => Err(Error),
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
            _ => Err(Error),
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

    pub fn get_val(&self, key: u8, time: TimeStamp) -> Result<f64, Error> {
        match self.map.get(&key) {
            Some(gen) => gen.get_val(time),
            None => Err(Error),
        }
    }

    pub fn get_vec(&self, key: u8, start: TimeStamp, samples: usize) -> Result<Vec<f64>, Error> {
        match self.map.get(&key) {
            Some(gen) => Ok(gen.get_vec(start, samples)),
            None => Err(Error),
        }
    }

    pub fn get_envelope(
        &self,
        key: u8,
        time: TimeStamp,
        sus_samples: usize,
    ) -> Result<Vec<f64>, Error> {
        match self.map.get(&key) {
            Some(gen) => gen.get_envelope(sus_samples, time),
            None => Err(Error),
        }
    }
}

#[derive(Debug)]
pub struct TrackGManager {
    pub locals: GeneratorSave,
    pub instr: GeneratorSave,
}

#[derive(Debug, Default)]
pub struct GeneratorManager {
    globals: GeneratorSave,
    tracks: HashMap<u8, TrackGManager>,
}

impl GeneratorManager {
    pub fn get_val(&self, id: GeneratorId, time: TimeStamp) -> Result<f64, Error> {
        match id {
            GeneratorId::Global(key) => self.globals.get_val(key, time),
            GeneratorId::Track { track, key } => match self.tracks.get(&track) {
                Some(tgm) => tgm.locals.get_val(key, time),
                None => Err(Error),
            },
            GeneratorId::Instr { track, key } => match self.tracks.get(&track) {
                Some(tgm) => tgm.instr.get_val(key, time),
                None => Err(Error),
            },
        }
    }
}

#[derive(Debug)]
pub enum GeneratorId {
    Global(u8),
    Track { track: u8, key: u8 },
    Instr { track: u8, key: u8 },
}
