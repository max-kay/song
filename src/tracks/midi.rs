use std::rc::Rc;

use crate::auto;
use crate::effects;
use crate::instruments;
use crate::time;
use crate::wave;

#[derive(Clone, Copy)]
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

#[derive(Clone, Copy)]
pub struct Note {
    pub pitch: Pitch,
    pub on: time::TimeStamp,
    pub off: time::TimeStamp,
    pub velocity: auto::CtrlVal,
}

pub struct MidiTrack<'a, W: wave::Wave> {
    pub name: String,
    pub instrument: Box<dyn instruments::MidiInstrument<W>>,
    pub gain: f64,
    pub effects: effects::EffectNode<W>,
    pub control_panel: effects::CtrlPanel<'a>,
    pub notes: Vec<Note>,
    pub automation: Rc<auto::AutomationManager>,
    pub time_manager: Rc<time::TimeManager>,
}

impl<W: wave::Wave> time::TimeKeeper for MidiTrack<'_, W> {
    fn set_time_manager(&mut self, time_manager: &Rc<time::TimeManager>) {
        self.instrument.set_time_manager(time_manager);
    }
}

impl<'a, W: 'static + wave::Wave> MidiTrack<'a, W> {
    pub fn new(time_manager: Rc<time::TimeManager>) -> Self {
        Self {
            name: String::new(),
            instrument: Box::new(instruments::EmptyInstrument::<W>::new()),
            gain: 1.0,
            effects: effects::EffectNode::Bypass,
            control_panel: effects::CtrlPanel::Bypass,
            automation: Rc::new(auto::AutomationManager::new()),
            notes: Vec::new(),
            time_manager,
        }
    }
    pub fn play(&self) -> W {
        let mut wave = self.instrument.play_notes(&self.notes);
        self.effects
            .apply(&mut wave, &self.control_panel, self.time_manager.zero());
        wave.scale(self.gain);
        wave
    }

    pub fn from_instrument(
        mut instrument: Box<dyn instruments::MidiInstrument<W>>,
        time_manager: Rc<time::TimeManager>,
    ) -> Self {
        let automation = Rc::new(auto::AutomationManager::new());
        instrument.set_track_automation(&automation);
        Self {
            name: String::from(instrument.name()),
            instrument,
            gain: 1.0,
            effects: effects::EffectNode::<W>::Bypass,
            control_panel: effects::CtrlPanel::Bypass,
            automation,
            notes: Vec::new(),
            time_manager,
        }
    }
}
