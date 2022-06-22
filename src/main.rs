pub mod constants;
pub mod io;
pub mod synth;
pub mod util;
pub mod audio_effects;
pub mod song;
pub mod midi;


fn main() {
    let name = String::from("first");
    let oscillator = synth::Oscillator::Sine(1.0);
    let envelope = synth::Envelope::new_adsr(0.1, 0.1, 0.5, 10.0);
    let oscillators = vec![oscillator];
    let synth = synth::Synthesizer::new(name, envelope, oscillators);

    synth.save_test_chord();
}
