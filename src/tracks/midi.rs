use crate::{
    control::{ControlError, SourceKeeper},
    ctrl_f::{FunctionManager, FunctionOwner, IdMap, IdMapOrErr},
    effects::EffectPanel,
    instr::{EmptyInstrument, MidiInstrument},
    time::{self, TimeKeeper, TimeManager},
    utils,
    wave::Wave,
};
use std::{cell::RefCell, rc::Rc};

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
pub struct MidiTrack<W: Wave> {
    name: String,
    instrument: Box<dyn MidiInstrument<W>>,
    gain: f64,
    effects: EffectPanel<W>,
    notes: Vec<Note>,
    function_manager: Rc<RefCell<FunctionManager>>,
    time_manager: Rc<RefCell<TimeManager>>,
}

impl<W: Wave> TimeKeeper for MidiTrack<W> {
    fn set_time_manager(&mut self, time_manager: Rc<RefCell<TimeManager>>) {
        self.instrument.set_time_manager(Rc::clone(&time_manager));
        self.effects.set_time_manager(Rc::clone(&time_manager));
        self.function_manager
            .borrow_mut()
            .set_time_manager(Rc::clone(&time_manager));
        self.time_manager = Rc::clone(&time_manager)
    }
}

impl<W: Wave> MidiTrack<W> {
    pub fn set_function_manager(&mut self) {
        self.instrument
            .set_fuction_manager(Rc::clone(&self.function_manager));
    }
}

impl<W: 'static + Wave> MidiTrack<W> {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            instrument: Box::new(EmptyInstrument::<W>::new()),
            gain: 1.0,
            effects: EffectPanel::EmptyLeaf,
            function_manager: Rc::new(RefCell::new(FunctionManager::new())),
            notes: Vec::new(),
            time_manager: Rc::new(RefCell::new(TimeManager::default())),
        }
    }
    pub fn play(&self) -> W {
        let mut wave = self.instrument.play_notes(&self.notes);
        self.effects
            .apply_to(&mut wave, self.time_manager.borrow().zero());
        wave.scale(self.gain);
        wave
    }

    pub fn from_instrument(instrument: Box<dyn MidiInstrument<W>>) -> Self {
        let function_manager = Rc::new(RefCell::new(FunctionManager::new()));
        Self {
            name: String::from(instrument.name()),
            instrument,
            gain: 1.0,
            effects: EffectPanel::<W>::EmptyLeaf,
            function_manager,
            notes: Vec::new(),
            time_manager: Rc::new(RefCell::new(TimeManager::default())),
        }
    }
}

impl<W: Wave> MidiTrack<W> {
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
}

impl<W: Wave> FunctionOwner for MidiTrack<W> {
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

impl<W: Wave> SourceKeeper for MidiTrack<W> {
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

impl<W: 'static + Wave> Default for MidiTrack<W> {
    fn default() -> Self {
        Self::new()
    }
}
