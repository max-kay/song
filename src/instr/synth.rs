use super::MidiInstrument;
use crate::{
    control::{Control, ControlError, FunctionKeeper, Source},
    ctrl_f::{
        self, Envelope, FunctionManager, FunctionMngrKeeper, FunctionOwner, IdMap, IdMapOrErr, Lfo,
    },
    effects::EffectPanel,
    time::{TimeKeeper, TimeManager, TimeStamp},
    tracks::midi,
    utils::oscs::Oscillator,
    wave::Wave,
};
use serde::{
    de::{Deserialize, Deserializer},
    ser::{Serialize, SerializeStruct},
};
use std::{cell::RefCell, path::Path, rc::Rc, result::Result};

const PITCH_WHEEL_RANGE: (f64, f64) = (-4800.0, 4800.0);
const VOL_CTRL_RANGE: (f64, f64) = (0.0, 5.0);

pub mod local_f_manager;
pub mod osc_panel;

pub use local_f_manager::LocalFManager;
pub use osc_panel::OscPanel;

#[derive(Debug, Clone)]
pub struct Synthesizer<'fm, 'tm> {
    name: String,
    effects: EffectPanel,
    oscillators: OscPanel,
    functions: &'fm LocalFManager<'tm>,
    pitch_control: Control,
    modulation_control: Control,
    volume_control: Control,
    time_manager: &'tm TimeManager,
}

// impl<'de> Deserialize<'de> for Synthesizer {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         todo!()
//     }
// }

// impl Serialize for Synthesizer {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::Serializer,
//     {
//         let mut state = serializer.serialize_struct("Synthesizer", 8)?;
//         state.serialize_field("name", &self.name)?;
//         state.serialize_field("effects", &self.effects)?;
//         state.serialize_field("oscillators", &self.oscillators)?;
//         state.serialize_field("fuctions", &self.functions.borrow().clone())?;
//         state.serialize_field("pitch_control", &self.pitch_control)?;
//         state.serialize_field("modulation_control", &self.modulation_control)?;
//         state.serialize_field("volume_control", &self.volume_control)?;
//         state.skip_field("time_manager")?;
//         state.end()
//     }
// }

impl Synthesizer {
    pub fn new(name: String) -> Self {
        Self {
            name,
            effects: EffectPanel::EmptyLeaf,
            functions: Rc::new(RefCell::new(LocalFManager::new())),
            pitch_control: Control::from_val_in_unit(0.5).unwrap(),
            modulation_control: Control::from_val_in_unit(0.5).unwrap(),
            volume_control: Control::from_val_in_unit(1.0).unwrap(),
            oscillators: OscPanel::default(),
            time_manager: Rc::new(RefCell::new(TimeManager::default())),
        }
    }
}

impl TimeKeeper for Synthesizer {
    fn set_time_manager(&mut self, time_manager: Rc<RefCell<TimeManager>>) {
        self.effects.set_time_manager(Rc::clone(&time_manager));
        self.functions
            .borrow_mut()
            .set_time_manager(Rc::clone(&time_manager));
        self.pitch_control
            .set_time_manager(Rc::clone(&time_manager));
        self.modulation_control
            .set_time_manager(Rc::clone(&time_manager))
    }
}

