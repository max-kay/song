use super::{CtrlFunction, IdMap, FunctionKeeper};
use crate::{
    consts::SAMPLE_RATE,
    control::{self, Control, ControlError},
    time::{TimeKeeper, TimeManager, TimeStamp},
    utils,
};
use std::{cell::RefCell, rc::Rc};

const TIME_RANGE: (f64, f64) = (0.0, 25.0);
const HALF_LIFE_RANGE: (f64, f64) = (0.01, 10.0);

#[derive(Debug)]
pub struct Envelope {
    attack: Control,
    decay: Control,
    sustain: Control,
    half_life: Option<Control>,
    release: Control,
    time_manager: Rc<RefCell<TimeManager>>,
    id: usize,
}

impl Envelope {
    pub fn new(
        attack: f64,
        decay: f64,
        sustain: f64,
        half_life: Option<f64>,
        release: f64,
    ) -> Result<Self, ControlError> {
        Ok(Self {
            attack: match Control::from_val_in_range(attack, TIME_RANGE) {
                Ok(ctrl) => ctrl,
                Err(err) => return Err(err.set_origin("Envelope", "attack")),
            },
            decay: match Control::from_val_in_range(decay, TIME_RANGE) {
                Ok(ctrl) => ctrl,
                Err(err) => return Err(err.set_origin("Envelope", "decay")),
            },
            sustain: match Control::from_val_in_unit(sustain) {
                Ok(ctrl) => ctrl,
                Err(err) => return Err(err.set_origin("Envelope", "sustain")),
            },
            half_life: match half_life {
                Some(half_life) => match Control::from_val_in_range(half_life, HALF_LIFE_RANGE) {
                    Ok(ctrl) => Some(ctrl),
                    Err(err) => return Err(err.set_origin("Envelope", "sustain half life")),
                },
                None => None,
            },
            release: match Control::from_val_in_range(release, TIME_RANGE) {
                Ok(ctrl) => ctrl,
                Err(err) => return Err(err.set_origin("Envelope", "release")),
            },
            time_manager: Rc::new(RefCell::new(TimeManager::default())),
            id: utils::get_ctrl_id(),
        })
    }

    pub fn new_decay(decay: f64) -> Result<Envelope, ControlError> {
        Self::new(0.0, decay, 0.0, None, 0.0)
    }

    pub fn new_ad(attack: f64, decay: f64) -> Result<Self, ControlError> {
        Self::new(attack, decay, 0.0, None, 0.0)
    }

    pub fn new_adsr(
        attack: f64,
        decay: f64,
        sustain: f64,
        release: f64,
    ) -> Result<Self, ControlError> {
        Self::new(attack, decay, sustain, None, release)
    }

    pub fn new_adsr_with_half_life(
        attack: f64,
        decay: f64,
        sustain: f64,
        half_life: f64,
        release: f64,
    ) -> Result<Envelope, ControlError> {
        Self::new(attack, decay, sustain, Some(half_life), release)
    }
}

impl Default for Envelope {
    fn default() -> Self {
        Self::new_adsr(0.1, 0.15, 0.8, 0.6).unwrap()
    }
}

