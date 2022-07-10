use crate::consts::SAMPLE_RATE;
use crate::time;
use crate::time::TimeManager;
use crate::utils;
use crate::utils::oscs;
use fixed;
use std::collections::HashMap;
use std::f64::consts::TAU;
use std::rc::Rc;
use std::vec;

pub type CtrlVal = fixed::types::I0F16;

pub trait TimeFunction: time::TimeKeeper {
    fn get_value(&self, time: time::TimeStamp) -> CtrlVal;
    fn get_vec(&self, start: time::TimeStamp, samples: usize) -> Vec<CtrlVal>;
    fn trigger(&self, samples: usize) -> Vec<CtrlVal>;
}

pub struct Control {
    pub value: CtrlVal,
    pub prescalar: f64,
    pub connection: Option<Rc<Box<dyn TimeFunction>>>,
}

impl Control {
    pub fn from_values(value: CtrlVal, prescalar: f64) -> Self {
        Self {
            value,
            prescalar,
            connection: None,
        }
    }

    pub fn get_value(&self, time: time::TimeStamp) -> f64 {
        let val: f64 = match &self.connection {
            Some(time_function) => time_function.get_value(time),
            None => self.value,
        }
        .to_num();
        val * self.prescalar
    }

    pub fn get_vec(&self, time: time::TimeStamp, samples: usize) -> Vec<f64> {
        match &self.connection {
            Some(time_function) => time_function
                .get_vec(time, samples)
                .into_iter()
                .map(|x| x.to_num::<f64>() * self.prescalar)
                .collect(),
            None => vec![self.value.to_num::<f64>() * self.prescalar; samples],
        }
    }
}

