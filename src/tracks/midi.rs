use serde::{
    de::{Deserialize, Deserializer},
    ser::{Serialize, SerializeStruct},
};

use crate::{
    control::{ControlError, FunctionKeeper},
    ctrl_f::{FunctionManager, FunctionOwner, IdMap, IdMapOrErr},
    effects::EffectPanel,
    instr::{EmptyInstrument, MidiInstrument},
    time::{self, TimeKeeper, TimeManager},
    utils,
    wave::Wave,
};
use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
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

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
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
    function_manager: Rc<RefCell<FunctionManager>>,
    time_manager: Rc<RefCell<TimeManager>>,
}

impl Serialize for MidiTrack {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("LocalFManager", 7)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("instrument", &*self.instrument)?;
        state.serialize_field("gain", &self.gain)?;
        state.serialize_field("effects", &self.effects)?;
        state.serialize_field("notes", &self.notes)?;
        state.serialize_field("function_manager", &self.function_manager.borrow().clone())?;
        state.serialize_field("time_manager", &self.time_manager.borrow().clone())?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for MidiTrack {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        todo!()
    }
}

impl TimeKeeper for MidiTrack {
    fn set_time_manager(&mut self, time_manager: Rc<RefCell<TimeManager>>) {
        self.instrument.set_time_manager(Rc::clone(&time_manager));
        self.effects.set_time_manager(Rc::clone(&time_manager));
        self.function_manager
            .borrow_mut()
            .set_time_manager(Rc::clone(&time_manager));
        self.time_manager = Rc::clone(&time_manager)
    }
}

impl MidiTrack {
    pub fn set_function_manager(&mut self) {
        self.instrument
            .set_fuction_manager(Rc::clone(&self.function_manager));
    }
}

impl MidiTrack {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            instrument: Box::new(EmptyInstrument::new()),
            gain: 1.0,
            effects: EffectPanel::EmptyLeaf,
            function_manager: Rc::new(RefCell::new(FunctionManager::new())),
            notes: Vec::new(),
            time_manager: Rc::new(RefCell::new(TimeManager::default())),
        }
    }
    pub fn play(&self) -> Wave {
        let mut wave = self.instrument.play_notes(&self.notes);
        self.effects
            .apply_to(&mut wave, self.time_manager.borrow().zero());
        wave.scale(self.gain);
        wave
    }

    pub fn from_instrument(instrument: Box<dyn MidiInstrument>) -> Self {
        let function_manager = Rc::new(RefCell::new(FunctionManager::new()));
        Self {
            name: String::from(instrument.name()),
            instrument,
            gain: 1.0,
            effects: EffectPanel::EmptyLeaf,
            function_manager,
            notes: Vec::new(),
            time_manager: Rc::new(RefCell::new(TimeManager::default())),
        }
    }
}

impl MidiTrack {
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
}

impl FunctionOwner for MidiTrack {
    unsafe fn new_ids(&mut self) {
        self.instrument.new_ids();
        self.function_manager.borrow_mut().new_ids();
    }

    fn get_id_map(&self) -> IdMapOrErr {
        let mut map = self.instrument.get_id_map()?;
        utils::my_extend(&mut map, self.function_manager.borrow().get_id_map()?)?;
        Ok(map)
    }
}

impl FunctionKeeper for MidiTrack {
    fn heal_sources(&mut self, id_map: &IdMap) -> Result<(), ControlError> {
        self.instrument
            .heal_sources(id_map)
            .map_err(|err| err.push_location("MidiTrack"))?;
        self.effects
            .heal_sources(id_map)
            .map_err(|err| err.push_location("MidiTrack"))?;
        self.function_manager
            .borrow_mut()
            .heal_sources(id_map)
            .map_err(|err| err.push_location("MidiTrack"))
    }

    fn test_sources(&self) -> Result<(), ControlError> {
        self.instrument
            .test_sources()
            .map_err(|err| err.push_location("MidiTrackr"))?;
        self.effects
            .test_sources()
            .map_err(|err| err.push_location("MidiTrack"))?;
        self.function_manager.borrow_mut().test_sources()
    }

    fn set_ids(&mut self) {
        self.instrument.set_ids();
        self.effects.set_ids();
        self.function_manager.borrow_mut().set_ids();
    }

    fn get_ids(&self) -> Vec<usize> {
        let mut ids = Vec::new();
        ids.append(&mut self.instrument.get_ids());
        ids.append(&mut self.effects.get_ids());
        ids.append(&mut self.function_manager.borrow().get_ids());
        ids
    }
}

impl Default for MidiTrack {
    fn default() -> Self {
        Self::new()
    }
}
