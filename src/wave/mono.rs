use std::{iter::zip, path::Path};

use hound::WavSpec;

use crate::{globals::SAMPLE_RATE, utils};

use super::Wave;

const STD_SPEC: WavSpec = WavSpec {
    channels: 1,
    sample_rate: SAMPLE_RATE as u32,
    bits_per_sample: 16,
    sample_format: hound::SampleFormat::Int,
};

#[derive(Debug, Clone)]
pub struct Mono {
    wave: Vec<f64>,
}

impl Wave for Mono {
    fn new() -> Self {
        Self { wave: Vec::new() }
    }

    fn with_capacity(capacity: usize) -> Self {
        Self {
            wave: Vec::with_capacity(capacity),
        }
    }

    fn zeros(length: usize) -> Self {
        Self {
            wave: vec![0.0; length],
        }
    }

    fn ones(length: usize) -> Self {
        Self {
            wave: vec![1.0; length],
        }
    }

    fn from_vec(vec: Vec<f64>) -> Self {
        Self { wave: vec }
    }

    fn resize(&mut self, new_len: usize, value: f64) {
        self.wave.resize(new_len, value)
    }

    fn clear(&mut self) {
        self.wave = Vec::new()
    }

    fn add(&mut self, other: &Self, index: usize) {
        if index == 0 && self.len() == other.len() {
            for (e1, e2) in zip(&mut self.wave, other.wave.iter()) {
                *e1 += e2;
            }
        } else {
            if self.len() < index + other.len() {
                self.resize(index + other.len(), 0.0)
            }
            for i in 0..other.len() {
                self.wave[i + index] += other.wave[i];
            }
        }
    }

    fn add_consuming(&mut self, other: Self, index: usize) {
        if index == 0 && self.len() == other.len() {
            for (e1, e2) in zip(&mut self.wave, other.wave) {
                *e1 += e2;
            }
        } else {
            if self.len() < index + other.len() {
                self.wave.resize(index + other.len(), 0.0)
            }
            for i in 0..other.len() {
                self.wave[i + index] += other.wave[i];
            }
        }
    }
    fn scale(&mut self, value: f64) {
        self.wave = self.wave.iter().map(|x| x * value).collect()
    }

    fn scale_by_vec(&mut self, vec: Vec<f64>) {
        debug_assert_eq!(self.len(), vec.len(), "error in scale_by_vec");
        for (e1, e2) in zip(&mut self.wave, vec.into_iter()) {
            *e1 *= e2;
        }
    }

    fn len(&self) -> usize {
        self.wave.len()
    }

    fn is_empty(&self) -> bool {
        self.wave.is_empty()
    }

    fn normalize(&mut self) {
        // TODO bad code
        let norm = (self.wave.iter().map(|x| x * x).fold(0.0, |acc, x| acc + x)
            / (self.wave.len() as f64))
            .sqrt();
        self.wave = self.wave.iter().map(|x| x / norm).collect();
    }

    fn peak_normalize(&mut self) {
        let scale = 0.9 / utils::max_abs_f64(&self.wave);
        self.scale(scale)
    }

    fn save(&self, path: &Path) {
        let mut writer =
            hound::WavWriter::create(path, STD_SPEC).expect("Error while saving wave!");
        let mut writer_i16 = writer.get_i16_writer(self.len() as u32);
        for sample in self.wave.iter().map(|x| (x * i16::MAX as f64) as i16) {
            unsafe { writer_i16.write_sample_unchecked(sample) }
        }
        writer_i16.flush().expect("Error while saving wave!")
    }
}
