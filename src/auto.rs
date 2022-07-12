use crate::time::{self, TimeKeeper};
use std::{cell::RefCell, collections::HashMap, rc::Rc, vec, fmt::Debug};

pub mod composed;
pub mod constant;
pub mod envelope;
pub mod lfo;
pub mod point_defined;

pub use composed::Composed;
pub use constant::Constant;
pub use envelope::{Ad, Adsr, AdsrDecayed, Decay, Envelope};
pub use lfo::Lfo;
pub use point_defined::PointDefined;




pub trait CtrlFunction: TimeKeeper + Debug {
    fn get_value(&self, time: time::TimeStamp) -> f64;
    fn get_vec(&self, start: time::TimeStamp, samples: usize) -> Vec<f64>;
    fn trigger(&self, samples: usize) -> Vec<f64>;
}

#[derive(Debug)]
pub struct Control {
    value: f64,
    prescalar: f64,
    connection: Option<Rc<RefCell<dyn CtrlFunction>>>,
}

impl Control {
    pub fn new(value: f64, prescalar: f64, connection: Rc<RefCell<dyn CtrlFunction>>) -> Self {
        Self {
            value,
            prescalar,
            connection: Some(Rc::clone(&connection)),
        }
    }

    pub fn from_values(value: f64, prescalar: f64) -> Self {
        Self {
            value,
            prescalar,
            connection: None,
        }
    }

    pub fn get_value(&self, time: time::TimeStamp) -> f64 {
        let val: f64 = match &self.connection {
            Some(time_function) => time_function.borrow().get_value(time),
            None => self.value,
        };
        val * self.prescalar
    }

    pub fn get_vec(&self, time: time::TimeStamp, samples: usize) -> Vec<f64> {
        match &self.connection {
            Some(time_function) => time_function
                .borrow()
                .get_vec(time, samples)
                .into_iter()
                .map(|x| x * self.prescalar)
                .collect(),
            None => vec![self.value * self.prescalar; samples],
        }
    }
}

impl time::TimeKeeper for Control {
    fn set_time_manager(&mut self, time_manager: Rc<RefCell<time::TimeManager>>) {
        if let Some(time_function) = &self.connection {
            time_function.borrow_mut().set_time_manager(time_manager)
        }
    }
}

pub trait AutomationKeeper {
    fn set_automation_manager(&mut self, automation_manager: Rc<RefCell<AutomationManager>>);
}

#[derive(Debug)]
pub struct AutomationManager {
    channels: HashMap<u8, Rc<RefCell<dyn CtrlFunction>>>,
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

    pub fn get_channel(&self, channel: u8) -> Option<Rc<RefCell<dyn CtrlFunction>>> {
        self.channels.get(&channel).map(Rc::clone)
    }
}

impl Default for AutomationManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TimeKeeper for AutomationManager {
    fn set_time_manager(&mut self, time_manager: Rc<RefCell<time::TimeManager>>) {
        for control in self.channels.values_mut() {
            control
                .borrow_mut()
                .set_time_manager(Rc::clone(&time_manager))
        }
    }
}
