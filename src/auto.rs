use std::{collections::HashMap, rc::Rc};

use crate::time::TimeStamp;

pub trait TimeFunction {
    fn get_value(&self, time: TimeStamp) -> f64;
    fn get_vec(&self, start: TimeStamp, samples: usize) -> Vec<f64>;
}

pub struct ValAndCh {
    pub value: f64,
    pub connection: Option<(Rc<dyn TimeFunction>, f64)>, // the f64 here is the prescalar
}

impl ValAndCh {
    pub fn get_value(&self, time: TimeStamp) -> f64 {
        match &self.connection {
            Some((time_function, prescalar)) => time_function.get_value(time) * prescalar,
            None => self.value,
        }
    }

    pub fn get_vec(&self, time: TimeStamp, samples: usize) -> Vec<f64> {
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

pub struct AutomationPoint {
    value: f64,
    time: TimeStamp,
}

impl AutomationPoint {
    pub fn new(value: f64, time: TimeStamp) -> Self {
        assert!(
            (0.0..=1.0).contains(&value),
            "the value of an AutomationPoint has to in [0.0, 1.0] (closed interval)"
        );
        Self { value, time }
    }
    pub fn get_value(&self) -> f64 {
        self.value
    }
    pub fn get_time(&self) -> TimeStamp {
        self.time
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
        todo!()
    }
}

pub struct PointDefined {
    points: Vec<AutomationPoint>,
    interpolation: Interpolation,
}

impl PointDefined {
    pub fn new(mut points: Vec<AutomationPoint>, interpolation: Interpolation) -> Self {
        points.sort();
        points.dedup_by_key(|x| x.get_time());
        Self {
            points,
            interpolation,
        }
    }
    fn find_around(&self, time: TimeStamp) -> (f64, f64, f64) {
        todo!()
    }

    pub fn one_point(val: f64) -> Self {
        Self {
            points: vec![AutomationPoint::new(val, TimeStamp::zero())],
            interpolation: Interpolation::Linear,
        }
    }
}

impl TimeFunction for PointDefined {
    fn get_value(&self, time: TimeStamp) -> f64 {
        let (val1, val2, progress) = self.find_around(time);
        self.interpolation.interpolate(val1, val2, progress)
    }

    fn get_vec(&self, onset: TimeStamp, samples: usize) -> Vec<f64> {
        todo!()
    }
}

pub enum Lfo {
    Sine {
        freq: ValAndCh,
    },
    ModSquare {
        freq: ValAndCh,
        modulation: ValAndCh,
    },
    ModSaw {
        freq: ValAndCh,
        modulation: ValAndCh,
    },
    Explicit {
        values: Vec<f64>,
    },
}

impl Lfo {
    pub fn default_lfo() -> Self {
        todo!()
    }
}

impl TimeFunction for Lfo {
    fn get_value(&self, time: TimeStamp) -> f64 {
        todo!()
    }

    fn get_vec(&self, time: TimeStamp, samples: usize) -> Vec<f64> {
        todo!()
    }
}

