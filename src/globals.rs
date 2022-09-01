use std::sync::Mutex;

use once_cell::sync::Lazy;

use crate::time::TimeManager;

pub const SAMPLE_RATE: usize = 44100;

pub static TIME_MANAGER: Lazy<Mutex<Box<TimeManager>>> = Lazy::new(|| Mutex::default());
