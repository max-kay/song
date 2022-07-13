use super::{Control, CtrlFunction};
use crate::{
    consts::SAMPLE_RATE,
    time::{TimeKeeper, TimeManager, TimeStamp},
    utils,
};
use std::{cell::RefCell, rc::Rc};

#[derive(Debug)]
pub struct Envelope {
    attack: Option<Control>,
    decay: Option<Control>,
    sustain: Option<Control>,
    sus_half_life: Option<Control>,
    release: Option<Control>,
    time_manager: Rc<RefCell<TimeManager>>,
}

impl Envelope {
    pub fn new_decay(decay: f64) -> Self {
        Self {
            attack: None,
            decay: Some(Control::from_val_in_range(decay, (0.0, 25.0))),
            sustain: None,
            sus_half_life: None,
            release: None,
            time_manager: Rc::new(RefCell::new(TimeManager::default())),
        }
    }

    pub fn new_ad(attack: f64, decay: f64) -> Self {
        Self {
            attack: Some(Control::from_val_in_range(attack, (0.0, 25.0))),
            decay: Some(Control::from_val_in_range(decay, (0.0, 25.0))),
            sustain: None,
            sus_half_life: None,
            release: None,
            time_manager: Rc::new(RefCell::new(TimeManager::default())),
        }
    }

    pub fn new_adsr(attack: f64, decay: f64, sustain: f64, release: f64) -> Self {
        Self {
            attack: Some(Control::from_val_in_range(attack, (0.0, 25.0))),
            decay: Some(Control::from_val_in_range(decay, (0.0, 25.0))),
            sustain: Some(Control::from_val_in_unit(sustain)),
            sus_half_life: None,
            release: Some(Control::from_val_in_range(release, (0.0, 25.0))),
            time_manager: Rc::new(RefCell::new(TimeManager::default())),
        }
    }

    pub fn new_adsr_with_half_life(
        attack: f64,
        decay: f64,
        sustain: f64,
        sus_half_life: f64,
        release: f64,
    ) -> Self {
        Self {
            attack: Some(Control::from_val_in_range(attack, (0.0, 25.0))),
            decay: Some(Control::from_val_in_range(decay, (0.0, 25.0))),
            sustain: Some(Control::from_val_in_unit(sustain)),
            sus_half_life: Some(Control::from_val_in_range(sus_half_life, (0.01, 10.0))),
            release: Some(Control::from_val_in_range(release, (0.0, 25.0))),
            time_manager: Rc::new(RefCell::new(TimeManager::default())),
        }
    }
}

impl Default for Envelope {
    fn default() -> Self {
        Self::new_adsr(0.1, 0.15, 0.8, 0.6)
    }
}

impl Envelope {
    fn get_vals(&self, time: TimeStamp) -> (usize, usize, f64, usize) {
        let attack = match &self.attack {
            Some(ctrl) => utils::seconds_to_samples(ctrl.get_value(time)),
            None => 0,
        };

        let decay = match &self.decay {
            Some(ctrl) => utils::seconds_to_samples(ctrl.get_value(time)),
            None => 0,
        };

        let sustain = match &self.sustain {
            Some(ctrl) => ctrl.get_value(time),
            None => 0.0,
        };

        let release = match &self.release {
            Some(ctrl) => utils::seconds_to_samples(ctrl.get_value(time)),
            None => 0,
        };
        (attack, decay, sustain, release)
    }

    pub fn get_envelope(&self, sus_samples: usize, time: TimeStamp) -> Vec<f64> {
        let (attack, decay, sustain, release) = self.get_vals(time);

        let mut out = Vec::with_capacity(sus_samples + release);
        for i in 0..attack {
            out.push((i as f64) / (attack as f64))
        }
        for i in 0..decay {
            out.push((1.0 - (i as f64) / (decay as f64)) * (1.0 - sustain) + sustain)
        }
        if out.len() < sus_samples {
            if let Some(d_ctrl) = &self.sus_half_life {
                let sus_half_life_factor =
                    0.5_f64.powf(1.0 / (d_ctrl.get_value(time) * SAMPLE_RATE as f64));
                let remaining = sus_samples - out.len();
                for i in 0..remaining {
                    out.push(sustain * sus_half_life_factor.powi(i as i32));
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
    pub fn set(&mut self, other: Envelope) {
        self.attack = other.attack;
        self.decay = other.decay;
        self.sustain = other.sustain;
        self.sus_half_life = other.sus_half_life;
        self.release = other.release;
    }
}

impl TimeKeeper for Envelope {
    fn set_time_manager(&mut self, time_manager: Rc<RefCell<crate::time::TimeManager>>) {
        self.time_manager = Rc::clone(&time_manager)
    }
}

impl CtrlFunction for Envelope {
    fn get_value(&self, _time: TimeStamp) -> f64 {
        panic!()
    }

    fn get_vec(&self, start: TimeStamp, samples: usize) -> Vec<f64> {
        let (attack, decay, _, release) = self.get_vals(start);
        if samples > attack + decay + release {
            let mut vec = self.get_envelope(0, start);
            vec.resize(samples, 0.0);
            vec
        } else {
            self.get_envelope(samples - release, start)
        }
    }
}

#[cfg(test)]
mod test {}
