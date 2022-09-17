use serde::{Deserialize, Serialize};

use crate::{network::Receiver, receivers::VOL_RECEIVER, time::ClockTick, wave::Wave};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Volume {
    volume: Receiver,
    on: bool,
}

impl Volume {
    pub fn new() -> Self {
        Self {
            volume: VOL_RECEIVER,
            on: true,
        }
    }
}

impl Default for Volume {
    fn default() -> Self {
        Self::new()
    }
}

impl Volume {
    pub fn apply(&self, wave: &mut Wave, time_triggered: ClockTick) {
        if self.on {
            let vol = self.volume.get_vec(time_triggered, wave.len());
            wave.scale_by_vec(vol)
        }
    }

    pub fn set_defaults(&mut self) {
        self.volume = VOL_RECEIVER
    }

    pub fn on(&mut self) {
        self.on = true
    }

    pub fn off(&mut self) {
        self.on = false
    }

    pub fn toggle(&mut self) {
        self.on = !self.on
    }
}

impl Volume {
    pub fn extract(&self) -> Self {
        Self {
            volume: self.volume.extract(),
            on: self.on,
        }
    }

    pub fn set_id(&mut self, track_id: u8) {
        self.volume.set_id(track_id)
    }
}
