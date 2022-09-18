use crate::{globals::SAMPLE_RATE, utils};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use hound::WavSpec;
use itertools::interleave;
use std::{
    fmt::Debug,
    iter::zip,
    path::Path,
    sync::{Arc, Condvar, Mutex},
};

#[derive(Debug, Clone)]
pub struct Wave {
    right: Vec<f32>,
    left: Vec<f32>,
}

impl Wave {
    pub fn new() -> Self {
        Self {
            right: Vec::new(),
            left: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            right: Vec::with_capacity(capacity),
            left: Vec::with_capacity(capacity),
        }
    }

    pub fn zeros(length: usize) -> Self {
        Self {
            right: vec![0.0; length],
            left: vec![0.0; length],
        }
    }

    pub fn ones(length: usize) -> Self {
        Self {
            right: vec![1.0; length],
            left: vec![1.0; length],
        }
    }

    pub fn from_vec(vec: Vec<f32>) -> Self {
        Self {
            right: vec.clone(),
            left: vec,
        }
    }

    pub fn from_vecs(right: Vec<f32>, left: Vec<f32>) -> Self {
        Self { right, left }
    }

    pub fn resize(&mut self, new_len: usize, value: f32) {
        self.right.resize(new_len, value);
        self.left.resize(new_len, value)
    }

    pub fn clear(&mut self) {
        self.right.clear();
        self.left.clear()
    }

    pub fn add(&mut self, other: &Self, index: usize) {
        if index == 0 && self.len() == other.len() {
            for (e1, e2) in zip(&mut self.right, other.right.iter()) {
                *e1 += e2;
            }
            for (e1, e2) in zip(&mut self.left, other.left.iter()) {
                *e1 += e2;
            }
        } else {
            if self.len() < index + other.len() {
                self.resize(index + other.len(), 0.0)
            }
            for i in 0..other.len() {
                self.right[i + index] += other.right[i];
            }
            for i in 0..other.len() {
                self.left[i + index] += other.left[i];
            }
        }
    }

    pub fn scale(&mut self, value: f32) {
        self.right = self.right.iter().map(|x| x * value).collect();
        self.left = self.left.iter().map(|x| x * value).collect()
    }

    pub fn scale_by_vec(&mut self, vec: Vec<f32>) {
        debug_assert_eq!(self.len(), vec.len(), "error in scale_by_vec");
        for (e1, e2) in zip(&mut self.right, vec.iter()) {
            *e1 *= e2;
        }
        for (e1, e2) in zip(&mut self.left, vec.into_iter()) {
            *e1 *= e2;
        }
    }

    pub fn len(&self) -> usize {
        self.right.len()
    }

    pub fn is_empty(&self) -> bool {
        self.right.is_empty()
    }

    pub fn rms_normalize(&mut self) {
        let rms = ((self.left.iter().fold(0.0, |i, x| i + x * x)
            + self.right.iter().fold(0.0, |i, x| i + x * x))
            / (2.0 * self.len() as f32))
            .sqrt();
        self.right.iter_mut().for_each(|x| *x /= rms * 10.0);
        self.left.iter_mut().for_each(|x| *x /= rms * 10.0); // TODO
    }

    pub fn peak_normalize(&mut self) {
        let scale = 0.9
            / f32::max(
                utils::max_abs_f32(&self.right),
                utils::max_abs_f32(&self.left),
            );
        self.scale(scale)
    }

    pub fn save(&self, path: impl AsRef<Path>) {
        let mut wave = self.clone();
        wave.rms_normalize();

        let spec = WavSpec {
            channels: 2,
            sample_rate: SAMPLE_RATE as u32,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let mut writer = hound::WavWriter::create(path, spec).expect("Error while saving wave!");
        let mut writer_i16 = writer.get_i16_writer(wave.len() as u32 * 2);
        let right = wave.right.iter().map(|x| (x * i16::MAX as f32) as i16);
        let left = wave.left.iter().map(|x| (x * i16::MAX as f32) as i16);
        for (r, l) in zip(right, left) {
            unsafe {
                writer_i16.write_sample_unchecked(r);
                writer_i16.write_sample_unchecked(l);
            }
        }
        writer_i16.flush().expect("Error while saving wave!");
        writer.finalize().expect("Error while saving wave!");
    }
}

impl Wave {
    pub fn interleave(self) -> Vec<f32> {
        interleave(self.right, self.left).collect()
    }

    pub fn dirty_play(&self) {
        let mut wave = self.clone();
        wave.rms_normalize();
        let mut wave = wave.interleave().into_iter();

        let host = cpal::default_host();

        let device = host
            .default_output_device()
            .expect("no default output audio");

        let mut supported_configs_range = device
            .supported_output_configs()
            .expect("error while querying configs");

        let supported_config = supported_configs_range
            .next()
            .expect("no supported config?!")
            .with_max_sample_rate();

        let err_fn = |err: cpal::StreamError| {
            eprintln!("an error occurred on the output audio stream: {}", err)
        };

        let mine = Arc::new((Mutex::new(false), Condvar::new()));
        let in_stream = Arc::clone(&mine);

        let stream = device
            .build_output_stream(
                &supported_config.into(),
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    for d in data {
                        match wave.next() {
                            Some(s) => *d = s,
                            None => {
                                *d = 0.0;
                                let (lock, cvar) = &*in_stream;
                                let mut started = lock.lock().unwrap();
                                *started = true;
                                // We notify the condvar that the value has changed.
                                cvar.notify_one();
                            }
                        }
                    }
                },
                err_fn,
            )
            .unwrap();

        stream.play().unwrap();
        let (lock, cvar) = &*mine;
        let mut started = lock.lock().unwrap();
        while !*started {
            started = cvar.wait(started).unwrap();
        }
    }
}

impl Default for Wave {
    fn default() -> Self {
        Self::new()
    }
}
