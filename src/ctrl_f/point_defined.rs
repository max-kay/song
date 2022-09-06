use serde::{Deserialize, Serialize};

use crate::{globals::TIME_MANAGER, time::TimeStamp, utils, Error};
use std::cmp::Ordering;

use super::{GenId, Generator};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AutomationPoint {
    value: f64,
    time: TimeStamp,
}

impl AutomationPoint {
    pub fn new(value: f64, time: TimeStamp) -> Result<Self, Error> {
        if !(0.0..=1.0).contains(&value) {
            return Err(Error::Value);
        }
        Ok(Self { value, time })
    }
    pub fn get_val(&self) -> f64 {
        self.value
    }
    pub fn get_time(&self) -> TimeStamp {
        self.time
    }
}

impl Default for AutomationPoint {
    fn default() -> Self {
        Self {
            value: 0.0,
            time: TimeStamp::zero(),
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
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.time.partial_cmp(&other.time) {
            Some(Ordering::Equal) => {}
            ord => return ord,
        }
        self.value.partial_cmp(&other.value)
    }
}

impl Ord for AutomationPoint {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other)
            .expect("error in Ord of AutomationPoint")
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub enum Interpolation {
    #[default]
    Step,
    Linear,
    Smooth,
}

impl Interpolation {
    pub fn interpolate(&self, val1: f64, val2: f64, progress: f64) -> f64 {
        match self {
            Interpolation::Step => val1,
            Interpolation::Linear => (val2 - val1) * progress + val1,
            Interpolation::Smooth => (val2 - val1) * utils::smooth_step(progress) + val1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointDefined {
    id: GenId,
    points: Vec<AutomationPoint>,
    interpolation: Interpolation,
}

impl PointDefined {
    pub fn new(mut points: Vec<AutomationPoint>, interpolation: Interpolation) -> Self {
        points.sort();
        points.dedup_by_key(|x| x.get_time());
        Self {
            id: GenId::Unbound,
            points,
            interpolation,
        }
    }

    pub fn w_default() -> Generator {
        Generator::PointDefined(Self::default())
    }

    pub fn new_val(val: f64) -> Result<Self, Error> {
        Ok(Self {
            id: GenId::Unbound,
            points: vec![AutomationPoint::new(
                val,
                TIME_MANAGER.read().unwrap().zero(),
            )?],
            interpolation: Default::default(),
        })
    }

    pub fn w_val(val: f64) -> Result<Generator, Error> {
        Ok(Generator::PointDefined(Self::new_val(val)?))
    }

    pub(crate) fn set_id(&mut self, id: GenId) {
        self.id = id
    }

    fn find_around(&self, time: TimeStamp) -> (f64, f64, f64) {
        let mut p1 = AutomationPoint::default();
        let mut p2 = AutomationPoint::default();
        for (i, p) in self.points.iter().enumerate() {
            if time <= p.get_time() {
                p1 = *p;
                p2 = self.points[i + 1];
                break;
            }
        }
        let val1 = p1.get_val();
        let val2 = p2.get_val();
        let tot_secs = TIME_MANAGER
            .read()
            .unwrap()
            .duration_to_seconds(p1.get_time(), p2.get_time());
        let part_secs = TIME_MANAGER
            .read()
            .unwrap()
            .duration_to_seconds(p1.get_time(), time);
        (val1, val2, part_secs / tot_secs)
    }
}

impl PointDefined {
    pub fn get_val(&self, time: TimeStamp) -> f64 {
        let (val1, val2, progress) = self.find_around(time);
        self.interpolation.interpolate(val1, val2, progress)
    }

    pub fn get_vec(&self, onset: TimeStamp, samples: usize) -> Vec<f64> {
        let time_stamps = TIME_MANAGER.read().unwrap().get_stamp_vec(onset, samples);
        let mut out = Vec::with_capacity(samples);
        for t in time_stamps {
            out.push(self.get_val(t))
        }
        out
    }
}
impl Default for PointDefined {
    fn default() -> Self {
        Self {
            id: GenId::Unbound,
            points: vec![AutomationPoint::default()],
            interpolation: Default::default(),
        }
    }
}
