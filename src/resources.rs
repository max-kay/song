use std::{
    collections::{hash_map::Entry, HashMap},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{io, wave::Wave, Error};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ResourceManager {
    #[serde(skip)]
    samples: HashMap<SampleId, Wave>,
    sample_path: HashMap<SampleId, PathBuf>,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct SampleId(u32);

impl ResourceManager {
    pub fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let erroring: Vec<(SampleId, PathBuf)> = self
            .sample_path
            .iter()
            .map(|(id, path)| -> Result<(), (&SampleId, &PathBuf)> {
                let wave = match io::read_wav(path) {
                    Ok(wave) => wave,
                    Err(_) => return Err((id, path)),
                };
                self.samples.insert(*id, wave);
                Ok(())
            })
            .filter(|x| x.is_err())
            .map(|x| x.unwrap_err())
            .map(|(id, path)| (*id, path.clone()))
            .collect();
        if !erroring.is_empty() {
            Err(Error::Sample(erroring))?
        }
        Ok(())
    }

    pub fn extract(&self) -> Self {
        Self {
            samples: Default::default(),
            sample_path: self.sample_path.clone(),
        }
    }
}
impl ResourceManager {
    pub fn get_sample(&self, id: SampleId) -> Wave {
        self.samples
            .get(&id)
            .expect("resource manager wasn't initialised correctly")
            .clone()
    }

    pub fn add_sample(
        &mut self,
        path: impl AsRef<Path> + Clone,
    ) -> Result<SampleId, Box<dyn std::error::Error>> {
        for index in 0..u32::MAX {
            let id = SampleId(index);
            match self.sample_path.entry(id) {
                Entry::Occupied(_) => continue,
                Entry::Vacant(e) => {
                    let mut buf = PathBuf::new();
                    buf.push(path.clone());
                    e.insert(buf);
                    self.samples.insert(id, io::read_wav(path)?);
                    return Ok(id);
                }
            };
        }
        Err(Error::Overflow)?
    }

    pub fn get_path(&self, id: SampleId) -> Result<PathBuf, Error> {
        match self.sample_path.get(&id) {
            Some(path) => Ok(path.clone()),
            None => Err(Error::Existence),
        }
    }
}
