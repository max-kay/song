use std::{fmt::Debug, path::Path};

pub mod mono;
pub mod stereo;

pub use mono::Mono;
pub use stereo::Stereo;

pub trait Wave: Clone + Debug {
    fn new() -> Self;
    fn with_capacity(capacity: usize) -> Self;
    fn zeros(length: usize) -> Self;
    fn ones(length: usize) -> Self;
    fn from_vec(vec: Vec<f64>) -> Self;
    fn resize(&mut self, new_len: usize, value: f64);
    fn clear(&mut self);

    fn add(&mut self, other: &Self, index: usize);
    fn add_consuming(&mut self, other: Self, index: usize);

    fn scale(&mut self, value: f64);
    fn scale_by_vec(&mut self, vec: Vec<f64>);

    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;

    fn normalize(&mut self);
    fn peak_normalize(&mut self);

    fn save(&self, path: &Path); //TODO error
}
