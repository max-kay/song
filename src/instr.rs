use crate::{tracks::midi, wave::Wave};
use serde::{Deserialize, Serialize};
use std::{fs::File, path::Path};
pub use synth::Synthesizer;
pub mod synth;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MidiInstrument {
    Synthesizer(Box<Synthesizer>),
    Empty { name: String },
}

impl MidiInstrument {
    pub fn empty() -> Self {
        Self::Empty {
            name: "".to_string(),
        }
    }

    pub fn named_empty(name: &str) -> Self {
        Self::Empty {
            name: name.to_string(),
        }
    }

    pub fn play_note(&self, note: midi::Note) -> Wave {
        match self {
            MidiInstrument::Synthesizer(synth) => synth.play_note(note),
            MidiInstrument::Empty { name: _ } => Wave::new(),
        }
    }

    pub fn play_notes(&self, notes: &[midi::Note]) -> Wave {
        match self {
            MidiInstrument::Synthesizer(synth) => synth.play_notes(notes),
            MidiInstrument::Empty { name: _ } => Wave::new(),
        }
    }

    pub fn name(&self) -> String {
        match self {
            MidiInstrument::Synthesizer(synth) => synth.name(),
            MidiInstrument::Empty { name } => name.clone(),
        }
    }

    pub fn save_to(&self, path: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            MidiInstrument::Synthesizer(synth) => {
                let data = synth.extract();
                let file = File::create(path)?;
                ron::ser::to_writer_pretty(file, &data, Default::default())?;
                Ok(())
            }
            MidiInstrument::Empty { name: _ } => todo!(),
        }
    }
}
