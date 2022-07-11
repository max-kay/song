use std::rc::Rc;

use crate::consts::SAMPLE_RATE;
use crate::time::{self, TimeKeeper, TimeManager};
use crate::utils::seconds_to_samples;

pub struct Decay {
    decay: f64,
    time_manager: Rc<TimeManager>,
}

impl Decay {
    pub fn new(decay: f64) -> Self {
        Self {
            decay,
            time_manager: Rc::new(time::TimeManager::default()),
        }
    }
    pub fn default() -> Self {
        todo!()
    }
}

impl time::TimeKeeper for Decay {
    fn set_time_manager(&mut self, time_manager: &Rc<TimeManager>) {
        self.time_manager = Rc::clone(time_manager)
    }
}

impl Envelope for Decay {
    fn get_envelope(&self, sus_samples: usize) -> Vec<f64> {
        let mut out = Vec::with_capacity(seconds_to_samples(self.decay));
        for i in 0..seconds_to_samples(self.decay) {
            out.push(1.0 - (i as f64) / (self.decay * (SAMPLE_RATE as f64)))
        }
        out
    }

    fn get_time_manager(&self) -> &Rc<TimeManager> {
        &self.time_manager
    }
}

pub struct Ad {
    attack: f64,
    decay: f64,
    time_manager: Rc<TimeManager>,
}

impl Ad {
    pub fn new(attack: f64, decay: f64) -> Self {
        Ad {
            attack,
            decay,
            time_manager: Rc::new(time::TimeManager::default()),
        }
    }

    pub fn default() -> Self {
        todo!()
    }
}

impl time::TimeKeeper for Ad {
    fn set_time_manager(&mut self, time_manager: &Rc<TimeManager>) {
        self.time_manager = Rc::clone(time_manager)
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

    fn get_time_manager(&self) -> &Rc<TimeManager> {
        &self.time_manager
    }
}

pub struct Adsr {
    attack: f64,
    decay: f64,
    sustain: f64,
    release: f64,
    time_manager: Rc<TimeManager>,
}

impl Adsr {
    pub fn new(attack: f64, decay: f64, sustain: f64, release: f64) -> Self {
        Adsr {
            attack,
            decay,
            sustain,
            release,
            time_manager: Rc::new(time::TimeManager::default()),
        }
    }
    pub fn default() -> Self {
        todo!()
    }
}

impl time::TimeKeeper for Adsr {
    fn set_time_manager(&mut self, time_manager: &Rc<TimeManager>) {
        self.time_manager = Rc::clone(time_manager)
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

    fn get_time_manager(&self) -> &Rc<TimeManager> {
        &self.time_manager
    }
}

pub struct AdsrDecayed {
    attack: f64,
    decay: f64,
    sustain: f64,
    sus_decay: f64,
    release: f64,
    time_manager: Rc<TimeManager>,
}

impl AdsrDecayed {
    pub fn new(attack: f64, decay: f64, sustain: f64, sus_decay: f64, release: f64) -> Self {
        Self {
            attack,
            decay,
            sustain,
            sus_decay,
            release,
            time_manager: Rc::new(time::TimeManager::default()),
        }
    }
    pub fn default() -> Self {
        todo!()
    }
}



impl time::TimeKeeper for AdsrDecayed {
    fn set_time_manager(&mut self, time_manager: &Rc<TimeManager>) {
        self.time_manager = Rc::clone(time_manager)
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

    fn get_time_manager(&self) -> &Rc<TimeManager> {
        &self.time_manager
    }
}

pub trait Envelope: TimeKeeper {
    fn get_envelope(&self, sus_samples: usize) -> Vec<f64>;
    fn get_time_manager(&self) -> &Rc<TimeManager>;
}

impl time::TimeKeeper for Box<dyn Envelope> {
    fn set_time_manager(&mut self, time_manager: &std::rc::Rc<time::TimeManager>) {
        todo!()
    }
}

impl super::CtrlFunction for Box<dyn Envelope> {
    fn get_value(&self, time: time::TimeStamp) -> super::CtrlVal {
        //TODO better
        let sample = self.get_time_manager().stamp_to_samples(time);
        *self
            .get_vec(time, sample + 1)
            .last()
            .expect("error in envelope ctrlfunction")
    }

    fn get_vec(&self, _start: time::TimeStamp, samples: usize) -> Vec<super::CtrlVal> {
        let mut out = self.get_envelope(samples);
        out.resize(samples, 0.0);
        out
    }

    fn trigger(&self, samples: usize) -> Vec<super::CtrlVal> {
        self.get_envelope(samples)
    }
}
