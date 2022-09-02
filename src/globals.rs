use std::sync::Mutex;

use once_cell::sync::Lazy;

use crate::{ctrl_f::GeneratorManager, time::TimeManager};

pub const SAMPLE_RATE: usize = 44100;

pub static TIME_MANAGER: Lazy<Mutex<TimeManager>> = Lazy::new(Mutex::default);

pub static FUNCTION_MANAGER: Lazy<Mutex<GeneratorManager>> = Lazy::new(Mutex::default);
