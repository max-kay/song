use crate::time::{self, TimeKeeper, TimeManager, TimeStamp};
use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::{Debug, Display},
    rc::Rc,
    vec,
};

pub mod composed;
pub mod constant;
pub mod envelope;
pub mod lfo;
pub mod point_defined;

pub use composed::Composed;
pub use constant::Constant;
pub use envelope::Envelope;
pub use lfo::Lfo;
pub use point_defined::PointDefined;

pub fn make_ctrl_function<'a, T>(
    ctrl_function: Rc<RefCell<T>>,
) -> Rc<RefCell<dyn CtrlFunction + 'a>>
where
    T: CtrlFunction + 'a,
{
    Rc::clone(&ctrl_function) as Rc<RefCell<dyn CtrlFunction>>
}

pub trait CtrlFunction: TimeKeeper + Debug {
    fn get_value(&self, time: time::TimeStamp) -> f64;
    fn get_vec(&self, start: time::TimeStamp, samples: usize) -> Vec<f64>;
}

#[derive(Debug)]
pub struct Control {
    value: f64,
    range: (f64, f64),
    connection: Option<Rc<RefCell<dyn CtrlFunction>>>,
}

impl Control {
    pub fn new(
        value_in_range: f64,
        range: (f64, f64),
        connection: Rc<RefCell<dyn CtrlFunction>>,
    ) -> Result<Self, ControlError> {
        let new_value = (value_in_range - range.0) / (range.1 - range.0);
        if !(0.0..=1.0).contains(&new_value) {
            return Err(ControlError::new(value_in_range, range));
        }
        Ok(Self {
            value: new_value,
            range,
            connection: Some(Rc::clone(&connection)),
        })
    }

    pub fn from_val_in_unit(value: f64) -> Result<Self, ControlError> {
        Self::from_val_in_range(value, (0.0, 1.0))
    }

    pub fn from_val_in_range(value: f64, range: (f64, f64)) -> Result<Self, ControlError> {
        let new_value = (value - range.0) / (range.1 - range.0);
        if !(0.0..=1.0).contains(&new_value) {
            Err(ControlError::new(value, range))
        } else {
            Ok(Self {
                value: new_value,
                range,
                connection: None,
            })
        }
    }

    #[inline(always)]
    fn put_in_range(&self, value: f64) -> f64 {
        (self.range.1 - self.range.0) * value + self.range.0
    }
}

impl Control {
    pub fn set_connection(&mut self, connection: Rc<RefCell<dyn CtrlFunction>>) {
        self.connection = Some(connection);
    }

    pub fn loose_connection(&mut self) {
        self.connection = None
    }

    pub fn get_value(&self, time: time::TimeStamp) -> f64 {
        let val: f64 = match &self.connection {
            Some(time_function) => time_function.borrow().get_value(time),
            None => self.value,
        };
        self.put_in_range(val)
    }

    pub fn get_vec(&self, start: TimeStamp, samples: usize) -> Vec<f64> {
        match &self.connection {
            Some(time_function) => time_function
                .borrow()
                .get_vec(start, samples)
                .into_iter()
                .map(|x| self.put_in_range(x))
                .collect(),
            None => vec![self.put_in_range(self.value); samples],
        }
    }

    pub fn set_value(&mut self, value: f64) -> Result<(), ControlError> {
        let new_value = (value - self.range.0) / (self.range.1 - self.range.0);
        if !(0.0..=1.0).contains(&new_value) {
            Err(ControlError::new(value, self.range))
        } else {
            self.value = value;
            Ok(())
        }
    }

    pub fn get_range(&self) -> (f64, f64) {
        self.range
    }

    pub fn set_range(&mut self, range: (f64, f64)) {
        self.range = range;
    }
}

impl Default for Control {
    fn default() -> Self {
        Self {
            value: 0.5,
            range: (0.0, 1.0),
            connection: None,
        }
    }
}

impl TimeKeeper for Control {
    fn set_time_manager(&mut self, time_manager: Rc<RefCell<TimeManager>>) {
        if let Some(time_function) = &self.connection {
            time_function.borrow_mut().set_time_manager(time_manager)
        }
    }
}

#[derive(Debug)]
pub struct ControlError {
    path: Vec<String>,
    origin: String,
    control: String,
    value: f64,
    range: (f64, f64),
}

impl ControlError {
    pub fn new(value: f64, range: (f64, f64)) -> Self {
        Self {
            path: Vec::new(),
            origin: String::new(),
            control: String::new(),
            value,
            range,
        }
    }

    pub fn set_origin(mut self, origin: &str, control: &str) -> Self {
        self.origin.push_str(origin);
        self.control.push_str(control);
        self
    }

    pub fn push_location(mut self, location: &str) -> Self {
        self.path.push(location.to_string());
        self
    }
}

impl Display for ControlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "The value of {} in {} was set to {}, which is not in range from {} to {}!\n    full path to value: {}/{}/{}",
            self.control,
            self.origin,
            self.value,
            self.range.0,
            self.range.1,
            self.path.join("/"),
            self.origin,
            self.control)
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
    fn set_time_manager(&mut self, time_manager: Rc<RefCell<TimeManager>>) {
        for control in self.channels.values_mut() {
            control
                .borrow_mut()
                .set_time_manager(Rc::clone(&time_manager))
        }
    }
}