impl time::TimeKeeper for Control {
    fn set_time_manager(&mut self, time_manager: &Rc<TimeManager>) {
        todo!()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AutomationPoint {
    value: CtrlVal,
    time: time::TimeStamp,
}

impl AutomationPoint {
    pub fn new(value: CtrlVal, time: time::TimeStamp) -> Self {
        assert!(
            (0.0..=1.0).contains(&value),
            "the value of an AutomationPoint has to in [0.0, 1.0] (closed interval)"
        );
        Self { value, time }
    }
    pub fn get_value(&self) -> CtrlVal {
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
    pub fn interpolate(&self, val1: CtrlVal, val2: CtrlVal, progress: CtrlVal) -> CtrlVal {
        match self {
            Interpolation::Linear => (val2 - val1) * progress + val1,
            Interpolation::Smooth => (val2 - val1) * utils::smooth_step(progress) + val1,
        }
    }
}

pub struct PointDefined {
    points: Vec<AutomationPoint>,
    interpolation: Interpolation,
    time_manager: Rc<time::TimeManager>,
}

impl PointDefined {
    pub fn new(mut points: Vec<AutomationPoint>, interpolation: Interpolation) -> Self {
        points.sort();
        points.dedup_by_key(|x| x.get_time());
        Self {
            points,
            interpolation,
            time_manager: Rc::new(TimeManager::default()),
        }
    }

    fn find_around(&self, time: time::TimeStamp) -> (CtrlVal, CtrlVal, CtrlVal) {
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
            .duration_to_seconds(p1.get_time(), p2.get_time());
        let part_secs = self.time_manager.duration_to_seconds(p1.get_time(), time);
        (val1, val2, CtrlVal::from_num(part_secs / tot_secs))
    }

    pub fn one_point(val: CtrlVal, time_manager: Rc<time::TimeManager>) -> Self {
        Self {
            points: vec![AutomationPoint::new(val, time::TimeStamp::zero())],
            interpolation: Interpolation::Linear,
            time_manager,
        }
    }
}

impl time::TimeKeeper for PointDefined {
    fn set_time_manager(&mut self, time_manager: &Rc<time::TimeManager>) {
        self.time_manager = Rc::clone(time_manager)
    }
}

impl TimeFunction for PointDefined {
    fn get_value(&self, time: time::TimeStamp) -> CtrlVal {
        let (val1, val2, progress) = self.find_around(time);
        self.interpolation.interpolate(val1, val2, progress)
    }

    fn get_vec(&self, onset: time::TimeStamp, samples: usize) -> Vec<CtrlVal> {
        let time_stamps = self.time_manager.get_stamp_vec(onset, samples);
        let mut out = Vec::with_capacity(samples);
        for t in time_stamps {
            out.push(self.get_value(t))
        }
        out
    }

    fn trigger(&self, samples: usize) -> Vec<CtrlVal> {
        self.get_vec(self.time_manager.zero(), samples)
    }
}

pub struct Lfo {
    oscillator: Box<dyn oscs::Oscillator>,
    freq: Control,
    modulation: Control,
    phase_shift: f64,
    time_manager: Rc<time::TimeManager>,
}

impl Lfo {
    pub fn new() -> Self {
        Self {
            oscillator: Box::new(oscs::ModSaw::new(1.0)),
            freq: Control::from_values(CtrlVal::from_num(1_f64), 1.0),
            modulation: Control::from_values(CtrlVal::from_num(1_f64), 1.0),
            phase_shift: 0.0,
            time_manager: Rc::new(time::TimeManager::default()),
        }
    }
}

impl time::TimeKeeper for Lfo {
    fn set_time_manager(&mut self, time_manager: &Rc<time::TimeManager>) {
        self.time_manager = Rc::clone(time_manager)
    }
}

impl Default for Lfo {
    fn default() -> Self {
        Self::new()
    }
}

impl TimeFunction for Lfo {
    fn get_value(&self, time: time::TimeStamp) -> CtrlVal {
        let phase = ((self.time_manager.stamp_to_seconds(time) * TAU * self.freq.get_value(time)
            / (SAMPLE_RATE as f64))
            + self.phase_shift)
            % TAU;
        CtrlVal::from_num(
            self.oscillator
                .get_sample(phase, self.modulation.get_value(time)),
        )
    }

    fn get_vec(&self, start: time::TimeStamp, samples: usize) -> Vec<CtrlVal> {
        self.oscillator
            .play_shifted(
                &self.freq.get_vec(start, samples),
                &self.modulation.get_vec(start, samples),
                samples,
                self.phase_shift,
            )
            .into_iter()
            .map(CtrlVal::from_num)
            .collect()
    }

    fn trigger(&self, samples: usize) -> Vec<CtrlVal> {
        self.get_vec(time::TimeStamp::zero(), samples)
    }
}

#[derive(Default)]
pub struct Constant(pub CtrlVal);

impl time::TimeKeeper for Constant {
    fn set_time_manager(&mut self, _time_manager: &Rc<TimeManager>) {}
}

impl TimeFunction for Constant {
    fn get_value(&self, _time: time::TimeStamp) -> CtrlVal {
        self.0
    }

    fn get_vec(&self, _start: time::TimeStamp, samples: usize) -> Vec<CtrlVal> {
        vec![self.0; samples]
    }

    fn trigger(&self, samples: usize) -> Vec<CtrlVal> {
        vec![self.0; samples]
    }
}

pub struct Composed(Vec<Box<dyn TimeFunction>>);

impl time::TimeKeeper for Composed {
    fn set_time_manager(&mut self, time_manager: &Rc<TimeManager>) {
        for time_function in &mut self.0 {
            time_function.set_time_manager(time_manager)
        }
    }
}

impl TimeFunction for Composed {
    fn get_value(&self, time: time::TimeStamp) -> CtrlVal {
        let mut val = CtrlVal::from_num(1_f32);
        for time_function in &self.0 {
            val *= time_function.get_value(time)
        }
        val
    }

    fn get_vec(&self, start: time::TimeStamp, samples: usize) -> Vec<CtrlVal> {
        let mut vec = vec![CtrlVal::from_num(1_f32); samples];
        for time_function in &self.0 {
            vec = vec
                .into_iter()
                .zip(time_function.get_vec(start, samples).into_iter())
                .map(|(x1, x2)| x1 * x2)
                .collect();
        }
        vec
    }

    fn trigger(&self, samples: usize) -> Vec<CtrlVal> {
        self.get_vec(time::TimeStamp::zero(), samples)
    }
}

pub struct AutomationManager {
    channels: HashMap<u8, Rc<Box<dyn TimeFunction>>>,
}

impl AutomationManager {
    pub fn new() -> Self {
        Self {
            channels: HashMap::new(),
        }
    }

    pub fn all_channels(&self) -> Vec<u8> {
        self.channels.keys().into_iter().copied().collect()
    }

    pub fn get_channel(&self, channel: u8) -> Option<Rc<Box<dyn TimeFunction>>> {
        self.channels.get(&channel).map(Rc::clone)
    }
}

impl Default for AutomationManager {
    fn default() -> Self {
        Self::new()
    }
}
