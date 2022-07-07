use std::marker::PhantomData;
use std::rc::Rc;

use crate::effects::EffectNode;
use crate::midi;
use crate::time::{Duration, TimeKeeper, TimeStamp};
use crate::wave::Wave;

pub trait Instrument<T: Wave> {
    fn play_freq(
        &self,
        onset: TimeStamp,
        note_held: Duration,
        freq: f64,
        velocity: midi::Velocity,
    ) -> T;
    fn play_midi_note(&self, note: midi::Note) -> T;
    fn name(&self) -> &str;
}

pub struct Empty<W: Wave> {
    phantom: PhantomData<W>,
}
impl<W: Wave> Empty<W>{
    pub fn new()-> Self{
        Empty { phantom: PhantomData }
    }
}

impl<W: Wave> Instrument<W> for Empty<W> {
    fn play_freq(
        &self,
        _onset: TimeStamp,
        _note_held: Duration,
        _freq: f64,
        _velocity: midi::Velocity,
    ) -> W {
        W::new()
    }
    fn play_midi_note(&self, _note: midi::Note) -> W {
        W::new()
    }
    fn name(&self) -> &str {
        "empty"
    }
}

pub struct Track<W: Wave> {
    pub name: String,
    pub instrument: Box<dyn Instrument<W>>,
    pub gain: f64,
    pub effects: EffectNode<W>,
    pub notes: Vec<midi::Note>,
    pub time_keeper: Rc<TimeKeeper>,
}

impl<W: 'static + Wave> Track<W> {
    pub fn new(time_keeper: Rc<TimeKeeper>)-> Self{
        Self { name: String::new(), instrument: Box::new(Empty::<W>::new()), gain: 1.0, effects: EffectNode::End, notes: Vec::new(), time_keeper }
    }
    pub fn play(&self) -> W {
        let mut wave = W::new();
        for note in &self.notes {
            let sound = self.instrument.play_midi_note(*note);
            wave.add_consuming(sound, self.time_keeper.stamp_to_samples(note.onset));
        }
        self.effects.apply(&mut wave, self.time_keeper.zero());
        wave.scale(self.gain);
        wave
    }

    pub fn from_instrument(
        instrument: Box<dyn Instrument<W>>,
        time_keeper: Rc<TimeKeeper>,
    ) -> Self {
        Self {
            name: String::from(instrument.name()),
            instrument,
            gain: 1.0,
            effects: EffectNode::<W>::End,
            notes: Vec::new(),
            time_keeper,
        }
    }
}

pub struct Song<W: Wave> {
    name: String,
    tracks: Vec<Track<W>>,
    time_keeper: Rc<TimeKeeper>,
}

impl<W: 'static + Wave> Song<W> {
    pub fn new(name: String) -> Self {
        Self {
            name,
            tracks: Vec::new(),
            time_keeper: Rc::new(TimeKeeper::default()),
        }
    }

    pub fn add_track(&mut self, track: Track<W>) {
        self.tracks.push(track)
    }

    pub fn add_instrument<I: Instrument<W>>(&mut self, instrument: Box<dyn Instrument<W>>) {
        self.tracks.push(Track::from_instrument(
            instrument,
            Rc::clone(&self.time_keeper),
        ))
    }

    pub fn get_wave(&self)-> W{
        let mut wave = W::new();
        for track in &self.tracks{
            wave.add_consuming(track.play(), 0);
        }
        wave
    }
}
