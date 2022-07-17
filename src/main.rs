use std::{cell::RefCell, path::Path, rc::Rc};

use song::{
    auto::{Envelope, Lfo},
    effects::{Delay, Effect},
    time::{TimeKeeper, TimeManager, TimeStamp},
    utils::oscs::Oscillator,
    wave::{Mono, Wave},
};

fn main() {
    let mut instrument = song::instr::Synthesizer::<Mono>::new("first".to_string());
    instrument.set_main_envelope(Envelope::default());
    instrument.add_osc(Oscillator::ModSaw);
    instrument.set_lfo1(Lfo::new(Oscillator::ModSaw, 1.0, 0.0, 0.0).unwrap());
    instrument.set_vol_control(instrument.get_lfo1());
    instrument.set_time_manager(Rc::new(RefCell::new(TimeManager::default())));

    let delay = Delay::<Mono>::default();
    let mut w = instrument.play_test_chord();

    w.peak_normalize();
    delay.apply(&mut w, TimeStamp::zero());
    let path = Path::new("out/test1.wav");
    w.save(path).unwrap();
}