impl Synthesizer {
    fn play_freq(&self, note_on: TimeStamp, note_off: TimeStamp, freq: f64, velocity: f64) -> Wave {
        self.functions.borrow_mut().set_velocity(velocity);
        let sus_samples = self
            .time_manager
            .borrow()
            .duration_to_samples(note_on, note_off);

        let envelope = self
            .functions
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

impl FunctionKeeper for Synthesizer {
    fn heal_sources(&mut self, id_map: &IdMap) -> Result<(), ControlError> {
        self.effects
            .heal_sources(id_map)
            .map_err(|err| err.push_location("Synthesizer"))?;
        self.oscillators
            .heal_sources(id_map)
            .map_err(|err| err.push_location("Synthesizer"))?;
        self.functions
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
        self.functions.borrow_mut().set_ids();
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
        self.functions
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
        ids.append(&mut self.functions.borrow().get_ids());
        ids.append(&mut self.pitch_control.get_ids());
        ids.append(&mut self.modulation_control.get_ids());
        ids.append(&mut self.volume_control.get_ids());
        ids
    }
}

impl FunctionOwner for Synthesizer {
    unsafe fn new_ids(&mut self) {
        self.functions.borrow_mut().new_ids()
    }

    fn get_id_map(&self) -> IdMapOrErr {
        self.functions.borrow().get_id_map()
    }
}

impl FunctionMngrKeeper for Synthesizer {
    fn set_fuction_manager(&mut self, function_manager: Rc<RefCell<FunctionManager>>) {
        self.functions
            .borrow_mut()
            .set_fuction_manager(Rc::clone(&function_manager))
    }
}

#[typetag::serde]
impl MidiInstrument for Synthesizer {
    fn play_note(&self, note: midi::Note) -> Wave {
        self.play_freq(note.on, note.off, note.pitch.get_freq(), note.velocity)
    }
    fn play_notes(&self, notes: &[midi::Note]) -> Wave {
        let mut wave = Wave::new();
        for note in notes {
            let sound = self.play_note(*note);
            wave.add_consuming(sound, self.time_manager.borrow().stamp_to_samples(note.on));
        }
        wave
    }
    fn name(&self) -> &str {
        &self.name
    }
}

impl Synthesizer {
    pub fn get_main_envelope(&self) -> Source {
        Source::from_function(ctrl_f::make_ctrl_function(Rc::clone(
            &self.functions.borrow().main_envelope,
        )))
    }

    pub fn get_alt_envelope(&self) -> Source {
        Source::from_function(ctrl_f::make_ctrl_function(Rc::clone(
            &self.functions.borrow().alt_envelope,
        )))
    }

    pub fn get_current_velocity(&self) -> Source {
        Source::from_function(ctrl_f::make_ctrl_function(Rc::clone(
            &self.functions.borrow().current_velocity,
        )))
    }

    pub fn get_lfo1(&self) -> Source {
        Source::from_function(ctrl_f::make_ctrl_function(Rc::clone(
            &self.functions.borrow().lfo1,
        )))
    }

    pub fn get_lfo2(&self) -> Source {
        Source::from_function(ctrl_f::make_ctrl_function(Rc::clone(
            &self.functions.borrow().lfo2,
        )))
    }

    pub fn get_automation_channel(&self, channel: u8) -> Option<Source> {
        self.functions
            .borrow()
            .track_functions
            .borrow()
            .get_channel(channel)
            .map(Source::from_function)
    }
}

impl Synthesizer {
    pub fn add_osc(&mut self, oscillator: Oscillator) {
        self.oscillators.add_osc(oscillator)
    }

    pub fn set_main_envelope(&mut self, envelope: Envelope) -> Result<(), ControlError> {
        self.functions
            .borrow_mut()
            .main_envelope
            .borrow_mut()
            .set(envelope)
            .map_err(|err| err.push_location("Synthesizer"))?;
        Ok(())
    }

    pub fn set_alt_envelope(&mut self, envelope: Envelope) -> Result<(), ControlError> {
        self.functions
            .borrow_mut()
            .alt_envelope
            .borrow_mut()
            .set(envelope)
            .map_err(|err| err.push_location("Synthesizer"))?;
        Ok(())
    }

    pub fn set_lfo1(&mut self, lfo: Lfo) -> Result<(), ControlError> {
        self.functions
            .borrow_mut()
            .lfo1
            .borrow_mut()
            .set(lfo)
            .map_err(|err| err.push_location("Synthesizer"))?;
        Ok(())
    }

    pub fn set_lfo2(&mut self, lfo: Lfo) -> Result<(), ControlError> {
        self.functions
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

    pub fn set_effects(&mut self, effects: EffectPanel){
        self.effects = effects
    }
}

impl Synthesizer {
    pub fn play_test_chord(&self) -> Wave {
        let note_on = self.time_manager.borrow().zero();
        let note_off = self.time_manager.borrow().seconds_to_stamp(6.0);
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
