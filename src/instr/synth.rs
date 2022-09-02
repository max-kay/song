use super::MidiInstrument;
use crate::{
    effects::EffectPanel,
    globals::TIME_MANAGER,
    network::{Reciever, Transform},
    time::TimeStamp,
    tracks::midi,
    wave::Wave,
};
use std::path::Path;

static PITCH_RECIEVER: Lazy<Reciever> =
    Lazy::new(|| Reciever::new(0.0, (-4800.0, 4800.0), Transform::Linear));
static MOD_RECIEVER: Lazy<Reciever> =
    Lazy::new(|| Reciever::new(0.5, (0.0, 1.0), Transform::Linear));
static VOL_CTRL_RECIEVER: Lazy<Reciever> =
    Lazy::new(|| Reciever::new(1.0, (0.0, 5.0), Transform::Linear));

pub mod local_g_manager;
pub mod osc_panel;

pub use local_g_manager::LocalGManager;
use once_cell::sync::Lazy;
pub use osc_panel::OscPanel;

#[derive(Debug)]
pub struct Synthesizer {
    name: String,
    effects: EffectPanel,
    oscillators: OscPanel,
    fuctions: LocalGManager,
    pitch_reciever: Reciever,
    modulation_reciever: Reciever,
    volume_control: Reciever,
}

impl Synthesizer {
    pub fn new(name: String) -> Self {
        Self {
            name,
            effects: EffectPanel::EmptyLeaf,
            fuctions: LocalGManager::new(),
            pitch_reciever: PITCH_RECIEVER.clone(),
            modulation_reciever: MOD_RECIEVER.clone(),
            volume_control: VOL_CTRL_RECIEVER.clone(),
            oscillators: OscPanel::default(),
        }
    }
}

impl Synthesizer {
    fn play_freq(&self, note_on: TimeStamp, note_off: TimeStamp, freq: f64, velocity: f64) -> Wave {
        self.fuctions.set_velocity(velocity);
        let sus_samples = TIME_MANAGER
            .lock()
            .unwrap()
            .duration_to_samples(note_on, note_off);

        let envelope = self.fuctions.get_main_envelope(note_on, sus_samples);

        let freq: Vec<f64> = self
            .pitch_reciever
            .get_vec(note_on, envelope.len())
            .into_iter()
            .map(|x| freq * 2_f64.powf(x / 1200.0))
            .collect();

        let modulation = self.modulation_reciever.get_vec(note_on, envelope.len());
        let mut wave = Wave::zeros(envelope.len());
        wave.add_consuming(
            self.oscillators
                .play(freq, modulation, note_on, envelope.len()),
            0,
        );
        wave.scale_by_vec(self.volume_control.get_vec(note_on, envelope.len()));
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
                TIME_MANAGER.lock().unwrap().stamp_to_samples(note.on),
            );
        }
        wave
    }
    fn name(&self) -> &str {
        &self.name
    }
}

impl Synthesizer {
    pub fn play_test_chord(&self) -> Wave {
        let note_on = TIME_MANAGER.lock().unwrap().zero();
        let note_off = TIME_MANAGER.lock().unwrap().seconds_to_stamp(6.0);
        let mut wave = self.play_freq(note_on, note_off, 300.0, 0.7);
        wave.add_consuming(self.play_freq(note_on, note_off, 375.0, 0.7), 0);
        wave.add_consuming(self.play_freq(note_on, note_off, 450.0, 0.7), 0);
        wave.add_consuming(self.play_freq(note_on, note_off, 600.0, 0.7), 0);
        wave
    }

    pub fn save_test_chord(&self) {
        let wave = self.play_test_chord();
        let path = format!("out/synthtest/{}_chord.wav", self.name);
        let path = Path::new(&path);
        wave.save(path);
    }
}
