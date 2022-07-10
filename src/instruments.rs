pub mod empty;
pub mod synth;

use std::rc::Rc;

use crate::auto::AutomationManager;
use crate::time::TimeKeeper;
use crate::tracks::midi;
use crate::wave;

pub use empty::EmptyInstrument;
pub use synth::Synthesizer;

pub trait MidiInstrument<W: wave::Wave>: TimeKeeper {
    fn play_note(&self, note: midi::Note) -> W;
    fn play_notes(&self, note: &[midi::Note]) -> W;
    fn name(&self) -> &str;
    fn set_track_automation(&mut self, automation: &Rc<AutomationManager>);
}
