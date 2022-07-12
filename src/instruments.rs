use crate::{auto::AutomationKeeper, time::TimeKeeper, tracks::midi, wave};
pub mod empty;
pub mod synth;

pub use empty::EmptyInstrument;
pub use synth::Synthesizer;

pub trait MidiInstrument<W: wave::Wave>: TimeKeeper + AutomationKeeper {
    fn play_note(&self, note: midi::Note) -> W;
    fn play_notes(&self, note: &[midi::Note]) -> W;
    fn name(&self) -> &str;
}
