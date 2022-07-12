use super::CtrlFunction;
use crate::{consts::SAMPLE_RATE, time, utils::seconds_to_samples};
use std::{cell::RefCell, ops::Deref, rc::Rc};

pub trait Envelope: CtrlFunction  {
    fn get_envelope(&self, sus_samples: usize) -> Vec<f64>;
}

#[derive(Debug)]
pub struct Decay {
    decay: f64,
    time_manager: Rc<RefCell<time::TimeManager>>,
}

impl Decay {
    pub fn new(decay: f64) -> Self {
        Self {
            decay,
            time_manager: Rc::new(RefCell::new(time::TimeManager::default())),
        }
    }
    pub fn default() -> Self {
        Self {
            decay: 0.8,
            time_manager: Rc::new(RefCell::new(time::TimeManager::default())),
        }
    }
}

impl time::TimeKeeper for Decay {
    fn set_time_manager(&mut self, time_manager: Rc<RefCell<time::TimeManager>>) {
        self.time_manager = Rc::clone(&time_manager)
    }
}

impl super::CtrlFunction for Decay {
    fn get_value(&self, time: time::TimeStamp) -> f64 {
        todo!()
    }

    fn get_vec(&self, start: time::TimeStamp, samples: usize) -> Vec<f64> {
        todo!()
    }

    fn trigger(&self, samples: usize) -> Vec<f64> {
        todo!()
    }
}

impl Deref for Decay {
    type Target = dyn CtrlFunction;

    fn deref(&self) -> &Self::Target {
        self
    }
}

impl Envelope for Decay {
    fn get_envelope(&self, _sus_samples: usize) -> Vec<f64> {
        let mut out = Vec::with_capacity(seconds_to_samples(self.decay));
        for i in 0..seconds_to_samples(self.decay) {
            out.push(1.0 - (i as f64) / (self.decay * (SAMPLE_RATE as f64)))
        }
        out
    }
}

#[derive(Debug)]
pub struct Ad {
    attack: f64,
    decay: f64,
    time_manager: Rc<RefCell<time::TimeManager>>,
}

impl Ad {
    pub fn new(attack: f64, decay: f64) -> Self {
        Ad {
            attack,
            decay,
            time_manager: Rc::new(RefCell::new(time::TimeManager::default())),
        }
    }

    pub fn default() -> Self {
        Self {
            attack: 0.2,
            decay: 0.8,
            time_manager: Rc::new(RefCell::new(time::TimeManager::default())),
        }
    }
}

impl time::TimeKeeper for Ad {
    fn set_time_manager(&mut self, time_manager: Rc<RefCell<time::TimeManager>>) {
        self.time_manager = Rc::clone(&time_manager)
    }
}

impl super::CtrlFunction for Ad {
    fn get_value(&self, time: time::TimeStamp) -> f64 {
        todo!()
    }

    fn get_vec(&self, start: time::TimeStamp, samples: usize) -> Vec<f64> {
        todo!()
    }

    fn trigger(&self, samples: usize) -> Vec<f64> {
        todo!()
    }
}

impl Deref for Ad {
    type Target = dyn CtrlFunction;

    fn deref(&self) -> &Self::Target {
        self
    }
}

impl Envelope for Ad {
    fn get_envelope(&self, sus_samples: usize) -> Vec<f64> {
        let mut out =
            Vec::with_capacity(seconds_to_samples(self.attack) + seconds_to_samples(self.decay));
        for i in 0..seconds_to_samples(self.attack) {
            out.push((i as f64) / (self.attack * SAMPLE_RATE as f64))
        }
        for i in 0..seconds_to_samples(self.decay) {
            out.push(1.0 - (i as f64) / (self.decay * SAMPLE_RATE as f64))
        }
        out
    }
}

#[derive(Debug)]
pub struct Adsr {
    attack: f64,
    decay: f64,
    sustain: f64,
    release: f64,
    time_manager: Rc<RefCell<time::TimeManager>>,
}

impl Adsr {
    pub fn new(attack: f64, decay: f64, sustain: f64, release: f64) -> Self {
        Adsr {
            attack,
            decay,
            sustain,
            release,
            time_manager: Rc::new(RefCell::new(time::TimeManager::default())),
        }
    }
    pub fn default() -> Self {
        Self {
            attack: 0.1,
            decay: 0.2,
            sustain: 0.7,
            release: 0.8,
            time_manager: Rc::new(RefCell::new(time::TimeManager::default())),
        }
    }
}

