use song::{
    instr::{synth::OscPanel, MidiInstrument, Synthesizer},
    tracks::midi,
    utils::oscs::Oscillator,
    Song,
};

fn main() {
    let mut song = Song::new("my Song".to_string());
    let instr: Box<dyn MidiInstrument> = Box::new(Synthesizer::new("synth".to_string()));
    let track = midi::MidiTrack::from_instrument(instr);
    song.add_midi_track(track).expect("hh");
    let mref = song
        .get_instr_as_any(0)
        .downcast_mut::<Synthesizer>()
        .expect("ollala");
    let osc_panel = OscPanel::from_oscs(
        vec![
            Oscillator::ModSaw,
            Oscillator::ModSaw,
            Oscillator::ModSquare,
        ],
        None,
    )
    .unwrap();
    mref.oscillators = osc_panel;
    let string = ron::ser::to_string_pretty(&song, Default::default()).unwrap();
    println!("{}", string)
}
