use std::{sync::Mutex, collections::HashMap, cell::RefCell};

use once_cell::sync::Lazy;

use crate::{time::TimeManager, ctrl_f::{CtrlFunction, Envelope, FunctionManager},};

pub const SAMPLE_RATE: usize = 44100;

pub static TIME_MANAGER: Lazy<Mutex<TimeManager>> = Lazy::new(Mutex::default);


static FUNCTION_MANAGER: Lazy<Mutex<FunctionManager>> = Lazy::new(Mutex::default);

