use std::path::Path;

use audio_effects::Effect;
use automation::ValAndCh;

pub mod audio_effects;
pub mod automation;
pub mod constants;
pub mod io;
pub mod midi;
pub mod song;
pub mod synth;
pub mod utils;

fn main() {
    let name = String::from("first");
    let oscillator = synth::Oscillator::Sine(1.0);
    let envelope = synth::Envelope::new_adsr(0.1, 0.1, 0.5, 10.0);
    let oscillators = vec![oscillator];
    let synth = synth::Synthesizer::new(name, envelope, oscillators);

    let automation = automation::AutomationManager::new();

    let delay = audio_effects::Delay {
        time_delta: ValAndCh {
            value: 0.5,
            connection: None,
        },
        gain: ValAndCh {
            value: 0.8,
            connection: None,
        },
    };

    let mut sound = synth.play_test_chord();
    delay.apply(&mut sound, &automation);

    let path = Path::new("out/delay_test.wav");
    io::easy_save(sound, path);
}
