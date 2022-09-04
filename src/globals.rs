use std::sync::RwLock;

use once_cell::sync::Lazy;

use crate::{ctrl_f::GeneratorManager, time::TimeManager};

pub const SAMPLE_RATE: usize = 44100;

pub static TIME_MANAGER: Lazy<RwLock<TimeManager>> = Lazy::new(RwLock::default);

pub static GENRATOR_MANAGER: Lazy<RwLock<GeneratorManager>> = Lazy::new(RwLock::default);
