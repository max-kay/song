use super::MidiInstrument;
use crate::{
    effects::EffectPanel,
    globals::TIME_MANAGER,
    network::{Reciever, Transform},
    time::TimeStamp,
    tracks::midi,
    wave::Wave,
    Error,
};
use serde::{Deserialize, Serialize};
use std::{any::Any, path::Path};

pub mod local_g_manager;
pub mod osc_panel;

pub use local_g_manager::LocalGManager;
pub use osc_panel::OscPanel;

const PITCH_RECIEVER: Reciever = Reciever::new(0.0, (-4800.0, 4800.0), Transform::Linear);
const VOL_CTRL_RECIEVER: Reciever = Reciever::new(1.0, (0.0, 5.0), Transform::Linear);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Synthesizer {
    name: String,
    id: Option<u8>,
    pub effects: EffectPanel,
    pub oscillators: OscPanel,
    pub local_g_manager: LocalGManager,
    pub pitch_reciever: Reciever,
    pub volume_reciever: Reciever,
}

impl Synthesizer {
    pub fn new(name: String) -> Self {
        Self {
            name,
            id: None,
            effects: EffectPanel::EmptyLeaf,
            local_g_manager: LocalGManager::new(),
            pitch_reciever: PITCH_RECIEVER,
            volume_reciever: VOL_CTRL_RECIEVER,
            oscillators: OscPanel::default(),
        }
    }
}

impl Synthesizer {
    fn play_freq(&self, note_on: TimeStamp, note_off: TimeStamp, freq: f64, velocity: f64) -> Wave {
        self.local_g_manager.set_velocity(velocity);
        let sus_samples = TIME_MANAGER
            .read()
            .unwrap()
            .duration_to_samples(note_on, note_off);

        let envelope = self.local_g_manager.get_main_envelope(note_on, sus_samples);

        let freq: Vec<f64> = self
            .pitch_reciever
            .get_vec(note_on, envelope.len())
            .into_iter()
            .map(|x| freq * 2_f64.powf(x / 1200.0))
            .collect();

        let mut wave = self.oscillators.play(freq, note_on, envelope.len());
        wave.scale_by_vec(self.volume_reciever.get_vec(note_on, envelope.len()));
        wave.scale_by_vec(envelope);
        self.effects.apply_to(&mut wave, note_on);
        wave
    }
}

impl MidiInstrument for Synthesizer {
    fn play_note(&self, note: midi::Note) -> Wave {
        self.play_freq(note.on, note.off, note.pitch.get_freq(), note.velocity)
    }
    fn play_notes(&self, notes: &[midi::Note]) -> Wave {
        let mut wave = Wave::new();
        for note in notes {
            let sound = self.play_note(*note);
            wave.add_consuming(
                sound,
                TIME_MANAGER.read().unwrap().stamp_to_samples(note.on),
            );
        }
        wave
    }
    fn put_in_song(&mut self, id: u8) -> Result<(), Error> {
        self.id = Some(id);
        self.local_g_manager.init(id)
    }
    fn name(&self) -> &str {
        &self.name
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }

    fn set_id(&mut self, id: u8) {
        self.id = Some(id);
        // self.effects.set_id(id) TODO
    }
}

impl Synthesizer {
    pub fn play_test_chord(&self) -> Wave {
        let note_on = TIME_MANAGER.read().unwrap().zero();
        let note_off = TIME_MANAGER.read().unwrap().seconds_to_stamp(6.0);
        let mut wave = self.play_freq(note_on, note_off, 300.0, 0.7);
        wave.add_consuming(self.play_freq(note_on, note_off, 375.0, 0.7), 0);
        wave.add_consuming(self.play_freq(note_on, note_off, 450.0, 0.7), 0);
        wave.add_consuming(self.play_freq(note_on, note_off, 600.0, 0.7), 0);
        wave
    }

    pub fn save_test_chord(&self) {
        let wave = self.play_test_chord();
        let path = format!("out/synthtest/{}_chord.wav", self.name);
        wave.save(Path::new(&path));
    }
}
