pub mod empty;
pub mod synth;

use crate::time;
use crate::tracks::midi;
use crate::wave;

pub use empty::EmptyInstrument;
pub use synth::Synthesizer;

pub trait MidiInstrument<W: wave::Wave> {
    fn play_note(&self, note: midi::Note) -> W;
    fn play_notes(&self, note: &Vec<midi::Note>) -> W;
    fn name(&self) -> &str;
}
