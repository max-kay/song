use crate::{ time::TimeStamp, utils};
use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::{Debug, Display},
    rc::Rc,
    vec,
};

pub mod constant;
pub mod envelope;
pub mod lfo;
pub mod phantom;
pub mod point_defined;

pub use constant::Constant;
pub use envelope::Envelope;
pub use lfo::Lfo;
pub use point_defined::PointDefined;

pub trait CtrlFunction: Debug {
    fn get_value(&self, time: TimeStamp) -> f64;
    fn get_vec(&self, start: TimeStamp, samples: usize) -> Vec<f64>;
}


#[derive(Debug)]
pub struct FunctionManager {
    channels: HashMap<u16, Box<dyn CtrlFunction + Send>>,
}

impl Default for FunctionManager {
    fn default() -> Self {
        todo!()
    }
}

#[derive(Debug)]
pub struct FuncId(u8);

impl FuncId {
    pub fn validate(&self) -> bool {
        todo!()
    }
}


#[derive(Debug)]
pub enum Source {
    Function {},
    WeigthedAverage {
        sources: Vec<(f64, Source)>,
    },
    WeightedProduct {
        sources: Vec<(f64, Source)>,
    },
    Inverted {
        source: Box<Source>,
    },
    Transformed {
        func: fn(f64) -> f64,
        source: Box<Source>,
    },
}

impl Source {
    pub fn from_function(function: FuncId) -> Self {
        Self::Function {}
    }
}

impl Source {
    pub fn get_value(&self, time: TimeStamp) -> f64 {
        match self {
            Source::Function {} => todo!(),
            Source::WeigthedAverage { sources } => {
                let mut value = 0.0;
                let mut total_w = 0.0;
                for (w, s) in sources {
                    total_w += w;
                    value += w * s.get_value(time);
                }
                value / total_w
            }
            Source::WeightedProduct { sources } => {
                let mut value = 1.0;
                for (w, s) in sources {
                    value *= s.get_value(time).powf(*w);
                }
                value
            }
            Source::Inverted { source } => -source.get_value(time) + 1.0,
            Source::Transformed { func, source } => func(source.get_value(time)),
        }
    }
    pub fn get_vec(&self, start: TimeStamp, samples: usize) -> Vec<f64> {
        match self {
            Source::Function {} => todo!(),
            Source::WeigthedAverage { sources } => {
                let mut values = vec![0.0; samples];
                let mut total_w = 0.0;
                for (w, s) in sources {
                    total_w += w;
                    utils::mul_elementwise(
                        &mut values,
                        s.get_vec(start, samples)
                            .into_iter()
                            .map(|x| x * w)
                            .collect(),
                    );
                }
                values.into_iter().map(|x| x / total_w).collect()
            }
            Source::WeightedProduct { sources } => {
                let mut values = vec![1.0; samples];
                for (w, s) in sources {
                    utils::mul_elementwise(
                        &mut values,
                        s.get_vec(start, samples)
                            .into_iter()
                            .map(|x| x.powf(*w))
                            .collect(),
                    );
                }
                values
            }
            Source::Inverted { source } => source
                .get_vec(start, samples)
                .into_iter()
                .map(|x| -x + 1.0)
                .collect(),
            Source::Transformed { func, source } => source
                .get_vec(start, samples)
                .into_iter()
                .map(func)
                .collect(),
        }
    }
}

#[derive(Debug)]
pub struct Control {
    value: f64,
    range: (f64, f64),
    source: Option<Source>,
}

impl Control {
    pub fn new(
        value_in_range: f64,
        range: (f64, f64),
        source: Source,
    ) -> Result<Self, ControlError> {
        let new_value = (value_in_range - range.0) / (range.1 - range.0);
        if !(0.0..=1.0).contains(&new_value) {
            return Err(ControlError::new_range_err(value_in_range, range));
        }
        Ok(Self {
            value: new_value,
            range,
            source: Some(source),
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
                source: None,
            })
        }
    }

    #[inline(always)]
    fn put_in_range(&self, value: f64) -> f64 {
        (self.range.1 - self.range.0) * value + self.range.0
    }
}

