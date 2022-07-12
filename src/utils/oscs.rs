use crate::consts::SAMPLE_RATE;
use std::{f64::consts::{PI, TAU}, fmt::Debug};

pub trait Oscillator: Debug {
    fn get_sample(&self, phase: f64, modulation: f64) -> f64;

    fn play(&self, freq: &Vec<f64>, modulation: &Vec<f64>, samples: usize) -> Vec<f64> {
        self.play_shifted(freq, modulation, samples, 0.0)
    }

    fn play_shifted(
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

#[derive(Debug)]
pub struct Sine(f64);

impl Oscillator for Sine {
    #[inline(always)]
    fn get_sample(&self, phase: f64, _modulation: f64) -> f64 {
        phase.sin() * self.0
    }
}
impl Sine {
    pub fn new(gain: f64) -> Self {
        Self(gain)
    }
}

#[derive(Debug)]
pub struct ModSquare(f64);

impl Oscillator for ModSquare {
    #[inline(always)]
    fn get_sample(&self, phase: f64, modulation: f64) -> f64 {
        if phase < modulation * TAU {
            self.0
        } else {
            -self.0
        }
    }
}
impl ModSquare {
    pub fn new(gain: f64) -> Self {
        Self(gain)
    }
}

#[derive(Debug)]
pub struct ModSaw(f64);

impl Oscillator for ModSaw {
    #[inline(always)]
    fn get_sample(&self, phase: f64, modulation: f64) -> f64 {
        if phase < modulation * TAU {
            (phase / modulation / PI - 1.0) * self.0
        } else {
            ((phase - (modulation + 1.0) * PI) / (modulation - 1.0) / PI) * self.0
        }
    }
}
impl ModSaw {
    pub fn new(gain: f64) -> Self {
        Self(gain)
    }
}
