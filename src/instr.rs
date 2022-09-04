use std::{any::Any, fmt::Debug};

use crate::{tracks::midi, wave::Wave, Error};
pub mod empty;
pub mod synth;

pub use empty::EmptyInstrument;
pub use synth::Synthesizer;

pub trait MidiInstrument: Debug + serde_traitobject::Serialize + serde_traitobject::Deserialize {
    fn play_note(&self, note: midi::Note) -> Wave;
    fn play_notes(&self, note: &[midi::Note]) -> Wave;
    fn name(&self) -> &str;
    fn put_in_song(&mut self, id: u8) -> Result<(), Error>;
    fn as_any(&mut self) -> &mut dyn Any;
}
