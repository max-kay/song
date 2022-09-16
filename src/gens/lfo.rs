use serde::{Deserialize, Serialize};

use crate::{
    gens::Error,
    globals::{SAMPLE_RATE, TIME_MANAGER},
    network::{self, Receiver, Transform},
    time::ClockTick,
    utils::oscs::Oscillator,
};
use std::f32::consts::TAU;

use super::{GenId, Generator};

const FREQ_RECEIVER: Receiver = Receiver::new(2.0, (0.001, 20.0), Transform::Linear);
const MOD_RECEIVER: Receiver = Receiver::new(0.5, (0.0, 1.0), Transform::Linear);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lfo {
    id: GenId,
    oscillator: Oscillator,
    freq: Receiver,
    modulation: Receiver,
    phase_shift: f32,
}

impl Lfo {
    pub fn new() -> Self {
        Self {
            id: GenId::Unbound,
            oscillator: Oscillator::Sine,
            freq: FREQ_RECEIVER,
            modulation: MOD_RECEIVER,
            phase_shift: 0.0,
        }
    }

    pub fn w_default() -> Generator {
        Generator::Lfo(Self::default())
    }

    pub(crate) fn set_id(&mut self, id: GenId) {
        self.id = id
    }

    pub fn get_sub_ids(&self) -> Vec<GenId> {
        let mut out = self.freq.get_ids();
        out.append(&mut self.modulation.get_ids());
        out
    }
}

impl Lfo {
    pub fn set(&mut self, other: &Lfo) -> Result<(), Error> {
        self.set_freq(&other.freq)?;
        self.set_modulation(&other.modulation)?;
        self.phase_shift = other.phase_shift;
        self.oscillator = other.oscillator;
        Ok(())
    }

    pub fn set_freq(&mut self, freq: &Receiver) -> Result<(), Error> {
        network::set_receiver(&mut self.freq, self.id, freq)
    }

    pub fn set_modulation(&mut self, modulation: &Receiver) -> Result<(), Error> {
        network::set_receiver(&mut self.modulation, self.id, modulation)
    }

    pub fn wrap(self) -> Generator {
        Generator::Lfo(self)
    }
}

impl Default for Lfo {
    fn default() -> Self {
        Self::new()
    }
}

impl Lfo {
    pub fn get_val(&self, time: ClockTick) -> f32 {
        let phase =
            ((TIME_MANAGER.read().unwrap().tick_to_second(time) * TAU * self.freq.get_val(time)
                / (SAMPLE_RATE as f32))
                + self.phase_shift)
                % TAU;
        (self
            .oscillator
            .get_sample(phase, self.modulation.get_val(time))
            + 1.0)
            / 2.0
    }

    pub fn get_vec(&self, start: ClockTick, samples: usize) -> Vec<f32> {
        self.oscillator
            .play_shifted(
                &self.freq.get_vec(start, samples),
                &self.modulation.get_vec(start, samples),
                samples,
                self.phase_shift,
            )
            .into_iter()
            .map(|x| (x + 1.0) / 2.0)
            .collect()
    }
}
