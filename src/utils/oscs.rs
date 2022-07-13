use factorial::Factorial;

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
    pub fn get_sample(&self, phase: f64, modulation: f64) -> f64 {
        use Oscillator::*;
        match self {
            Sine => aproximate_sin::<5>(phase),
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
        assert_eq!(freq.len(), samples);
        assert_eq!(modulation.len(), samples);
        let mut out = Vec::with_capacity(samples);
        let mut phase = phase_shift;
        for i in 0..samples {
            phase += TAU * freq[i] / (SAMPLE_RATE as f64) % TAU;
            out.push(self.get_sample(phase, modulation[i]))
        }
        out
    }
}

fn aproximate_sin<const ITTERATIONS: u8>(x: f64) -> f64 {
    let mut result = 0.0;
    for i in 0..ITTERATIONS {
        result += x.powi(i as i32) / (Factorial::factorial(&i) as f64)
    }
    result
}
