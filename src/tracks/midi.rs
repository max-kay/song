use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    effects::EffectPanel,
    gens::TI,
    globals::{GENRATOR_MANAGER, RESOURCE_MANAGER, TIME_MANAGER},
    instr::{
        drums::{Drums, DrumsBuilder},
        synth::SynthBuilder,
        MidiInstrument, Synthesizer,
    },
    resources::SampleId,
    time,
    wave::Wave,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
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

    pub fn get_freq(&self) -> f32 {
        440.0 * 2.0_f32.powf((self.value as f32 - 69.0) / 12.0)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Note {
    pub pitch: Pitch,
    pub on: time::ClockTick,
    pub off: time::ClockTick,
    pub velocity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidiTrack {
    name: String,
    track_id: u8,
    // after_touch_id: Option<GenId>,
    pub instrument: MidiInstrument,
    gain: f32,
    effects: EffectPanel,
    notes: Vec<Note>,
}

impl MidiTrack {
    pub fn new(track_id: u8) -> Self {
        Self {
            name: String::new(),
            track_id,
            instrument: MidiInstrument::empty(),
            gain: 1.0,
            effects: EffectPanel::EmptyLeaf,
            notes: Vec::new(),
        }
    }

    pub fn play(&self) -> Wave {
        let mut wave = self.instrument.play_notes(&self.notes);
        self.effects
            .apply_to(&mut wave, TIME_MANAGER.read().unwrap().abs_start());
        wave.scale(self.gain);
        wave
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn add_synth(&mut self, data: SynthBuilder) {
        let mut effects = data.effects;
        effects.set_id(self.track_id);

        let mut oscillators = data.oscillators;
        oscillators.set_id(self.track_id);

        let mut main_enevelope = data.main_enevelope;
        main_enevelope.set_id(self.track_id);

        let mut alt_enevelope = data.alt_enevelope;
        alt_enevelope.set_id(self.track_id);

        let mut lfo_1 = data.lfo_1;
        lfo_1.set_id(self.track_id);

        let mut lfo_2 = data.lfo_2;
        lfo_2.set_id(self.track_id);

        let mut pitch_receiver = data.pitch_receiver;
        pitch_receiver.set_id(self.track_id);

        let mut volume_receiver = data.volume_receiver;
        volume_receiver.set_id(self.track_id);

        *GENRATOR_MANAGER
            .write()
            .unwrap()
            .get_mut_instr_save(self.track_id)
            .unwrap() = data
            .instr_generator
            .as_generator_save(self.track_id, TI::Instr);

        let synth = Synthesizer {
            name: data.name,
            track_id: self.track_id,
            effects,
            oscillators,
            main_enevelope,
            alt_enevelope,
            lfo_1,
            lfo_2,
            pitch_receiver,
            volume_receiver,
        };

        self.instrument = synth.wrap_midi();
    }

    pub fn add_drums(&mut self, drums: DrumsBuilder) -> Result<(), Box<dyn std::error::Error>> {
        let mut effects = drums.effects;
        effects.set_id(self.track_id);

        let mut volume = drums.volume;
        volume.set_id(self.track_id);

        let mut samples = HashMap::<Pitch, SampleId>::new();
        for (pitch, path) in drums.samples {
            samples.insert(pitch, RESOURCE_MANAGER.write().unwrap().add_sample(path)?);
        }

        let drums = Drums {
            name: drums.name,
            effects,
            volume,
            samples,
        };
        self.instrument = MidiInstrument::Drums(Box::new(drums));
        Ok(())
    }
}

impl MidiTrack {
    pub fn set_inst_unchecked(&mut self, inst: MidiInstrument) {
        self.instrument = inst
    }
    pub fn get_inst(&self) -> &MidiInstrument {
        &self.instrument
    }
}

impl MidiTrack {
    pub(crate) fn add_notes(&mut self, mut notes: Vec<Note>) {
        self.notes.append(&mut notes)
    }

    pub(crate) fn set_name(&mut self, name: String) {
        self.name = name
    }
}
