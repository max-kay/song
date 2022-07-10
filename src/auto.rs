use crate::time;
use crate::utils;
use std::rc::Rc;

pub trait TimeFunction {
    fn get_value(&self, time: time::TimeStamp) -> f64;
    fn get_vec(&self, start: time::TimeStamp, samples: usize) -> Vec<f64>;
}

pub struct Control {
    pub value: f64,
    pub connection: Option<(Rc<dyn TimeFunction>, f64)>, // the f64 here is the prescalar
}

impl Control {
    pub fn get_value(&self, time: time::TimeStamp) -> f64 {
        match &self.connection {
            Some((time_function, prescalar)) => time_function.get_value(time) * prescalar,
            None => self.value,
        }
    }

    pub fn get_vec(&self, time: time::TimeStamp, samples: usize) -> Vec<f64> {
        match &self.connection {
            Some((time_function, prescalar)) => time_function
                .get_vec(time, samples)
                .into_iter()
                .map(|x| x * prescalar)
                .collect(),
            None => vec![self.value; samples],
        }
    }
}

pub enum ValOrVec {
    Val(f64),
    Vec(Vec<f64>),
}

#[derive(Debug, Clone, Copy)]
pub struct AutomationPoint {
    value: f64,
    time: time::TimeStamp,
}

impl AutomationPoint {
    pub fn new(value: f64, time: time::TimeStamp) -> Self {
        assert!(
            (0.0..=1.0).contains(&value),
            "the value of an AutomationPoint has to in [0.0, 1.0] (closed interval)"
        );
        Self { value, time }
    }
    pub fn get_value(&self) -> f64 {
        self.value
    }
    pub fn get_time(&self) -> time::TimeStamp {
        self.time
    }
}

impl Default for AutomationPoint {
    fn default() -> Self {
        Self {
            value: Default::default(),
            time: time::TimeStamp::zero(),
        }
    }
}

impl PartialEq for AutomationPoint {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.time == other.time
    }
}

impl Eq for AutomationPoint {}

impl PartialOrd for AutomationPoint {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.time.partial_cmp(&other.time) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.value.partial_cmp(&other.value)
    }
}

impl Ord for AutomationPoint {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other)
            .expect("error in Ord of AutomationPoint")
    }
}

pub enum Interpolation {
    Linear,
    Smooth,
}

impl Interpolation {
    pub fn interpolate(&self, val1: f64, val2: f64, progress: f64) -> f64 {
        match self {
            Interpolation::Linear => (val2 - val1) * progress + val1,
            Interpolation::Smooth => (val2 - val1) * utils::smooth_step(progress) + val1,
        }
    }
}

pub struct PointDefined {
    points: Vec<AutomationPoint>,
    interpolation: Interpolation,
    time_keeper: Rc<time::TimeKeeper>,
}

impl PointDefined {
    pub fn new(
        mut points: Vec<AutomationPoint>,
        interpolation: Interpolation,
        time_keeper: Rc<time::TimeKeeper>,
    ) -> Self {
        points.sort();
        points.dedup_by_key(|x| x.get_time());
        Self {
            points,
            interpolation,
            time_keeper,
        }
    }

    fn find_around(&self, time: time::TimeStamp) -> (f64, f64, f64) {
        let mut p1 = AutomationPoint::default();
        let mut p2 = AutomationPoint::default();
        for (i, p) in self.points.iter().enumerate() {
            if time <= p.get_time() {
                p1 = *p;
                p2 = self.points[i + 1];
                break;
            }
        }
        let val1 = p1.get_value();
        let val2 = p2.get_value();
        let tot_secs = self
            .time_keeper
            .duration_to_seconds(p1.get_time(), p2.get_time());
        let part_secs = self.time_keeper.duration_to_seconds(p1.get_time(), time);
        (val1, val2, part_secs / tot_secs)
    }

    pub fn one_point(val: f64, time_keeper: Rc<time::TimeKeeper>) -> Self {
        Self {
            points: vec![AutomationPoint::new(val, time::TimeStamp::zero())],
            interpolation: Interpolation::Linear,
            time_keeper,
        }
    }
}

impl TimeFunction for PointDefined {
    fn get_value(&self, time: time::TimeStamp) -> f64 {
        let (val1, val2, progress) = self.find_around(time);
        self.interpolation.interpolate(val1, val2, progress)
    }

    fn get_vec(&self, onset: time::TimeStamp, samples: usize) -> Vec<f64> {
        let time_stamps = self.time_keeper.get_stamp_vec(onset, samples);
        let mut out = Vec::with_capacity(samples);
        for t in time_stamps {
            out.push(self.get_value(t))
        }
        out
    }
}

pub enum Lfo {
    Sine { freq: Control },
    ModSquare { freq: Control, modulation: Control },
    ModSaw { freq: Control, modulation: Control },
    Explicit { values: Vec<f64> },
}

impl Lfo {
    pub fn default_lfo() -> Self {
        todo!()
    }
}

impl TimeFunction for Lfo {
    fn get_value(&self, time: time::TimeStamp) -> f64 {
        todo!()
    }

    fn get_vec(&self, time: time::TimeStamp, samples: usize) -> Vec<f64> {
        todo!()
    }
}
