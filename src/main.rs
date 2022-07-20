use std::{cell::RefCell, path::Path, rc::Rc};

use song::{
    ctrl_f::Lfo,
    effects::{Delay, Effect},
    instr::Synthesizer,
    time::{TimeKeeper, TimeManager, TimeStamp},
    utils::oscs::Oscillator,
    wave::{Stereo, Wave},
};

fn main() {
    let mut instrument = Synthesizer::<Stereo>::new("first".to_string());
    instrument.add_osc(Oscillator::ModSaw);
    instrument
        .set_lfo1(Lfo::new(Oscillator::ModSaw, 1.0, 0.0, 0.0).unwrap())
        .unwrap();
    instrument.set_vol_source(instrument.get_lfo1());
    instrument.set_time_manager(Rc::new(RefCell::new(TimeManager::default())));

    let delay = Delay::<Stereo>::default();
    let mut w = instrument.play_test_chord();

    w.peak_normalize();
    delay.apply(&mut w, TimeStamp::zero());
    let path = Path::new("out/test1.wav");
    w.save(path);
}
