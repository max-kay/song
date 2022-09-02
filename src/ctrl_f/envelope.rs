use once_cell::sync::Lazy;

use crate::{
    ctrl_f::Error,
    globals::SAMPLE_RATE,
    network::{Reciever, Transform},
    time::TimeStamp,
    utils,
};

static ATTACK_RECIEVER: Lazy<Reciever> =
    Lazy::new(|| Reciever::new(0.1, (0.0, 25.0), Transform::Linear));
static DECAY_RECIEVER: Lazy<Reciever> =
    Lazy::new(|| Reciever::new(0.2, (0.0, 25.0), Transform::Linear));
static SUSTAIN_RECIEVER: Lazy<Reciever> =
    Lazy::new(|| Reciever::new(0.75, (0.0, 1.0), Transform::Linear));
static RELEASE_RECIEVER: Lazy<Reciever> =
    Lazy::new(|| Reciever::new(0.1, (0.0, 25.0), Transform::Linear));
static HALF_LIFE_RECIEVER: Lazy<Reciever> =
    Lazy::new(|| Reciever::new(0.2, (0.01, 10.0), Transform::Linear));

#[derive(Debug)]
pub struct Envelope {
    attack: Reciever,
    decay: Reciever,
    sustain: Reciever,
    half_life: Option<Reciever>,
    release: Reciever,
}

impl Envelope {
    pub fn new() -> Self {
        Self {
            attack: ATTACK_RECIEVER.clone(),
            decay: DECAY_RECIEVER.clone(),
            sustain: SUSTAIN_RECIEVER.clone(),
            half_life: None,
            release: RELEASE_RECIEVER.clone(),
        }
    }

    pub fn new_decay(decay: f64) -> Result<Self, Error> {
        Ok(Self {
            attack: ATTACK_RECIEVER.clone().sv(0.0),
            decay: DECAY_RECIEVER.clone().csv(decay)?,
            sustain: SUSTAIN_RECIEVER.clone().sv(0.0),
            half_life: None,
            release: RELEASE_RECIEVER.clone().sv(0.0),
        })
    }

    pub fn new_ad(attack: f64, decay: f64) -> Result<Self, Error> {
        Ok(Self {
            attack: ATTACK_RECIEVER.clone().csv(attack)?,
            decay: DECAY_RECIEVER.clone().csv(decay)?,
            sustain: SUSTAIN_RECIEVER.clone().sv(0.0),
            half_life: None,
            release: RELEASE_RECIEVER.clone().sv(0.0),
        })
    }

    pub fn new_adsr(attack: f64, decay: f64, sustain: f64, release: f64) -> Result<Self, Error> {
        Ok(Self {
            attack: ATTACK_RECIEVER.clone().csv(attack)?,
            decay: DECAY_RECIEVER.clone().csv(decay)?,
            sustain: SUSTAIN_RECIEVER.clone().csv(sustain)?,
            half_life: None,
            release: RELEASE_RECIEVER.clone().csv(release)?,
        })
    }

    pub fn new_adsr_with_half_life(
        attack: f64,
        decay: f64,
        sustain: f64,
        half_life: f64,
        release: f64,
    ) -> Result<Envelope, Error> {
        Ok(Self {
            attack: ATTACK_RECIEVER.clone().csv(attack)?,
            decay: DECAY_RECIEVER.clone().csv(decay)?,
            sustain: SUSTAIN_RECIEVER.clone().csv(sustain)?,
            half_life: Some(HALF_LIFE_RECIEVER.clone().csv(half_life)?),
            release: RELEASE_RECIEVER.clone().csv(release)?,
        })
    }
}

impl Default for Envelope {
    fn default() -> Self {
        Self::new_adsr(0.1, 0.15, 0.8, 0.6).unwrap()
    }
}

impl Envelope {
    pub fn get_envelope(&self, sus_samples: usize, time: TimeStamp) -> Vec<f64> {
        let attack = utils::seconds_to_samples(self.attack.get_val(time));
        let decay = utils::seconds_to_samples(self.decay.get_val(time));
        let sustain = self.sustain.get_val(time);
        let release = utils::seconds_to_samples(self.release.get_val(time));

        let mut out = Vec::with_capacity(sus_samples + release);
        for i in 0..attack {
            out.push((i as f64) / (attack as f64))
        }
        for i in 0..decay {
            out.push((1.0 - (i as f64) / (decay as f64)) * (1.0 - sustain) + sustain)
        }
        if out.len() < sus_samples {
            if let Some(d_ctrl) = &self.half_life {
                let half_life_factor =
                    0.5_f64.powf(1.0 / (d_ctrl.get_val(time) * SAMPLE_RATE as f64));
                let remaining = sus_samples - out.len();
                for i in 0..remaining {
                    out.push(sustain * half_life_factor.powi(i as i32));
                }
            } else {
                let mut sus = vec![sustain; sus_samples - out.len()];
                out.append(&mut sus);
            }
        }
        let last_sustain = *out.last().expect("error while calculating envelope");
        for i in 0..release {
            out.push((1.0 - (i as f64) / (release as f64)) * last_sustain)
        }
        out
    }
}

impl Envelope {
    pub fn get_vec(&self, start: TimeStamp, samples: usize) -> Vec<f64> {
        let attack = utils::seconds_to_samples(self.attack.get_val(start));
        let decay = utils::seconds_to_samples(self.decay.get_val(start));
        let release = utils::seconds_to_samples(self.release.get_val(start));

        if samples > attack + decay + release {
            let mut vec = self.get_envelope(0, start);
            vec.resize(samples, 0.0);
            vec
        } else {
            self.get_envelope(samples - release, start)
        }
    }
}
