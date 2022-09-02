use crate::{
    effects::EffectPanel,
    globals::TIME_MANAGER,
    instr::{EmptyInstrument, MidiInstrument},
    time::{self},
    wave::Wave,
};

#[derive(Debug, Clone, Copy)]
pub struct Pitch {
    value: u8,
}
impl Pitch {
    pub fn new(value: u8) -> Option<Self> {
        if value < 0x80 {
            Some(Self { value })
        } else {
            None
        }
    }

    pub fn get(&self) -> u8 {
        self.value
    }

    pub fn new_unchecked(value: u8) -> Self {
        Self { value }
    }

    pub fn get_freq(&self) -> f64 {
        440.0 * 2.0_f64.powf((self.value as f64 - 69.0) / 12.0)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Note {
    pub pitch: Pitch,
    pub on: time::TimeStamp,
    pub off: time::TimeStamp,
    pub velocity: f64,
}

#[derive(Debug)]
pub struct MidiTrack {
    name: String,
    instrument: Box<dyn MidiInstrument>,
    gain: f64,
    effects: EffectPanel,
    notes: Vec<Note>,
}

impl MidiTrack {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            instrument: Box::new(EmptyInstrument::new()),
            gain: 1.0,
            effects: EffectPanel::EmptyLeaf,
            notes: Vec::new(),
        }
    }

    pub fn play(&self) -> Wave {
        let mut wave = self.instrument.play_notes(&self.notes);
        self.effects
            .apply_to(&mut wave, TIME_MANAGER.lock().unwrap().zero());
        wave.scale(self.gain);
        wave
    }

    pub fn from_instrument(instrument: Box<dyn MidiInstrument>) -> Self {
        Self {
            name: String::from(instrument.name()),
            instrument,
            gain: 1.0,
            effects: EffectPanel::EmptyLeaf,
            notes: Vec::new(),
        }
    }
}

impl MidiTrack {
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
}

impl Default for MidiTrack {
    fn default() -> Self {
        Self::new()
    }
}
