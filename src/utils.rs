use crate::consts::SAMPLE_RATE;

pub mod oscs;

#[inline(always)]
pub fn seconds_to_samples(seconds: f64) -> usize {
    (seconds * (SAMPLE_RATE as f64)) as usize
}

#[inline(always)]
pub fn samples_to_seconds(samples: usize) -> f64 {
    (samples as f64) / (SAMPLE_RATE as f64)
}

#[inline(always)]
pub fn smooth_step(x: f64) -> f64 {
    3.0 * x * x - 2.0 * x * x * x
}

#[inline(always)]
pub fn add_elementwise(v1: &mut Vec<f64>, v2: Vec<f64>) {
    debug_assert_eq!(v1.len(), v2.len());
    for (x2, x1) in v2.into_iter().zip(v1) {
        *x1 += x2
    }
}

pub fn user_input(prompt: &str) -> String {
    println!("{}", prompt);

    let mut input = String::new();
    match std::io::stdin().read_line(&mut input) {
        Ok(_) => {}
        Err(error) => println!("{}", error),
    };
    input
}
