use crate::constants::SAMPLE_RATE;
use crate::io;
use crate::midi::{Pitch, Velocity};
use crate::song;
use crate::util::{add_same_len, seconds_to_samples};
use std::f64::consts::{PI, TAU};
use std::path::Path;

pub enum Envelope {
    Decay {
        decay: usize,
    },
    Ad {
        attack: usize,
        decay: usize,
    },
    Adsr {
        attack: usize,
        decay: usize,
        sustain: f64,
        release: usize,
    },
    AdsrDecayed {
        attack: usize,
        decay: usize,
        sustain: f64,
        sus_decay: f64,
        release: usize,
    },
}

impl Envelope {
    pub fn play(&self, sus_samples: usize, velocity: Velocity) -> Vec<f64> {
        // velocity here?
        let out = match self {
            Self::Decay { decay } => {
                let mut out = Vec::with_capacity(*decay);
                for i in 0..*decay {
                    out.push(1.0 - (i as f64) / (*decay as f64))
                }
                out
            }
            Self::Ad { attack, decay } => {
                let mut out = Vec::with_capacity(*attack + *decay);
                for i in 0..*attack {
                    out.push((i as f64) / (*attack as f64))
                }
                for i in 0..*decay {
                    out.push(1.0 - (i as f64) / (*decay as f64))
                }
                out
            }
            Self::Adsr {
                attack,
                decay,
                sustain,
                release,
            } => {
                let mut out = Vec::with_capacity(sus_samples + *release);
                for i in 0..*attack {
                    out.push((i as f64) / (*attack as f64))
                }
                for i in 0..*decay {
                    out.push((1.0 - (i as f64) / (*decay as f64)) * (1.0 - sustain) + sustain)
                }
                if (attack + decay) < sus_samples {
                    let mut sus = vec![*sustain; sus_samples - attack - decay];
                    out.append(&mut sus);
                }
                for i in 0..*release {
                    out.push((1.0 - (i as f64) / (*release as f64)) * sustain)
                }
                out
            }
            Self::AdsrDecayed {
                attack,
                decay,
                sustain,
                sus_decay,
                release,
            } => {
                let mut out = Vec::with_capacity(sus_samples + *release);
                for i in 0..*attack {
                    out.push((i as f64) / (*attack as f64))
                }
                for i in 0..*decay {
                    out.push((1.0 - (i as f64) / (*decay as f64)) * (1.0 - sustain) + sustain)
                }
                if (attack + decay) < sus_samples {
                    for i in 0..sus_samples {
                        out.push(sustain * (sus_decay * (i as f64) / (SAMPLE_RATE as f64)));
                    }
                }
                let last_sustain = *out.last().expect("error while calculating envelope");
                for i in 0..*release {
                    out.push((1.0 - (i as f64) / (*release as f64)) * last_sustain)
                }
                out
            }
        };
        out.into_iter()
            .map(|x| x * (velocity.get() as f64) / 127.0)
            .collect()
    }

    pub fn new_decay(decay: f64) -> Self {
        Self::Decay {
            decay: seconds_to_samples(decay),
        }
    }

    pub fn new_ad(attack: f64, decay: f64) -> Self {
        Self::Ad {
            attack: seconds_to_samples(attack),
            decay: seconds_to_samples(decay),
        }
    }

    pub fn new_adsr(attack: f64, decay: f64, sustain: f64, release: f64) -> Self {
        Self::Adsr {
            attack: seconds_to_samples(attack),
            decay: seconds_to_samples(decay),
            sustain,
            release: seconds_to_samples(release),
        }
    }

    pub fn new_adsr_decayed(
        attack: f64,
        decay: f64,
        sustain: f64,
        sus_decay: f64,
        release: f64,
    ) -> Self {
        Self::AdsrDecayed {
            attack: seconds_to_samples(attack),
            decay: seconds_to_samples(decay),
            sustain,
            sus_decay,
            release: seconds_to_samples(release),
        }
    }
}

// impl Sustain {
//     pub fn play(&self, samples: usize) -> Vec<f64> {
//         match self {
//             Self::Hold(val) => vec![*val; samples],
//             Self::Decay(val, decay) => {
//                 let mut out = Vec::with_capacity(samples);
//                 for i in 0..samples {
//                     out.push(val * (decay * (i as f64) / (SAMPLE_RATE as f64)));
//                 }
//                 out
//             }
//         }
//     }
// }
pub enum Oscillator {
    Sine(f64),
    ModSquare(f64, f64),
    ModSaw(f64, f64),
}

impl Oscillator {
    pub fn play_freq(&self, freq: f64, samples: usize) -> Vec<f64> {
        let ang_vel = TAU * freq;
        let mut out = Vec::with_capacity(samples);
        match self {
            Self::Sine(vol) => {
                let scale = ang_vel / (SAMPLE_RATE as f64);
                for i in 0..samples {
                    out.push((scale * (i as f64)).sin() * vol)
                }
                out
            }
            Self::ModSquare(vol, modulation) => {
                for i in 0..samples {
                    let phase = (i as f64) * ang_vel / (SAMPLE_RATE as f64) % TAU;
                    if phase < *modulation * TAU {
                        out.push(*vol)
                    } else {
                        out.push(-vol)
                    }
                }
                out
            }
            Self::ModSaw(vol, modulation) => {
                for i in 0..samples {
                    let phase = (i as f64) * ang_vel / (SAMPLE_RATE as f64) % TAU;
                    if phase < *modulation * TAU {
                        out.push((phase / modulation / PI - 1.0) * vol)
                    } else {
                        out.push(
                            ((phase - (modulation + 1.0) * PI) / (modulation - 1.0) / PI) * vol,
                        )
                    }
                }
                out
            }
        }
    }
}

pub struct Synthesizer {
    name: String,
    envelope: Envelope,
    oscillators: Vec<Oscillator>,
}

impl Synthesizer {
    pub fn new(name: String, envelope: Envelope, oscillators: Vec<Oscillator>) -> Self {
        Self {
            name,
            envelope,
            oscillators,
        }
    }

    pub fn play_freq(&self, freq: f64, note_held: f64, velocity: Velocity) -> Vec<f64> {
        let sus_samples = (note_held * (SAMPLE_RATE as f64)) as usize;
        let envelope = self.envelope.play(sus_samples, velocity);
        let mut out = vec![0.0; envelope.len()];
        for osc in &self.oscillators {
            add_same_len(&mut out, osc.play_freq(freq, envelope.len()));
        }
        add_same_len(&mut out, envelope);
        out
    }

    pub fn play_test_chord(&self) -> Vec<f64> {
        let mut out = self.play_freq(300.0, 2.0, Velocity::new(80).unwrap());
        add_same_len(&mut out, self.play_freq(375.0, 2.0, Velocity::new(80).unwrap()));
        add_same_len(&mut out, self.play_freq(450.0, 2.0, Velocity::new(80).unwrap()));
        add_same_len(&mut out, self.play_freq(600.0, 2.0, Velocity::new(80).unwrap()));
        out
    }

    pub fn save_test_chord(&self) {
        let track = self.play_test_chord();
        let path = format!("out/synthtest/{}_chord.wav", self.name);
        let path = Path::new(&path);
        io::easy_save(track, path);
    }
}

impl song::Instrument for Synthesizer {
    fn play_midi_note(&self, pitch: Pitch, velocity: Velocity, duration: f64) -> Vec<f64> {
        self.play_freq(pitch.get_freq(), duration, velocity)
    }
}
