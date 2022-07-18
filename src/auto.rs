use crate::time::{self, TimeKeeper, TimeManager, TimeStamp};
use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::{Debug, Display},
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
    vec,
};

pub mod composed;
pub mod constant;
pub mod envelope;
pub mod lfo;
pub mod point_defined;

// pub use composed::Composed;
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

fn get_ctrl_id() -> usize {
    static COUNTER: AtomicUsize = AtomicUsize::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

pub trait CtrlFunction: TimeKeeper + Debug {
    fn get_value(&self, time: time::TimeStamp) -> f64;
    fn get_vec(&self, start: time::TimeStamp, samples: usize) -> Vec<f64>;
    fn get_id(&self) -> usize;
    fn get_sub_ids(&self) -> Vec<usize>;
    fn get_all_ids(&self) -> Vec<usize> {
        let mut out = self.get_sub_ids();
        out.push(self.get_id());
        out
    }
}

#[derive(Debug)]
pub struct Control {
    value: f64,
    range: (f64, f64),
    connection: Option<Rc<RefCell<dyn CtrlFunction>>>,
    id: Option<usize>,
}

impl Control {
    pub fn new(
        value_in_range: f64,
        range: (f64, f64),
        connection: Rc<RefCell<dyn CtrlFunction>>,
    ) -> Result<Self, ControlError> {
        let new_value = (value_in_range - range.0) / (range.1 - range.0);
        if !(0.0..=1.0).contains(&new_value) {
            return Err(ControlError::new_range_err(value_in_range, range));
        }
        Ok(Self {
            value: new_value,
            range,
            connection: Some(Rc::clone(&connection)),
            id: Some(connection.borrow().get_id()),
        })
    }

    pub fn from_val_in_unit(value: f64) -> Result<Self, ControlError> {
        Self::from_val_in_range(value, (0.0, 1.0))
    }

    pub fn from_val_in_range(value: f64, range: (f64, f64)) -> Result<Self, ControlError> {
        let new_value = (value - range.0) / (range.1 - range.0);
        if !(0.0..=1.0).contains(&new_value) {
            Err(ControlError::new_range_err(value, range))
        } else {
            Ok(Self {
                value: new_value,
                range,
                connection: None,
                id: None,
            })
        }
    }

    #[inline(always)]
    fn put_in_range(&self, value: f64) -> f64 {
        (self.range.1 - self.range.0) * value + self.range.0
    }
}

impl Control {
    pub fn try_set(&mut self, other: Self) -> Result<(), ControlError>{
        self.cmp_ranges(other.range)?;
        if let Some(connection) = other.connection{
            self.try_set_connection(connection)?
        }
        self.value = other.value;
        Ok(())
    }

    pub fn try_set_connection(
        &mut self,
        connection: Rc<RefCell<dyn CtrlFunction>>,
    ) -> Result<(), ControlError> {
        if let Some(id) = self.id {
            if connection.borrow().get_all_ids().contains(&id) {
                return Err(ControlError::new_circ_ref_err(id));
            }
        }
        self.connection = Some(Rc::clone(&connection));
        self.id = Some(connection.borrow().get_id());
        Ok(())
    }

    pub fn cmp_ranges(&self, range: (f64, f64)) -> Result<(), ControlError>{
        if self.range == range{
            Ok(())
        } else {
            Err(ControlError::new_range_mismatch_err(self.range, range))
        }
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
            Err(ControlError::new_range_err(value, self.range))
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

impl Control {
    pub fn get_ids(&self) -> Vec<usize> {
        match &self.connection {
            Some(ctrl_func) => ctrl_func.borrow().get_all_ids(),
            None => Vec::new(),
        }
    }

    pub fn get_ctrl_id(&self) -> Option<usize> {
        self.id
    }
}

impl Default for Control {
    fn default() -> Self {
        Self {
            value: 0.5,
            range: (0.0, 1.0),
            connection: None,
            id: None,
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
    kind: ErrorKind,
}

#[derive(Debug)]
enum ErrorKind {
    Range {
        value: f64,
        range: (f64, f64),
    },
    RangeMismatch {
        trg_range: (f64, f64),
        src_range: (f64, f64),
    },
    CircRef {
        id: usize,
    },
}

impl ControlError {
    pub fn new_range_err(value: f64, range: (f64, f64)) -> Self {
        Self {
            path: Vec::new(),
            origin: String::new(),
            control: String::new(),
            kind: ErrorKind::Range { value, range },
        }
    }

    pub fn new_range_mismatch_err(trg_range: (f64, f64), src_range: (f64, f64)) -> Self {
        Self {
            path: Vec::new(),
            origin: String::new(),
            control: String::new(),
            kind: ErrorKind::RangeMismatch { trg_range, src_range },
        }
    }

    pub fn new_circ_ref_err(id: usize) -> Self {
        Self {
            path: Vec::new(),
            origin: String::new(),
            control: String::new(),
            kind: ErrorKind::CircRef { id },
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

    fn get_string(&self) -> String {
        match &self.kind {
            ErrorKind::Range { value, range } => {
                format!(
                    "The value of {} in {} was set to {}, which is not in range from {} to {}!\n    full path to value: {}/{}/{}/{}",
                    self.control,
                    self.origin,
                    value,
                    range.0,
                    range.1,
                    self.path.join("/"),
                    self.origin,
                    self.control,
                    value,
                )
            }
            ErrorKind::CircRef { id } => {
                format!(
                    "You tried to set {} in {} to {} this leeds to a circular Reference between CtrlFunctions, which is not allowed!\n    full path to control: {}/{}/{}",
                    self.control,
                    self.origin,
                    id,
                    self.path.join("/"),
                    self.origin,
                    self.control,
                )
            }
            ErrorKind::RangeMismatch { trg_range, src_range } => {
                format!(
                    "You tried to set {} in {} which has a range of ({}, {}) to a control with a range of ({}, {})!\n    full path to control: {}/{}/{}",
                    self.control,
                    self.origin,
                    trg_range.0,
                    trg_range.1,
                    src_range.0,
                    src_range.1,
                    self.path.join("/"),
                    self.origin,
                    self.control,
                )
            },
        }
    }
}

impl Display for ControlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_string())
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
