use std::{cell::RefCell, path::Path, rc::Rc};

use song::{
    consts::SAMPLE_RATE,
    effects::{Delay, Effect},
    instr::synth::OscPanel,
    time::{TimeKeeper, TimeManager, TimeStamp},
    utils::oscs::Oscillator,
    wave::{Mono, Wave},
};

fn main() {
    // let path = Path::new("midi_files/seven8.mid");
    // let song: song::Song<wave::Mono> = io::read_midi_file(path).unwrap();
    // let wave = song.get_wave();
    // let target = Path::new("out/hello_world.wav");
    // io::save_m_i16_wav(wave, target).unwrap();

    let mut instrument = song::instr::Synthesizer::<Mono>::new("first".to_string());
    instrument.set_time_manager(Rc::new(RefCell::new(TimeManager::default())));

    let osc = OscPanel::<Mono>::default();
    println!("{:#?}", instrument);
    let delay = Delay::<Mono>::default();
    let mut w = instrument.play_test_chord();
    let path = Path::new("out/test1.wav");
    // delay.apply(&mut w, TimeStamp::zero());
    w.save(&path);

}
