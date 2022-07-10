use std::rc::Rc;

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
    pub velocity: f64,
}

pub struct MidiTrack<W: wave::Wave> {
    pub name: String,
    pub instrument: Box<dyn instruments::MidiInstrument<W>>,
    pub gain: f64,
    pub effects: effects::EffectNode<W>,
    pub notes: Vec<Note>,
    pub time_keeper: Rc<time::TimeKeeper>,
}

impl<W: 'static + wave::Wave> MidiTrack<W> {
    pub fn new(time_keeper: Rc<time::TimeKeeper>) -> Self {
        Self {
            name: String::new(),
            instrument: Box::new(instruments::EmptyInstrument::<W>::new()),
            gain: 1.0,
            effects: effects::EffectNode::Bypass,
            notes: Vec::new(),
            time_keeper,
        }
    }
    pub fn play(&self) -> W {
        let mut wave = self.instrument.play_notes(&self.notes);
        self.effects.apply(&mut wave, self.time_keeper.zero());
        wave.scale(self.gain);
        wave
    }

    pub fn from_instrument(
        instrument: Box<dyn instruments::MidiInstrument<W>>,
        time_keeper: Rc<time::TimeKeeper>,
    ) -> Self {
        Self {
            name: String::from(instrument.name()),
            instrument,
            gain: 1.0,
            effects: effects::EffectNode::<W>::Bypass,
            notes: Vec::new(),
            time_keeper,
        }
    }
}
