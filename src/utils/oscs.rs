use crate::consts::SAMPLE_RATE;
use std::{
    f64::consts::{PI, TAU},
    fmt::Debug,
};

#[derive(Debug)]
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
    #[inline(always)]
    pub fn get_sample(&self, phase: f64, modulation: f64) -> f64 {
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

    pub fn play(&self, freq: &Vec<f64>, modulation: &Vec<f64>, samples: usize) -> Vec<f64> {
        self.play_shifted(freq, modulation, samples, 0.0)
    }

    pub fn play_shifted(
        &self,
        freq: &Vec<f64>,
        modulation: &Vec<f64>,
        samples: usize,
        phase_shift: f64,
    ) -> Vec<f64> {
        assert_eq!(
            freq.len(),
            samples,
            "freq.len() doesn't match the requested samples"
        );
        assert_eq!(
            modulation.len(),
            samples,
            "modulation.len() doesn't match the requsted samples"
        );
        let mut out = Vec::with_capacity(samples);
        let mut phase = phase_shift;
        for i in 0..samples {
            phase += TAU * freq[i] / (SAMPLE_RATE as f64);
            phase %= TAU;
            out.push(self.get_sample(phase, modulation[i]))
        }
        out
    }
}
