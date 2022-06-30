use crate::{consts::SAMPLE_RATE, utils::seconds_to_samples};

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

impl Default for Envelope {
    fn default() -> Self {
        Envelope::Adsr {
            attack: 4000,
            decay: 6000,
            sustain: 0.8,
            release: 10000,
        }
    }
}

impl Envelope {
    pub fn get_envelope(&self, sus_samples: usize) -> Vec<f64> {
        match self {
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
        }
    }
}