impl Envelope {
    pub fn get_envelope(&self, sus_samples: usize, time: TimeStamp) -> Vec<f64> {
        let attack = utils::seconds_to_samples(self.attack.get_value(time));
        let decay = utils::seconds_to_samples(self.decay.get_value(time));
        let sustain = self.sustain.get_value(time);
        let release = utils::seconds_to_samples(self.release.get_value(time));

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
                    0.5_f64.powf(1.0 / (d_ctrl.get_value(time) * SAMPLE_RATE as f64));
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
    pub fn set(&mut self, other: Envelope) -> Result<(), ControlError> {
        self.set_attack(other.attack)?;
        self.set_decay(other.decay)?;
        self.set_sustain(other.sustain)?;
        if let Some(half_life) = other.half_life {
            self.set_half_life(half_life)?
        }
        self.set_release(other.release)?;
        Ok(())
    }

    pub fn set_attack(&mut self, attack_ctrl: Control) -> Result<(), ControlError> {
        self.attack
            .try_set_checked(attack_ctrl, self.id)
            .map_err(|err| err.set_origin("Lfo", "attack"))
    }

    pub fn set_decay(&mut self, decay_ctrl: Control) -> Result<(), ControlError> {
        self.decay
            .try_set_checked(decay_ctrl, self.id)
            .map_err(|err| err.set_origin("Lfo", "decay"))
    }

    pub fn set_sustain(&mut self, sustain_ctrl: Control) -> Result<(), ControlError> {
        self.sustain
            .try_set_checked(sustain_ctrl, self.id)
            .map_err(|err| err.set_origin("Lfo", "sustain"))
    }

    pub fn set_half_life(&mut self, half_life_ctrl: Control) -> Result<(), ControlError> {
        control::opt_try_set_checked(
            &mut self.half_life,
            HALF_LIFE_RANGE,
            half_life_ctrl,
            self.id,
        )
        .map_err(|err| err.set_origin("Envelope", "halflife"))
    }

    pub fn set_release(&mut self, release_ctrl: Control) -> Result<(), ControlError> {
        self.release
            .try_set_checked(release_ctrl, self.id)
            .map_err(|err| err.set_origin("Lfo", "release"))
    }
}

impl TimeKeeper for Envelope {
    fn set_time_manager(&mut self, time_manager: Rc<RefCell<TimeManager>>) {
        self.time_manager = Rc::clone(&time_manager)
    }
}

impl FunctionKeeper for Envelope {
    fn heal_sources(&mut self, id_map: &IdMap) -> Result<(), ControlError> {
        self.attack
            .heal_sources(id_map)
            .map_err(|err| err.set_origin("Envelope", "attack"))?;
        self.decay
            .heal_sources(id_map)
            .map_err(|err| err.set_origin("Envelope", "decay"))?;
        self.sustain
            .heal_sources(id_map)
            .map_err(|err| err.set_origin("Envelope", "sustain"))?;
        self.release
            .heal_sources(id_map)
            .map_err(|err| err.set_origin("Envelope", "release"))?;
        if let Some(half_life) = &mut self.half_life {
            half_life
                .heal_sources(id_map)
                .map_err(|err| err.set_origin("Envelope", "half_life"))?
        };
        Ok(())
    }

    fn test_sources(&self) -> Result<(), ControlError> {
        self.attack
            .test_sources()
            .map_err(|err| err.set_origin("Envelope", "attack"))?;
        self.decay
            .test_sources()
            .map_err(|err| err.set_origin("Envelope", "decay"))?;
        self.sustain
            .test_sources()
            .map_err(|err| err.set_origin("Envelope", "sustain"))?;
        self.release
            .test_sources()
            .map_err(|err| err.set_origin("Envelope", "release"))?;
        if let Some(half_life) = &self.half_life {
            half_life
                .test_sources()
                .map_err(|err| err.set_origin("Envelope", "half_life"))?
        };
        Ok(())
    }

    fn set_ids(&mut self) {
        self.attack.set_ids();
        self.decay.set_ids();
        self.sustain.set_ids();
        self.release.set_ids();
        if let Some(half_life) = &mut self.half_life {
            half_life.set_ids()
        }
    }

    fn get_ids(&self) -> Vec<usize> {
        let mut ids = vec![self.get_id()];
        ids.append(&mut self.attack.get_ids());
        ids.append(&mut self.decay.get_ids());
        ids.append(&mut self.sustain.get_ids());
        if let Some(func) = &self.half_life {
            ids.append(&mut func.get_ids())
        };
        ids.append(&mut self.release.get_ids());
        ids
    }
}

impl CtrlFunction for Envelope {
    fn get_value(&self, _time: TimeStamp) -> f64 {
        panic!("an Envelope cannot be bound to a reciever which requries getting a value")
    }

    fn get_vec(&self, start: TimeStamp, samples: usize) -> Vec<f64> {
        let attack = utils::seconds_to_samples(self.attack.get_value(start));
        let decay = utils::seconds_to_samples(self.decay.get_value(start));
        let release = utils::seconds_to_samples(self.release.get_value(start));

        if samples > attack + decay + release {
            let mut vec = self.get_envelope(0, start);
            vec.resize(samples, 0.0);
            vec
        } else {
            self.get_envelope(samples - release, start)
        }
    }

    fn get_id(&self) -> usize {
        self.id
    }

    unsafe fn new_id_f(&mut self) {
        self.id = utils::get_ctrl_id()
    }

    // fn get_sub_ids(&self) -> Vec<usize> {
    //     let mut ids = Vec::new();
    //     ids.append(&mut self.attack.get_ids());
    //     ids.append(&mut self.decay.get_ids());
    //     ids.append(&mut self.sustain.get_ids());
    //     if let Some(half_life) = &self.half_life {
    //         ids.append(&mut half_life.get_ids())
    //     }
    //     ids.append(&mut self.release.get_ids());
    //     ids
    // }
}

#[cfg(test)]
mod test {}
