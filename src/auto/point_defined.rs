use crate::{time, utils};
use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Clone, Copy)]
pub struct AutomationPoint {
    value: super::CtrlVal,
    time: time::TimeStamp,
}

impl AutomationPoint {
    pub fn new(value: super::CtrlVal, time: time::TimeStamp) -> Self {
        assert!(
            (0.0..=1.0).contains(&value),
            "the value of an AutomationPoint has to in [0.0, 1.0] (closed interval)"
        );
        Self { value, time }
    }
    pub fn get_value(&self) -> super::CtrlVal {
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

#[derive(Debug)]
pub enum Interpolation {
    Linear,
    Smooth,
}

impl Interpolation {
    pub fn interpolate(
        &self,
        val1: super::CtrlVal,
        val2: super::CtrlVal,
        progress: super::CtrlVal,
    ) -> super::CtrlVal {
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
    time_manager: Rc<RefCell<time::TimeManager>>,
}

impl PointDefined {
    pub fn new(mut points: Vec<AutomationPoint>, interpolation: Interpolation) -> Self {
        points.sort();
        points.dedup_by_key(|x| x.get_time());
        Self {
            points,
            interpolation,
            time_manager: Rc::new(RefCell::new(time::TimeManager::default())),
        }
    }

    fn find_around(
        &self,
        time: time::TimeStamp,
    ) -> (super::CtrlVal, super::CtrlVal, super::CtrlVal) {
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

    pub fn one_point(val: super::CtrlVal, time_manager: Rc<RefCell<time::TimeManager>>) -> Self {
        Self {
            points: vec![AutomationPoint::new(val, time::TimeStamp::zero())],
            interpolation: Interpolation::Linear,
            time_manager,
        }
    }
}

impl time::TimeKeeper for PointDefined {
    fn set_time_manager(&mut self, time_manager: Rc<RefCell<time::TimeManager>>) {
        self.time_manager = Rc::clone(&time_manager)
    }
}

impl super::CtrlFunction for PointDefined {
    fn get_value(&self, time: time::TimeStamp) -> super::CtrlVal {
        let (val1, val2, progress) = self.find_around(time);
        self.interpolation.interpolate(val1, val2, progress)
    }

    fn get_vec(&self, onset: time::TimeStamp, samples: usize) -> Vec<super::CtrlVal> {
        let time_stamps = self.time_manager.borrow().get_stamp_vec(onset, samples);
        let mut out = Vec::with_capacity(samples);
        for t in time_stamps {
            out.push(self.get_value(t))
        }
        out
    }

    fn trigger(&self, samples: usize) -> Vec<super::CtrlVal> {
        self.get_vec(self.time_manager.borrow().zero(), samples)
    }
}
