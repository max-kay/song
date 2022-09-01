use std::fmt::Debug;
use crate::{control::FunctionKeeper, ctrl_f::FunctionMngrKeeper, tracks::midi, wave::Wave};

pub mod empty;
pub mod synth;

use dyn_clone::DynClone;
pub use empty::EmptyInstrument;
pub use synth::Synthesizer;

#[typetag::serde]
pub trait MidiInstrument:
    FunctionKeeper + FunctionMngrKeeper + Debug + DynClone
{
    fn play_note(&self, note: midi::Note) -> Wave;
    fn play_notes(&self, note: &[midi::Note]) -> Wave;
    fn name(&self) -> &str;
}
