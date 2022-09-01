use ron::ser;
use std::{cell::RefCell, fs, path::Path, rc::Rc};

use song::{
    ctrl_f::Lfo,
    effects::{Delay, EffectPanel},
    instr::{Synthesizer, MidiInstrument},
    time::{TimeKeeper, TimeManager},
    utils::oscs::Oscillator, Song, tracks::MidiTrack,
};

fn main() {
    let mut instrument = Synthesizer::new("first".to_string());
    instrument.add_osc(Oscillator::ModSaw);
    instrument
        .set_lfo1(Lfo::new(Oscillator::ModSaw, 1.0, 0.0, 0.0).unwrap())
        .unwrap();
    instrument.set_vol_source(instrument.get_lfo1());
    instrument.set_time_manager(Rc::new(RefCell::new(TimeManager::default())));


    let delay = Delay::default();
    let effects = EffectPanel::Leaf(Box::new(delay));
    instrument.set_effects(effects);
    
    let track = MidiTrack::from_instrument(Box::new(instrument) as Box<dyn MidiInstrument>);
    let mut song = Song::new("hello");
    song.add_midi_track(track);
    
    // let mut w = instrument.play_test_chord();
    // w.peak_normalize();
    // delay.apply(&mut w, TimeStamp::zero());
    // let path = Path::new("out/test1.wav");
    // w.save(path);
    let path = Path::new("instruments/1.ron");
    let config = ser::PrettyConfig::default();
    fs::write(path, ser::to_string_pretty(&song, config).unwrap()).unwrap();
}