pub fn check_id(ids: &[usize], id: usize) -> Result<(), ControlError> {
    if ids.contains(&id) {
        Err(ControlError::new_circ_ref_err(id))
    } else {
        Ok(())
    }
}

pub fn opt_try_set_checked(
    to_replace: &mut Option<Control>,
    range: (f64, f64),
    other: Control,
    id: usize,
) -> Result<(), ControlError> {
    if let Some(ctrl) = to_replace {
        debug_assert_eq!(ctrl.range, range);
        ctrl.try_set_checked(other, id)?
    } else {
        other.cmp_ranges(range)?;
        *to_replace = Some(other)
    }
    Ok(())
}

impl Control {
    pub fn try_set(&mut self, other: Self) -> Result<(), ControlError> {
        self.cmp_ranges(other.range)?;
        self.source = other.source;
        self.value = other.value;
        Ok(())
    }

    pub fn try_set_checked(&mut self, other: Self, id: usize) -> Result<(), ControlError> {
        todo!();
    }

    pub fn cmp_ranges(&self, range: (f64, f64)) -> Result<(), ControlError> {
        if self.range == range {
            Ok(())
        } else {
            Err(ControlError::new_range_mismatch_err(self.range, range))
        }
    }

    pub fn get_source(&self) -> &Option<Source> {
        &self.source
    }

    pub fn loose_source(&mut self) {
        self.source = None;
    }
}

impl Control {
    pub fn get_value(&self, time: TimeStamp) -> f64 {
        let val: f64 = match &self.source {
            Some(source) => source.get_value(time),
            None => self.value,
        };
        self.put_in_range(val)
    }

    pub fn get_vec(&self, start: TimeStamp, samples: usize) -> Vec<f64> {
        match &self.source {
            Some(source) => source
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
    DoubleId {
        id: usize,
    },
    FNotFound {
        id: usize,
    },
    PhantomF,
}

#[derive(Debug)]
pub struct ControlError {
    path: Vec<String>,
    origin: String,
    control: String,
    kind: ErrorKind,
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
            kind: ErrorKind::RangeMismatch {
                trg_range,
                src_range,
            },
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

    pub fn new_double_id_err(id: usize) -> Self {
        Self {
            path: Vec::new(),
            origin: String::new(),
            control: String::new(),
            kind: ErrorKind::DoubleId { id },
        }
    }

    pub fn new_func_not_found(id: usize) -> Self {
        Self {
            path: Vec::new(),
            origin: String::new(),
            control: String::new(),
            kind: ErrorKind::FNotFound { id },
        }
    }

    pub fn new_phantom_f_err() -> Self {
        Self {
            path: Vec::new(),
            origin: String::new(),
            control: String::new(),
            kind: ErrorKind::PhantomF,
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
            ErrorKind::RangeMismatch {
                trg_range,
                src_range,
            } => {
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
            }
            ErrorKind::DoubleId { id } => {
                format!(
                    "The id : {}, was encountered twice while creating IdMap",
                    id
                )
            }
            ErrorKind::FNotFound { id } => {
                format!(
                    "The source for {} in {} could not find the function with id: {}!\n    full path to control: {}/{}/{}",
                    self.control,
                    self.origin,
                    id,
                    self.path.join("/"),
                    self.origin,
                    self.control,
                )
            }
            ErrorKind::PhantomF =>
            format!(
                "The source for {} in {} has no assigned CtrlFunction!\n    full path to control: {}/{}/{}",
                self.control,
                self.origin,
                self.path.join("/"),
                self.origin,
                self.control,
            ),
        }
    }
}

impl Display for ControlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_string())
    }
}
