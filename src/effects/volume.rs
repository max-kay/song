use once_cell::sync::Lazy;
use crate::{
    network::{Reciever, Transform},
    time::TimeStamp,
    wave::Wave,
};

use super::{EffMarker, Effect};

static VOL_RECIEVER: Lazy<Reciever> =
    Lazy::new(|| Reciever::new(1.0, (0.0, 5.0), Transform::Linear));

#[derive(Debug)]
pub struct Volume {
    volume: Reciever,
    on: bool,
}

impl Volume {
    pub fn new() -> Self {
        Self {
            volume: VOL_RECIEVER.clone(),
            on: true,
        }
    }
}

impl Default for Volume {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for Volume {
    fn apply(&self, wave: &mut Wave, time_triggered: TimeStamp) {
        if self.on {
            let vol = self.volume.get_vec(time_triggered, wave.len());
            wave.scale_by_vec(vol)
        }
    }

    fn set_defaults(&mut self) {
        self.volume = VOL_RECIEVER.clone()
    }

    fn on(&mut self) {
        self.on = true
    }

    fn off(&mut self) {
        self.on = false
    }

    fn toggle(&mut self) {
        self.on = !self.on
    }
}

impl EffMarker for Volume {}
