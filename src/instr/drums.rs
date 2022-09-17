use std::{
    collections::HashMap,
    fs::File,
    path::{Path, PathBuf},
};

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::{
    effects::EffectPanel,
    globals::{RESOURCE_MANAGER, TIME_MANAGER},
    network::Receiver,
    receivers::VOL_RECEIVER,
    resources::SampleId,
    tracks::midi::{Note, Pitch},
    wave::Wave,
    Error,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Drums {
    pub(crate) name: String,
    pub(crate) effects: EffectPanel,
    pub(crate) volume: Receiver,
    pub(crate) samples: HashMap<Pitch, SampleId>,
}

impl Drums {
    pub fn play_note(&self, note: Note) -> Wave {
        let mut wave = RESOURCE_MANAGER.read().unwrap().get_sample(
            *self
                .samples
                .get(&note.pitch)
                .expect("drums played unmapped sample"),
        );
        wave.scale(note.velocity);
        wave.scale_by_vec(self.volume.get_vec(note.on, wave.len()));
        wave
    }

    pub fn play_notes(&self, notes: &[Note]) -> Wave {
        let mut wave = Wave::new();
        for note in notes {
            wave.add(
                &self.play_note(*note),
                TIME_MANAGER.read().unwrap().tick_to_sample(note.on),
            )
        }
        wave
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }
}

impl Drums {
    pub fn extract(&self) -> Result<DrumsBuilder, Error> {
        let mut samples = HashMap::new();
        for (pitch, id) in &self.samples {
            samples.insert(*pitch, RESOURCE_MANAGER.read().unwrap().get_path(*id)?);
        }
        Ok(DrumsBuilder {
            name: self.name.clone(),
            effects: self.effects.extract(),
            volume: self.volume.extract(),
            samples,
        })
    }

    pub fn save_to(&self, path: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::create(path)?;
        ron::ser::to_writer_pretty(file, &self.extract()?, Default::default())?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DrumsBuilder {
    pub(crate) name: String,
    pub(crate) effects: EffectPanel,
    pub(crate) volume: Receiver,
    pub(crate) samples: HashMap<Pitch, PathBuf>,
}

impl DrumsBuilder {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let drums = ron::de::from_reader(file)?;
        Ok(drums)
    }
}

impl Default for DrumsBuilder {
    fn default() -> Self {
        Self {
            name: "drums".into(),
            effects: EffectPanel::EmptyLeaf,
            volume: VOL_RECEIVER.sv(30.0), // TODO
            samples: HashMap::from([
                (Pitch::new(36).unwrap(), PathBuf::from("samples/kick.wav")),
                (Pitch::new(38).unwrap(), PathBuf::from("samples/snare.wav")),
                (Pitch::new(42).unwrap(), PathBuf::from("samples/hihat.wav")),
                (
                    Pitch::new(46).unwrap(),
                    PathBuf::from("samples/hihat_half.wav"),
                ),
            ]),
        }
    }
}

#[allow(dead_code)]
static DRUM_MAP: Lazy<HashMap<u8, &str>> = Lazy::new(|| {
    HashMap::from([
        (35, "Acoustic Bass Drum"),
        (36, "Bass Drum 1"),
        (37, "Side Stick"),
        (38, "Acoustic Snare"),
        (39, "Hand Clap"),
        (40, "Electric Snare"),
        (41, "Low Floor Tom"),
        (42, "Closed Hi Hat"),
        (43, "High Floor Tom"),
        (44, "Pedal Hi-Hat"),
        (45, "Low Tom"),
        (46, "Open Hi-Hat"),
        (47, "Low-Mid Tom"),
        (48, "Hi Mid Tom"),
        (49, "Crash Cymbal 1"),
        (50, "High Tom"),
        (51, "Ride Cymbal 1"),
        (52, "Chinese Cymbal"),
        (53, "Ride Bel"),
        (54, "Tambourine"),
        (55, "Splash Cymbal"),
        (56, "Cowbell"),
        (57, "Crash Cymbal "),
        (58, "Vibraslap"),
        (59, "Ride Cymbal 2"),
        (60, "Hi Bongo"),
        (61, "Low Bongo"),
        (62, "Mute Hi Conga"),
        (63, "Open Hi Conga"),
        (64, "Low Conga"),
        (65, "High Timbale"),
        (66, "Low Timbale"),
        (67, "High Agogo"),
        (68, "Low Agogo"),
        (69, "Cabasa"),
        (70, "Maracas"),
        (71, "Short Whistle"),
        (72, "Long Whistle"),
        (73, "Short Guiro"),
        (74, "Long Guiro"),
        (75, "Claves"),
        (76, "Hi Wood Block"),
        (77, "Low Wood Block"),
        (78, "Mute Cuica"),
        (79, "Open Cuica"),
        (80, "Mute Triangle"),
        (81, "Open Triangle"),
    ])
});
