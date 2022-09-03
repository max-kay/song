use std::fmt::Debug;

use crate::{tracks::midi, wave::Wave, Error};
pub mod empty;
pub mod synth;

pub use empty::EmptyInstrument;
pub use synth::Synthesizer;

pub trait MidiInstrument: Debug {
    fn play_note(&self, note: midi::Note) -> Wave;
    fn play_notes(&self, note: &[midi::Note]) -> Wave;
    fn name(&self) -> &str;
    fn put_in_song(&mut self, id: u8) -> Result<(), Error>;
}
