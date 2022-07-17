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

#[inline]
pub fn add_elementwise<T: std::ops::AddAssign>(v1: &mut Vec<T>, v2: Vec<T>) {
    assert_eq!(
        v1.len(),
        v2.len(),
        "vectors passed to add_elementwise didn't have equal len"
    );
    for (x2, x1) in v2.into_iter().zip(v1) {
        *x1 += x2
    }
}

pub fn max_abs_f64(vec: &[f64]) -> f64 {
    let max = vec
        .iter()
        .fold(None, |r, &val| match r {
            Some(p) => Some(f64::max(p, val)),
            None => Some(val),
        })
        .unwrap_or(0.0);
    let min = vec
        .iter()
        .fold(None, |r, &val| match r {
            Some(p) => Some(f64::min(p, val)),
            None => Some(val),
        })
        .unwrap_or(0.0);
    f64::max(f64::abs(max), f64::abs(min))
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

#[cfg(test)]
mod test {
    use super::add_elementwise;

    #[test]
    fn test_elementwise_addition() {
        let mut v1: Vec<i32> = vec![2, 4, 6, 8];
        let v2: Vec<i32> = vec![4, 5, 1, 7];
        add_elementwise(&mut v1, v2);
        assert_eq!(v1, vec![6, 9, 7, 15])
    }

    #[test]
    #[should_panic(expected = "equal len")]
    fn test_panic_unequal_len_add() {
        let mut v1: Vec<i32> = vec![2, 4, 6];
        let v2: Vec<i32> = vec![4, 5, 1, 7];
        add_elementwise(&mut v1, v2);
    }
}
