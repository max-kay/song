use crate::{
    effects::EffectPanel,
    gens::{Envelope, GenId, GenSaveBuilder, Lfo, Specific},
    globals::{GENRATOR_MANAGER, TIME_MANAGER},
    network::{Receiver, Transform},
    time::ClockTick,
    tracks::midi,
    wave::Wave,
};
use serde::{Deserialize, Serialize};
use std::{fs::File, path::Path};

pub mod osc_panel;

pub use osc_panel::OscPanel;

use super::MidiInstrument;

const PITCH_RECEIVER: Receiver = Receiver::new(0.0, (-4800.0, 4800.0), Transform::Linear);
const VOL_CTRL_RECEIVER: Receiver = Receiver::new(1.0, (0.0, 5.0), Transform::Linear);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Synthesizer {
    pub(crate) name: String,
    pub(crate) track_id: u8,
    pub(crate) effects: EffectPanel,
    pub(crate) oscillators: OscPanel,
    pub(crate) main_enevelope: GenId,
    pub(crate) alt_enevelope: GenId,
    pub(crate) lfo_1: GenId,
    pub(crate) lfo_2: GenId,
    pub(crate) pitch_receiver: Receiver,
    pub(crate) volume_receiver: Receiver,
}

impl Synthesizer {
    fn play_freq(&self, note_on: ClockTick, note_off: ClockTick, freq: f32, velocity: f32) -> Wave {
        GENRATOR_MANAGER
            .write()
            .unwrap()
            .set_const(
                GenId::Specific {
                    track_id: self.track_id,
                    kind: Specific::Vel,
                },
                velocity,
            )
            .expect("invalid velocity id in synthersizer");

        let sus_samples = TIME_MANAGER
            .read()
            .unwrap()
            .duration_to_samples(note_on, note_off);

        let envelope = GENRATOR_MANAGER
            .read()
            .unwrap()
            .get_envelope(self.main_enevelope, note_on, sus_samples)
            .expect("non envelope envelope call");
        // TODO
        let cent_offsets = self.pitch_receiver.get_vec(note_on, envelope.len());

        let mut wave = self.oscillators.play(freq, &cent_offsets, note_on, envelope.len());
        wave.scale_by_vec(self.volume_receiver.get_vec(note_on, envelope.len()));
        wave.scale_by_vec(envelope);
        self.effects.apply_to(&mut wave, note_on);
        wave
    }

    pub fn play_note(&self, note: midi::Note) -> Wave {
        self.play_freq(note.on, note.off, note.pitch.get_freq(), note.velocity)
    }

    pub fn play_notes(&self, notes: &[midi::Note]) -> Wave {
        let mut wave = Wave::new();
        // TODO think about how to handle if notes only start at some timestamp
        for note in notes {
            let sound = self.play_note(*note);
            wave.add(&sound, TIME_MANAGER.read().unwrap().tick_to_sample(note.on));
        }
        wave
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }
}

impl Synthesizer {
    pub fn play_test_chord(&self) -> Wave {
        let note_on = TIME_MANAGER.read().unwrap().abs_start();
        let note_off = TIME_MANAGER.read().unwrap().second_to_tick(6.0);
        let mut wave = self.play_freq(note_on, note_off, 300.0, 0.7);
        wave.add(&self.play_freq(note_on, note_off, 375.0, 0.7), 0);
        wave.add(&self.play_freq(note_on, note_off, 450.0, 0.7), 0);
        wave.add(&self.play_freq(note_on, note_off, 600.0, 0.7), 0);
        wave
    }

    pub fn save_test_chord(&self) {
        let wave = self.play_test_chord();
        let path = format!("out/synthtest/{}_chord.wav", self.name);
        wave.save(Path::new(&path));
    }

    pub fn extract(&self) -> SynthBuilder {
        SynthBuilder {
            name: self.name.clone(),
            effects: self.effects.extract(),
            oscillators: self.oscillators.extract(),
            main_enevelope: self
                .main_enevelope
                .extract()
                .expect("synthesizer had invalid GenId"),
            alt_enevelope: self
                .alt_enevelope
                .extract()
                .expect("synthesizer had invalid GenId"),
            lfo_1: self.lfo_1.extract().expect("synthesizer had invalid GenId"),
            lfo_2: self.lfo_2.extract().expect("synthesizer had invalid GenId"),
            pitch_receiver: self.pitch_receiver.extract(),
            volume_receiver: self.volume_receiver.extract(),
            instr_generator: GENRATOR_MANAGER
                .read()
                .unwrap()
                .get_instr_save(self.track_id)
                .expect("synthesizer had invalid generator save")
                .into(),
        }
    }

    pub fn wrap_midi(self) -> MidiInstrument {
        MidiInstrument::Synthesizer(Box::new(self))
    }
}

impl Synthesizer {
    pub fn save_to(&self, path: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
        let data: SynthBuilder = self.extract();
        let file = File::create(path)?;
        ron::ser::to_writer_pretty(file, &data, Default::default())?;
        // serde_json::to_writer_pretty(file, &data)?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SynthBuilder {
    pub name: String,
    pub effects: EffectPanel,
    pub oscillators: OscPanel,
    pub main_enevelope: GenId,
    pub alt_enevelope: GenId,
    pub lfo_1: GenId,
    pub lfo_2: GenId,
    pub pitch_receiver: Receiver,
    pub volume_receiver: Receiver,
    pub instr_generator: GenSaveBuilder,
}

impl SynthBuilder {
    pub fn new(name: &str) -> Self {
        let mut instr_generator = GenSaveBuilder::new();
        let main_enevelope = GenId::InstrExtracted {
            key: instr_generator.insert_gen(Envelope::w_default()).unwrap(),
        };
        let alt_enevelope = GenId::InstrExtracted {
            key: instr_generator.insert_gen(Envelope::w_default()).unwrap(),
        };
        let lfo_1 = GenId::InstrExtracted {
            key: instr_generator.insert_gen(Lfo::w_default()).unwrap(),
        };
        let lfo_2 = GenId::InstrExtracted {
            key: instr_generator.insert_gen(Lfo::w_default()).unwrap(),
        };
        Self {
            name: name.to_string(),
            effects: EffectPanel::EmptyLeaf,
            oscillators: OscPanel::default(),
            main_enevelope,
            alt_enevelope,
            lfo_1,
            lfo_2,
            pitch_receiver: PITCH_RECEIVER,
            volume_receiver: VOL_CTRL_RECEIVER,
            instr_generator,
        }
    }
}

impl SynthBuilder {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let rdr = File::open(path)?;
        Ok(ron::de::from_reader(rdr)?)
    }
}
