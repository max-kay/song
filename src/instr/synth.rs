use super::MidiInstrument;
use crate::{
    ctrl_f::{Control, ControlError, Source},
    ctrl_f::{self, Envelope, Lfo},
    effects::EffectPanel,
    globals::TIME_MANAGER,
    time::TimeStamp,
    tracks::midi,
    utils::oscs::Oscillator,
    wave::Wave,
};
use std::{cell::RefCell, path::Path, rc::Rc, result::Result};

const PITCH_WHEEL_RANGE: (f64, f64) = (-4800.0, 4800.0);
const VOL_CTRL_RANGE: (f64, f64) = (0.0, 5.0);

pub mod local_f_manager;
pub mod osc_panel;

pub use local_f_manager::LocalFManager;
pub use osc_panel::OscPanel;

#[derive(Debug)]
pub struct Synthesizer<W: Wave> {
    name: String,
    effects: EffectPanel<W>,
    oscillators: OscPanel<W>,
    fuctions: LocalFManager,
    pitch_control: Control,
    modulation_control: Control,
    volume_control: Control,
}

impl<W: Wave> Synthesizer<W> {
    pub fn new(name: String) -> Self {
        Self {
            name,
            effects: EffectPanel::EmptyLeaf,
            fuctions: LocalFManager::new(),
            pitch_control: Control::from_val_in_unit(0.5).unwrap(),
            modulation_control: Control::from_val_in_unit(0.5).unwrap(),
            volume_control: Control::from_val_in_unit(1.0).unwrap(),
            oscillators: OscPanel::default(),
        }
    }
}

impl<W: Wave> Synthesizer<W> {
    fn play_freq(&self, note_on: TimeStamp, note_off: TimeStamp, freq: f64, velocity: f64) -> W {
        self.fuctions.set_velocity(velocity);
        let sus_samples = TIME_MANAGER
            .lock()
            .unwrap()
            .duration_to_samples(note_on, note_off);

        let envelope = self.fuctions.get_main_envelope(note_on, sus_samples);

        let freq: Vec<f64> = self
            .pitch_control
            .get_vec(note_on, envelope.len())
            .into_iter()
            .map(|x| freq * 2_f64.powf(x / 1200.0))
            .collect();

        let modulation = self.modulation_control.get_vec(note_on, envelope.len());
        let mut wave = W::zeros(envelope.len());
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

impl<W: Wave> MidiInstrument<W> for Synthesizer<W> {
    fn play_note(&self, note: midi::Note) -> W {
        self.play_freq(note.on, note.off, note.pitch.get_freq(), note.velocity)
    }
    fn play_notes(&self, notes: &[midi::Note]) -> W {
        let mut wave = W::new();
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

impl<W: Wave> Synthesizer<W> {
    pub fn get_main_envelope(&self) -> Source {
        todo!()
    }

    pub fn get_alt_envelope(&self) -> Source {
        todo!()
    }

    pub fn get_current_velocity(&self) -> Source {
        todo!()
    }

    pub fn get_lfo1(&self) -> Source {
        todo!()
    }

    pub fn get_lfo2(&self) -> Source {
        todo!()
    }

    pub fn get_automation_channel(&self, channel: u8) -> Option<Source> {
        todo!()
    }
}

impl<W: Wave> Synthesizer<W> {
    pub fn add_osc(&mut self, oscillator: Oscillator) {
        self.oscillators.add_osc(oscillator)
    }

    pub fn set_main_envelope(&mut self, envelope: Envelope) -> Result<(), ControlError> {
        todo!()
    }

    pub fn set_alt_envelope(&mut self, envelope: Envelope) -> Result<(), ControlError> {
        todo!()
    }

    pub fn set_lfo1(&mut self, lfo: Lfo) -> Result<(), ControlError> {
        todo!()
    }

    pub fn set_lfo2(&mut self, lfo: Lfo) -> Result<(), ControlError> {
        todo!()
    }

    pub fn set_vol_source(&mut self, source: Source) {
        self.volume_control = Control::new(1.0, VOL_CTRL_RANGE, source).unwrap()
    }

    pub fn set_pitch_source(&mut self, source: Source) {
        self.volume_control = Control::new(0.0, PITCH_WHEEL_RANGE, source).unwrap()
    }
}

impl<W: Wave> Synthesizer<W> {
    pub fn play_test_chord(&self) -> W {
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
