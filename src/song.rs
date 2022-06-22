use crate::midi::{Velocity, Note, Pitch};
use crate::audio_effects::Effect;
use crate::util::{seconds_to_samples, add_from_index};

pub trait Instrument {
    fn play_midi_note(
        &self,
        pitch: Pitch,
        velocity: Velocity,
        duration: f64,
    ) -> Vec<f64>;
    // fn play_test(&self) -> Vec<f64>;
}

pub struct Track<I: Instrument>{
    instrument: I,
    gain: f64,
    effects: Vec<Effect>,
    notes: Vec<Note>,
}

impl<I: Instrument> Track<I>{
    pub fn play(&self) -> Vec<f64>{
        let mut out = Vec::<f64>::new();
        for note in &self.notes{
            let onset = seconds_to_samples(note.onset);
            let sound = self.instrument.play_midi_note(note.pitch, note.velocity, note.duration);
            add_from_index(&mut out, sound, onset);
        };
        for effect in &self.effects{
            todo!()
        }
        out.into_iter().map(|x| x * self.gain).collect()
    }
}