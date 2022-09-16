use serde::{Deserialize, Serialize};

use crate::{
    gens::Error,
    globals::SAMPLE_RATE,
    network::{self, Receiver, Transform},
    time::ClockTick,
    utils,
};

use super::{GenId, Generator};

const ATTACK_RECEIVER: Receiver = Receiver::new(0.1, (0.0, 25.0), Transform::Linear);
const DECAY_RECEIVER: Receiver = Receiver::new(0.2, (0.0, 25.0), Transform::Linear);
const SUSTAIN_RECEIVER: Receiver = Receiver::new(0.75, (0.0, 1.0), Transform::Linear);
const RELEASE_RECEIVER: Receiver = Receiver::new(0.1, (0.0, 25.0), Transform::Linear);
const HALF_LIFE_RECEIVER: Receiver = Receiver::new(0.2, (0.01, 10.0), Transform::Linear);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope {
    id: GenId,
    attack: Receiver,
    decay: Receiver,
    sustain: Receiver,
    half_life: Option<Receiver>,
    release: Receiver,
}

impl Envelope {
    pub fn new() -> Self {
        Self {
            id: GenId::Unbound,
            attack: ATTACK_RECEIVER,
            decay: DECAY_RECEIVER,
            sustain: SUSTAIN_RECEIVER,
            half_life: None,
            release: RELEASE_RECEIVER,
        }
    }

    pub fn wrap(self) -> Generator {
        Generator::Envelope(self)
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
        if let Some(receiver) = &self.half_life {
            out.append(&mut receiver.get_ids());
        }
        out.append(&mut self.release.get_ids());
        out
    }

    pub fn new_decay(decay: f32) -> Result<Self, Error> {
        Ok(Self {
            id: GenId::Unbound,
            attack: ATTACK_RECEIVER.sv(0.0),
            decay: DECAY_RECEIVER.csv(decay)?,
            sustain: SUSTAIN_RECEIVER.sv(0.0),
            half_life: None,
            release: RELEASE_RECEIVER.sv(0.0),
        })
    }

    pub fn new_ad(attack: f32, decay: f32) -> Result<Self, Error> {
        Ok(Self {
            id: GenId::Unbound,
            attack: ATTACK_RECEIVER.csv(attack)?,
            decay: DECAY_RECEIVER.csv(decay)?,
            sustain: SUSTAIN_RECEIVER.sv(0.0),
            half_life: None,
            release: RELEASE_RECEIVER.sv(0.0),
        })
    }

    pub fn new_adsr(attack: f32, decay: f32, sustain: f32, release: f32) -> Result<Self, Error> {
        Ok(Self {
            id: GenId::Unbound,
            attack: ATTACK_RECEIVER.csv(attack)?,
            decay: DECAY_RECEIVER.csv(decay)?,
            sustain: SUSTAIN_RECEIVER.csv(sustain)?,
            half_life: None,
            release: RELEASE_RECEIVER.csv(release)?,
        })
    }

    pub fn new_adsr_with_half_life(
        attack: f32,
        decay: f32,
        sustain: f32,
        half_life: f32,
        release: f32,
    ) -> Result<Envelope, Error> {
        Ok(Self {
            id: GenId::Unbound,
            attack: ATTACK_RECEIVER.csv(attack)?,
            decay: DECAY_RECEIVER.csv(decay)?,
            sustain: SUSTAIN_RECEIVER.csv(sustain)?,
            half_life: Some(HALF_LIFE_RECEIVER.csv(half_life)?),
            release: RELEASE_RECEIVER.csv(release)?,
        })
    }
}

impl Default for Envelope {
    fn default() -> Self {
        Self::new_adsr(0.1, 0.15, 0.8, 0.6).unwrap()
    }
}

impl Envelope {
    pub fn get_envelope(&self, note_on: ClockTick, sus_samples: usize) -> Vec<f32> {
        let attack = utils::seconds_to_samples(self.attack.get_val(note_on));
        let decay = utils::seconds_to_samples(self.decay.get_val(note_on));
        let sustain = self.sustain.get_val(note_on);
        let release = utils::seconds_to_samples(self.release.get_val(note_on));

        let mut out = Vec::with_capacity(sus_samples + release);
        for i in 0..attack {
            out.push((i as f32) / (attack as f32))
        }
        for i in 0..decay {
            out.push((1.0 - (i as f32) / (decay as f32)) * (1.0 - sustain) + sustain)
        }
        if out.len() < sus_samples {
            if let Some(d_ctrl) = &self.half_life {
                let half_life_factor =
                    0.5_f32.powf(1.0 / (d_ctrl.get_val(note_on) * SAMPLE_RATE as f32));
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
            out.push((1.0 - (i as f32) / (release as f32)) * last_sustain)
        }
        out
    }
}

impl Envelope {
    pub fn set_attack(&mut self, attack: &Receiver) -> Result<(), Error> {
        network::set_receiver(&mut self.attack, self.id, attack)
    }

    pub fn set_decay(&mut self, decay: &Receiver) -> Result<(), Error> {
        network::set_receiver(&mut self.decay, self.id, decay)
    }

    pub fn set_sustain(&mut self, sustain: &Receiver) -> Result<(), Error> {
        network::set_receiver(&mut self.sustain, self.id, sustain)
    }

    pub fn set_half_life(&mut self, half_life: &Receiver) -> Result<(), Error> {
        match &mut self.half_life {
            Some(l_half_life) => network::set_receiver(l_half_life, self.id, half_life)?,
            None => {
                let mut halflife = HALF_LIFE_RECEIVER;
                network::set_receiver(&mut halflife, self.id, half_life)?;
                self.half_life = Some(half_life.clone());
            }
        }
        Ok(())
    }

    pub fn set_release(&mut self, release: &Receiver) -> Result<(), Error> {
        network::set_receiver(&mut self.release, self.id, release)
    }
}

impl Envelope {
    pub fn get_vec(&self, _start: ClockTick, _samples: usize) -> Vec<f32> {
        todo!()
    }
}
