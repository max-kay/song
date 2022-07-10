use crate::consts::SAMPLE_RATE;
pub mod oscs;
pub mod envelope;

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

pub fn user_input(prompt: &str) -> String {
    println!("{}", prompt);

    let mut input = String::new();
    match std::io::stdin().read_line(&mut input) {
        Ok(_) => {}
        Err(error) => println!("{}", error),
    };
    input
}
