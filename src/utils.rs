use std::iter::zip;

use crate::constants::SAMPLE_RATE;

pub fn normalize(track: &Vec<f64>) -> Vec<f64> {
    let norm =
        (track.iter().map(|x| x * x).fold(0.0, |acc, x| acc + x) / (track.len() as f64)).sqrt();
    track.iter().map(|x| x / norm).collect()
}

pub fn add_same_len(main: &mut Vec<f64>, summand: Vec<f64>) {
    for (e1, e2) in zip(main, summand) {
        *e1 += e2;
    }
}

pub fn mult_same_len(main: &mut Vec<f64>, factor: Vec<f64>) {
    for (e1, e2) in zip(main, factor) {
        *e1 *= e2;
    }
}

pub fn add_from_index(main: &mut Vec<f64>, summand: Vec<f64>, index: usize) {
    if main.len() < index + summand.len() {
        main.resize(index + summand.len(), 0.0)
    }
    for i in 0..summand.len() {
        main[i + index] += summand[i];
    }
}
pub fn add_from_index_by_ref(main: &mut Vec<f64>, summand: &Vec<f64>, index: usize) {
    if main.len() < index + summand.len() {
        main.resize(index + summand.len(), 0.0)
    }
    for i in 0..summand.len() {
        main[i + index] += summand[i];
    }
}

#[inline(always)]
pub fn seconds_to_samples(seconds: f64) -> usize {
    (seconds * (SAMPLE_RATE as f64)) as usize
}

#[inline(always)]
pub fn samples_to_seconds(samples: usize) -> f64 {
    (samples as f64) / (SAMPLE_RATE as f64)
}
