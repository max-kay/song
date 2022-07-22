use crate::{
    control::ControlError,
    time::{TimeKeeper, TimeManager, TimeStamp},
    utils,
};
use std::{cell::RefCell, cmp::Ordering, rc::Rc};

use super::{CtrlFunction, IdMap, FunctionKeeper};

#[derive(Debug, Clone, Copy)]
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

impl Default for AutomationPoint {
    fn default() -> Self {
        Self {
            value: Default::default(),
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

#[derive(Debug)]
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

#[derive(Debug)]
pub struct PointDefined {
    points: Vec<AutomationPoint>,
    interpolation: Interpolation,
    time_manager: Rc<RefCell<TimeManager>>,
    id: usize,
}

impl PointDefined {
    pub fn new(mut points: Vec<AutomationPoint>, interpolation: Interpolation) -> Self {
        points.sort();
        points.dedup_by_key(|x| x.get_time());
        Self {
            points,
            interpolation,
            time_manager: Rc::new(RefCell::new(TimeManager::default())),
            id: utils::get_f_id(),
        }
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
        let val1 = p1.get_value();
        let val2 = p2.get_value();
        let tot_secs = self
            .time_manager
            .borrow()
            .duration_to_seconds(p1.get_time(), p2.get_time());
        let part_secs = self
            .time_manager
            .borrow()
            .duration_to_seconds(p1.get_time(), time);
        (val1, val2, part_secs / tot_secs)
    }

    pub fn one_point(val: f64) -> Self {
        Self::new(
            vec![AutomationPoint::new(val, TimeStamp::zero())],
            Interpolation::Linear,
        )
    }
}

impl TimeKeeper for PointDefined {
    fn set_time_manager(&mut self, time_manager: Rc<RefCell<TimeManager>>) {
        self.time_manager = Rc::clone(&time_manager)
    }
}

impl FunctionKeeper for PointDefined {
    fn heal_sources(&mut self, _id_map: &IdMap) -> Result<(), ControlError> {
        Ok(())
    }

    fn test_sources(&self) -> Result<(), ControlError> {
        Ok(())
    }

    fn set_ids(&mut self) {}

    fn get_ids(&self) -> Vec<usize> {
        vec![self.get_id()]
    }
}

impl CtrlFunction for PointDefined {
    fn get_value(&self, time: TimeStamp) -> f64 {
        let (val1, val2, progress) = self.find_around(time);
        self.interpolation.interpolate(val1, val2, progress)
    }

    fn get_vec(&self, onset: TimeStamp, samples: usize) -> Vec<f64> {
        let time_stamps = self.time_manager.borrow().get_stamp_vec(onset, samples);
        let mut out = Vec::with_capacity(samples);
        for t in time_stamps {
            out.push(self.get_value(t))
        }
        out
    }

    fn get_id(&self) -> usize {
        self.id
    }

    unsafe fn new_id_f(&mut self) {
        self.id = utils::get_f_id()
    }
}
