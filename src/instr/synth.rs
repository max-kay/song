use super::MidiInstrument;
use crate::{
    control::{Control, ControlError, FunctionKeeper, Source},
    ctrl_f::{
        self, Envelope, FunctionManager, FunctionMngrKeeper, FunctionOwner, IdMap, IdMapOrErr, Lfo,
    },
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
    fuctions: Rc<RefCell<LocalFManager>>,
    pitch_control: Control,
    modulation_control: Control,
    volume_control: Control,
}

impl<W: Wave> Synthesizer<W> {
    pub fn new(name: String) -> Self {
        Self {
            name,
            effects: EffectPanel::EmptyLeaf,
            fuctions: Rc::new(RefCell::new(LocalFManager::new())),
            pitch_control: Control::from_val_in_unit(0.5).unwrap(),
            modulation_control: Control::from_val_in_unit(0.5).unwrap(),
            volume_control: Control::from_val_in_unit(1.0).unwrap(),
            oscillators: OscPanel::default(),
        }
    }
}

impl<W: Wave> Synthesizer<W> {
    fn play_freq(&self, note_on: TimeStamp, note_off: TimeStamp, freq: f64, velocity: f64) -> W {
        self.fuctions.borrow_mut().set_velocity(velocity);
        let sus_samples = TIME_MANAGER
            .lock()
            .unwrap()
            .duration_to_samples(note_on, note_off);

        let envelope = self
            .fuctions
            .borrow()
            .main_envelope
            .borrow()
            .get_envelope(sus_samples, note_on);

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

impl<W: Wave> FunctionKeeper for Synthesizer<W> {
    fn heal_sources(&mut self, id_map: &IdMap) -> Result<(), ControlError> {
        self.effects
            .heal_sources(id_map)
            .map_err(|err| err.push_location("Synthesizer"))?;
        self.oscillators
            .heal_sources(id_map)
            .map_err(|err| err.push_location("Synthesizer"))?;
        self.fuctions
            .borrow_mut()
            .heal_sources(id_map)
            .map_err(|err| err.push_location("Synthesizer"))?;
        self.pitch_control
            .heal_sources(id_map)
            .map_err(|err| err.push_location("Synthesizer"))?;
        self.modulation_control
            .heal_sources(id_map)
            .map_err(|err| err.push_location("Synthesizer"))?;
        self.volume_control
            .heal_sources(id_map)
            .map_err(|err| err.push_location("Synthesizer"))
    }

    fn set_ids(&mut self) {
        self.effects.set_ids();
        self.oscillators.set_ids();
        self.fuctions.borrow_mut().set_ids();
        self.pitch_control.set_ids();
        self.modulation_control.set_ids();
        self.volume_control.set_ids();
    }

    fn test_sources(&self) -> Result<(), ControlError> {
        self.effects
            .test_sources()
            .map_err(|err| err.push_location("Synthesizer"))?;
        self.oscillators
            .test_sources()
            .map_err(|err| err.push_location("Synthesizer"))?;
        self.fuctions
            .borrow()
            .test_sources()
            .map_err(|err| err.push_location("Synthesizer"))?;
        self.pitch_control
            .test_sources()
            .map_err(|err| err.push_location("Synthesizer"))?;
        self.modulation_control
            .test_sources()
            .map_err(|err| err.push_location("Synthesizer"))?;
        self.volume_control
            .test_sources()
            .map_err(|err| err.push_location("Synthesizer"))
    }

    fn get_ids(&self) -> Vec<usize> {
        let mut ids = Vec::new();
        ids.append(&mut self.effects.get_ids());
        ids.append(&mut self.oscillators.get_ids());
        ids.append(&mut self.fuctions.borrow().get_ids());
        ids.append(&mut self.pitch_control.get_ids());
        ids.append(&mut self.modulation_control.get_ids());
        ids.append(&mut self.volume_control.get_ids());
        ids
    }
}

impl<W: Wave> FunctionOwner for Synthesizer<W> {
    unsafe fn new_ids(&mut self) {
        self.fuctions.borrow_mut().new_ids()
    }

    fn get_id_map(&self) -> IdMapOrErr {
        self.fuctions.borrow().get_id_map()
    }
}

impl<W: Wave> FunctionMngrKeeper for Synthesizer<W> {
    fn set_fuction_manager(&mut self, function_manager: Rc<RefCell<FunctionManager>>) {
        self.fuctions
            .borrow_mut()
            .set_fuction_manager(Rc::clone(&function_manager))
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
        Source::from_function(ctrl_f::make_ctrl_function(Rc::clone(
            &self.fuctions.borrow().main_envelope,
        )))
    }

    pub fn get_alt_envelope(&self) -> Source {
        Source::from_function(ctrl_f::make_ctrl_function(Rc::clone(
            &self.fuctions.borrow().alt_envelope,
        )))
    }

    pub fn get_current_velocity(&self) -> Source {
        Source::from_function(ctrl_f::make_ctrl_function(Rc::clone(
            &self.fuctions.borrow().current_velocity,
        )))
    }

    pub fn get_lfo1(&self) -> Source {
        Source::from_function(ctrl_f::make_ctrl_function(Rc::clone(
            &self.fuctions.borrow().lfo1,
        )))
    }

    pub fn get_lfo2(&self) -> Source {
        Source::from_function(ctrl_f::make_ctrl_function(Rc::clone(
            &self.fuctions.borrow().lfo2,
        )))
    }

    pub fn get_automation_channel(&self, channel: u8) -> Option<Source> {
        self.fuctions
            .borrow()
            .track_functions
            .borrow()
            .get_channel(channel)
            .map(Source::from_function)
    }
}

impl<W: Wave> Synthesizer<W> {
    pub fn add_osc(&mut self, oscillator: Oscillator) {
        self.oscillators.add_osc(oscillator)
    }

    pub fn set_main_envelope(&mut self, envelope: Envelope) -> Result<(), ControlError> {
        self.fuctions
            .borrow_mut()
            .main_envelope
            .borrow_mut()
            .set(envelope)
            .map_err(|err| err.push_location("Synthesizer"))?;
        Ok(())
    }

    pub fn set_alt_envelope(&mut self, envelope: Envelope) -> Result<(), ControlError> {
        self.fuctions
            .borrow_mut()
            .alt_envelope
            .borrow_mut()
            .set(envelope)
            .map_err(|err| err.push_location("Synthesizer"))?;
        Ok(())
    }

    pub fn set_lfo1(&mut self, lfo: Lfo) -> Result<(), ControlError> {
        self.fuctions
            .borrow_mut()
            .lfo1
            .borrow_mut()
            .set(lfo)
            .map_err(|err| err.push_location("Synthesizer"))?;
        Ok(())
    }

    pub fn set_lfo2(&mut self, lfo: Lfo) -> Result<(), ControlError> {
        self.fuctions
            .borrow_mut()
            .lfo2
            .borrow_mut()
            .set(lfo)
            .map_err(|err| err.push_location("Synthesizer"))?;
        Ok(())
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
