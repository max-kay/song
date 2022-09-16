use serde::{Deserialize, Serialize};

use crate::globals::SAMPLE_RATE;
use std::{
    f32::consts::{PI, TAU},
    fmt::Debug,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Oscillator {
    Sine,
    ModSquare,
    ModSaw,
}

impl Default for Oscillator {
    fn default() -> Self {
        Self::Sine
    }
}

impl Oscillator {
    #[inline(always)] // TODO Performance
    pub fn get_sample(&self, phase: f32, modulation: f32) -> f32 {
        use Oscillator::*;
        match self {
            Sine => phase.sin(), // TODO should I use a aproximation for (0, Tau)?
            ModSquare => {
                if phase < modulation * TAU {
                    1.0
                } else {
                    -1.0
                }
            }
            ModSaw => {
                if phase < modulation * TAU {
                    phase / modulation / PI - 1.0
                } else {
                    (phase - (modulation + 1.0) * PI) / (modulation - 1.0) / PI
                }
            }
        }
    }

    pub fn play(&self, freq: &[f32], modulation: &[f32], samples: usize) -> Vec<f32> {
        self.play_shifted(freq, modulation, samples, 0.0)
    }

    pub fn play_shifted(
        &self,
        freq: &[f32],
        modulation: &[f32],
        samples: usize,
        phase_shift: f32,
    ) -> Vec<f32> {
        debug_assert_eq!(
            freq.len(),
            samples,
            "freq.len() doesn't match the requested samples"
        );
        debug_assert_eq!(
            modulation.len(),
            samples,
            "modulation.len() doesn't match the requested samples"
        );
        let mut out = Vec::with_capacity(samples);
        let mut phase = phase_shift;
        for i in 0..samples {
            phase += TAU * freq[i] / (SAMPLE_RATE as f32);
            phase %= TAU;
            out.push(self.get_sample(phase, modulation[i]))
        }
        out
    }
}