impl time::TimeKeeper for Adsr {
    fn set_time_manager(&mut self, time_manager: Rc<RefCell<time::TimeManager>>) {
        self.time_manager = Rc::clone(&time_manager)
    }
}

impl super::CtrlFunction for Adsr {
    fn get_value(&self, time: time::TimeStamp) -> f64 {
        todo!()
    }

    fn get_vec(&self, start: time::TimeStamp, samples: usize) -> Vec<f64> {
        todo!()
    }

    fn trigger(&self, samples: usize) -> Vec<f64> {
        todo!()
    }
}

impl Deref for Adsr {
    type Target = dyn CtrlFunction;

    fn deref(&self) -> &Self::Target {
        self
    }
}

impl Envelope for Adsr {
    fn get_envelope(&self, sus_samples: usize) -> Vec<f64> {
        let mut out = Vec::with_capacity(sus_samples + seconds_to_samples(self.release));
        for i in 0..seconds_to_samples(self.attack) {
            out.push((i as f64) / (self.attack * SAMPLE_RATE as f64))
        }
        for i in 0..seconds_to_samples(self.decay) {
            out.push(
                (1.0 - (i as f64) / (self.decay * SAMPLE_RATE as f64)) * (1.0 - self.sustain)
                    + self.sustain,
            )
        }
        if (seconds_to_samples(self.attack) + seconds_to_samples(self.decay)) < sus_samples {
            let mut sus =
                vec![
                    self.sustain;
                    sus_samples - seconds_to_samples(self.attack) - seconds_to_samples(self.decay)
                ];
            out.append(&mut sus);
        }
        for i in 0..seconds_to_samples(self.release) {
            out.push((1.0 - (i as f64) / (self.release * SAMPLE_RATE as f64)) * self.sustain)
        }
        out
    }
}

#[derive(Debug)]
pub struct AdsrDecayed {
    attack: f64,
    decay: f64,
    sustain: f64,
    sus_decay: f64,
    release: f64,
    time_manager: Rc<RefCell<time::TimeManager>>,
}

impl AdsrDecayed {
    pub fn new(attack: f64, decay: f64, sustain: f64, sus_decay: f64, release: f64) -> Self {
        Self {
            attack,
            decay,
            sustain,
            sus_decay,
            release,
            time_manager: Rc::new(RefCell::new(time::TimeManager::default())),
        }
    }
    pub fn default() -> Self {
        Self {
            attack: 0.1,
            decay: 0.2,
            sustain: 0.7,
            sus_decay: 0.2,
            release: 0.4,
            time_manager: Rc::new(RefCell::new(time::TimeManager::default())),
        }
    }
}

impl time::TimeKeeper for AdsrDecayed {
    fn set_time_manager(&mut self, time_manager: Rc<RefCell<time::TimeManager>>) {
        self.time_manager = Rc::clone(&time_manager)
    }
}

impl super::CtrlFunction for AdsrDecayed {
    fn get_value(&self, time: time::TimeStamp) -> f64 {
        todo!()
    }

    fn get_vec(&self, start: time::TimeStamp, samples: usize) -> Vec<f64> {
        todo!()
    }

    fn trigger(&self, samples: usize) -> Vec<f64> {
        todo!()
    }
}



impl Envelope for AdsrDecayed {
    fn get_envelope(&self, sus_samples: usize) -> Vec<f64> {
        let mut out = Vec::with_capacity(sus_samples + seconds_to_samples(self.release));
        for i in 0..seconds_to_samples(self.attack) {
            out.push((i as f64) / (self.attack * SAMPLE_RATE as f64))
        }
        for i in 0..seconds_to_samples(self.decay) {
            out.push(
                (1.0 - (i as f64) / (self.decay * SAMPLE_RATE as f64)) * (1.0 - self.sustain)
                    + self.sustain,
            )
        }
        if (seconds_to_samples(self.attack) + seconds_to_samples(self.decay)) < sus_samples {
            for i in 0..sus_samples {
                out.push(self.sustain * (self.sus_decay * (i as f64) / (SAMPLE_RATE as f64)));
            }
        }
        let last_sustain = *out.last().expect("error while calculating envelope");
        for i in 0..seconds_to_samples(self.release) {
            out.push((1.0 - (i as f64) / (self.release * SAMPLE_RATE as f64)) * last_sustain)
        }
        out
    }
}
