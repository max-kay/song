use crate::{
    ctrl_f::Error,
    globals::SAMPLE_RATE,
    network::{self, Reciever, Transform},
    time::TimeStamp,
    utils,
};

use super::{GenId, Generator};

const ATTACK_RECIEVER: Reciever = Reciever::new(0.1, (0.0, 25.0), Transform::Linear);
const DECAY_RECIEVER: Reciever = Reciever::new(0.2, (0.0, 25.0), Transform::Linear);
const SUSTAIN_RECIEVER: Reciever = Reciever::new(0.75, (0.0, 1.0), Transform::Linear);
const RELEASE_RECIEVER: Reciever = Reciever::new(0.1, (0.0, 25.0), Transform::Linear);
const HALF_LIFE_RECIEVER: Reciever = Reciever::new(0.2, (0.01, 10.0), Transform::Linear);

#[derive(Debug)]
pub struct Envelope {
    id: GenId,
    attack: Reciever,
    decay: Reciever,
    sustain: Reciever,
    half_life: Option<Reciever>,
    release: Reciever,
}

impl Envelope {
    pub fn new() -> Self {
        Self {
            id: GenId::Unbound,
            attack: ATTACK_RECIEVER,
            decay: DECAY_RECIEVER,
            sustain: SUSTAIN_RECIEVER,
            half_life: None,
            release: RELEASE_RECIEVER,
        }
    }

    pub fn w_default() -> Generator {
        Generator::Envelope(Self::default())
    }

    pub(crate) fn set_id(&mut self, id: GenId) {
        self.id = id
    }

    pub fn get_sub_ids(&self) -> Vec<GenId> {
        let mut out = self.attack.get_ids();
        out.append(&mut self.decay.get_ids());
        out.append(&mut self.sustain.get_ids());
        if let Some(reciever) = &self.half_life {
            out.append(&mut reciever.get_ids());
        }
        out.append(&mut self.release.get_ids());
        out
    }

    pub fn new_decay(decay: f64) -> Result<Self, Error> {
        Ok(Self {
            id: GenId::Unbound,
            attack: ATTACK_RECIEVER.sv(0.0),
            decay: DECAY_RECIEVER.csv(decay)?,
            sustain: SUSTAIN_RECIEVER.sv(0.0),
            half_life: None,
            release: RELEASE_RECIEVER.sv(0.0),
        })
    }

    pub fn new_ad(attack: f64, decay: f64) -> Result<Self, Error> {
        Ok(Self {
            id: GenId::Unbound,
            attack: ATTACK_RECIEVER.csv(attack)?,
            decay: DECAY_RECIEVER.csv(decay)?,
            sustain: SUSTAIN_RECIEVER.sv(0.0),
            half_life: None,
            release: RELEASE_RECIEVER.sv(0.0),
        })
    }

    pub fn new_adsr(attack: f64, decay: f64, sustain: f64, release: f64) -> Result<Self, Error> {
        Ok(Self {
            id: GenId::Unbound,
            attack: ATTACK_RECIEVER.csv(attack)?,
            decay: DECAY_RECIEVER.csv(decay)?,
            sustain: SUSTAIN_RECIEVER.csv(sustain)?,
            half_life: None,
            release: RELEASE_RECIEVER.csv(release)?,
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
            id: GenId::Unbound,
            attack: ATTACK_RECIEVER.csv(attack)?,
            decay: DECAY_RECIEVER.csv(decay)?,
            sustain: SUSTAIN_RECIEVER.csv(sustain)?,
            half_life: Some(HALF_LIFE_RECIEVER.csv(half_life)?),
            release: RELEASE_RECIEVER.csv(release)?,
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
    pub fn set_attack(&mut self, attack: &Reciever) -> Result<(), Error> {
        network::set_reciever(&mut self.attack, self.id, attack)
    }

    pub fn set_decay(&mut self, decay: &Reciever) -> Result<(), Error> {
        network::set_reciever(&mut self.decay, self.id, decay)
    }

    pub fn set_sustain(&mut self, sustain: &Reciever) -> Result<(), Error> {
        network::set_reciever(&mut self.sustain, self.id, sustain)
    }

    pub fn set_half_life(&mut self, half_life: &Reciever) -> Result<(), Error> {
        match &mut self.half_life {
            Some(l_half_life) => network::set_reciever(l_half_life, self.id, half_life)?,
            None => {
                let mut halflife = HALF_LIFE_RECIEVER;
                network::set_reciever(&mut halflife, self.id, half_life)?;
                self.half_life = Some(half_life.clone());
            }
        }
        Ok(())
    }

    pub fn set_release(&mut self, release: &Reciever) -> Result<(), Error> {
        network::set_reciever(&mut self.release, self.id, release)
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
