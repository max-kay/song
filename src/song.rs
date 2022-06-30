use crate::auto::AutomationManager;
use crate::effects::Effect;
use crate::midi;
use crate::time::{Duration, TimeStamp};
use crate::utils::add_from_index;
use std::rc::Rc;

pub trait Instrument {
    fn play_freq(
        &self,
        onset: TimeStamp,
        note_held: Duration,
        freq: f64,
        velocity: midi::Velocity,
    ) -> Vec<f64>;
    fn play_midi_note(&self, note: midi::Note) -> Vec<f64>;
}

pub struct Empty {}
impl Instrument for Empty {
    fn play_freq(
        &self,
        onset: TimeStamp,
        note_held: Duration,
        freq: f64,
        velocity: midi::Velocity,
    ) -> Vec<f64> {
        Vec::new()
    }
    fn play_midi_note(&self, note: midi::Note) -> Vec<f64> {
        Vec::new()
    }
}

pub struct Track {
    instrument: Box<dyn Instrument>,
    gain: f64,
    effects: Vec<Box<dyn Effect>>,
    notes: Vec<midi::Note>,
    global_automation: Rc<AutomationManager>,
}

impl Track {
    pub fn play(&self) -> Vec<f64> {
        let mut out = Vec::<f64>::new();
        for note in &self.notes {
            let sound = self.instrument.play_midi_note(*note);
            add_from_index(&mut out, sound, note.onset.to_samples());
        }
        for effect in &self.effects {
            todo!()
        }
        out.into_iter().map(|x| x * self.gain).collect()
    }

    pub fn from_instrument(
        instrument: Box<dyn Instrument>,
        global_automation: Rc<AutomationManager>,
    ) -> Self {
        Self {
            instrument,
            gain: 1.0,
            effects: Vec::new(),
            notes: Vec::new(),
            global_automation,
        }
    }
}

pub struct Song {
    name: String,
    global_automation: Rc<AutomationManager>,
    tracks: Vec<Track>,
}

impl Song {
    pub fn new(name: String, global_automation: Rc<AutomationManager>, tracks: Vec<Track>) -> Self {
        Self {
            name,
            global_automation,
            tracks,
        }
    }

    pub fn empty(name: String) -> Self {
        Self {
            name,
            global_automation: Rc::new(AutomationManager::new()),
            tracks: Vec::new(),
        }
    }

    pub fn add_track(&mut self, track: Track) {
        self.tracks.push(track)
    }

    pub fn add_instrument<I: Instrument>(&mut self, instrument: Box<dyn Instrument>) {
        self.tracks.push(Track::from_instrument(
            instrument,
            Rc::clone(&self.global_automation),
        ))
    }
}
