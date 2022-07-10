use std::f64::consts::{PI, TAU};
use std::marker::PhantomData;

use crate::{auto::ValOrVec, consts::SAMPLE_RATE, wave::Wave};

pub trait Oscillator<W: Wave> {
    fn get_sample(&self, phase: f64, modulation: f64) -> f64;

    fn val_all(&self, freq: &f64, modulation: &f64, samples: usize) -> Vec<f64> {
        let mut out = Vec::with_capacity(samples);
        for i in 0..samples {
            let phase = (i as f64) * TAU * freq / (SAMPLE_RATE as f64) % TAU;
            out.push(self.get_sample(phase, *modulation))
        }
        out
    }

    fn var_freq(&self, freq: &Vec<f64>, modulation: &f64, samples: usize) -> Vec<f64> {
        assert_eq!(freq.len(), samples);
        let mut out = Vec::with_capacity(samples);
        let mut phase = 0.0;
        for f in freq {
            phase += TAU * f / (SAMPLE_RATE as f64) % TAU;
            out.push(self.get_sample(phase, *modulation))
        }
        out
    }

    fn var_mod(&self, freq: &f64, modulation: &Vec<f64>, samples: usize) -> Vec<f64> {
        assert_eq!(modulation.len(), samples);
        let mut out = Vec::with_capacity(samples);
        for (i, m) in modulation.iter().enumerate() {
            let phase = (i as f64) * TAU * freq / (SAMPLE_RATE as f64) % TAU;
            out.push(self.get_sample(phase, *m))
        }
        out
    }

    fn var_all(&self, freq: &Vec<f64>, modulation: &Vec<f64>, samples: usize) -> Vec<f64> {
        assert_eq!(freq.len(), samples);
        assert_eq!(modulation.len(), samples);
        let mut out = Vec::with_capacity(samples);
        let mut phase = 0.0;
        for i in 0..samples {
            phase += TAU * freq[i] / (SAMPLE_RATE as f64) % TAU;
            out.push(self.get_sample(phase, modulation[i]))
        }
        out
    }

    fn wave(&self, freq: &ValOrVec, modulation: &ValOrVec, samples: usize) -> W {
        use ValOrVec::*;
        W::from_vec(match freq {
            Val(freq) => match modulation {
                Val(modulation) => self.val_all(freq, modulation, samples),
                Vec(modulation) => self.var_mod(freq, modulation, samples),
            },
            Vec(freq) => match modulation {
                Val(modulation) => self.var_freq(freq, modulation, samples),
                Vec(modulation) => self.var_all(freq, modulation, samples),
            },
        })
    }
}

pub struct Sine<W>(f64, PhantomData<W>);

impl<W: Wave> Oscillator<W> for Sine<W> {
    #[inline(always)]
    fn get_sample(&self, phase: f64, _modulation: f64) -> f64 {
        phase.sin() * self.0
    }
}

pub struct ModSquare<W>(f64, PhantomData<W>);

impl<W: Wave> Oscillator<W> for ModSquare<W> {
    #[inline(always)]
    fn get_sample(&self, phase: f64, modulation: f64) -> f64 {
        if phase < modulation * TAU {
            self.0
        } else {
            -self.0
        }
    }
}

pub struct ModSaw<W>(f64, PhantomData<W>);

impl<W: Wave> Oscillator<W> for ModSaw<W> {
    #[inline(always)]
    fn get_sample(&self, phase: f64, modulation: f64) -> f64 {
        if phase < modulation * TAU {
            (phase / modulation / PI - 1.0) * self.0
        } else {
            ((phase - (modulation + 1.0) * PI) / (modulation - 1.0) / PI) * self.0
        }
    }
}
