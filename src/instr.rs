use std::fmt::Debug;

use crate::{tracks::midi, wave::Wave};
pub mod empty;
pub mod synth;

pub use empty::EmptyInstrument;
pub use synth::Synthesizer;

pub trait MidiInstrument<W: Wave>: Debug {
    fn play_note(&self, note: midi::Note) -> W;
    fn play_notes(&self, note: &[midi::Note]) -> W;
    fn name(&self) -> &str;
}
