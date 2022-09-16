use std::{
    io,
    ops::{AddAssign, MulAssign},
};

use serde::{Deserialize, Serialize};

use crate::{globals::SAMPLE_RATE, Error};

pub mod oscs;

#[inline(always)]
pub fn seconds_to_samples(seconds: f32) -> usize {
    (seconds * (SAMPLE_RATE as f32)) as usize
}

#[inline(always)]
pub fn samples_to_seconds(samples: usize) -> f32 {
    (samples as f32) / (SAMPLE_RATE as f32)
}

#[inline(always)]
pub fn smooth_step(x: f32) -> f32 {
    3.0 * x * x - 2.0 * x * x * x
}

#[inline]
pub fn add_elementwise<T: AddAssign + Copy>(v1: &mut Vec<T>, v2: &[T]) {
    debug_assert_eq!(
        v1.len(),
        v2.len(),
        "vectors passed to add_elementwise didn't have equal len"
    );
    for (x2, x1) in v2.iter().zip(v1) {
        *x1 += *x2
    }
}

#[inline]
pub fn mul_elementwise<T: MulAssign + Copy>(v1: &mut Vec<T>, v2: &[T]) {
    debug_assert_eq!(
        v1.len(),
        v2.len(),
        "vectors passed to add_elementwise didn't have equal len"
    );
    for (x2, x1) in v2.iter().zip(v1) {
        *x1 *= *x2
    }
}

#[inline(always)] //is this needed TODO
pub fn cents_to_factor(cents: &mut [f32]) {
    cents.iter_mut().for_each(|x| *x = fast_pow2(*x / 1200.0))
}

// https://karmafx.net/docs/karmafx_floattricks.pdf
// Ian Stephenson

#[inline(always)]
pub fn fast_pow2(x: f32) -> f32 {
    let mp = 0.33971;
    let mut temp = x - x.floor();
    temp = (temp - temp * temp) * mp;
    let mut result = x + 127.0 - temp;
    result *= (1 << 23) as f32;
    unsafe {
        let casted = result as i32;
        *(&mut result as *mut f32 as *mut i32) = casted;
    }
    result
}

#[cfg(test)]
mod test {
    use std::ptr::read_volatile;

    use super::fast_pow2;
    #[test]
    fn pow2_test() {
        let floats = (0..100).map(|x| -2.0 + x as f32 / 25.0);
        assert!(floats
            .map(|x| (2.0_f32.powf(x) / fast_pow2(x) - 1.0).abs())
            .all(|x| x <= 0.004))
    }

    #[test]
    fn pow2_speed() {
        let floats0 = (0..100000).map(|x| -2.0 + x as f32 / 25000.0);
        let floats1 = floats0.clone();
        let start = std::time::Instant::now();
        for f in floats0 {
            unsafe {
                read_volatile((&2.0_f32.powf(f)) as *const f32);
            }
        }
        let t0 = start.elapsed().as_nanos();
        let start = std::time::Instant::now();
        for f in floats1 {
            unsafe {
                read_volatile((&fast_pow2(f))as *const f32);
            }
        }
        let t1 = start.elapsed().as_nanos();
        println!("normal pow: {}, fast pow: {}", t0, t1);
        assert!(t1 < t0)
    }
}

pub fn max_abs_f32(vec: &[f32]) -> f32 {
    let max = vec
        .iter()
        .fold(None, |r, &val| match r {
            Some(p) => Some(f32::max(p, val)),
            None => Some(val),
        })
        .unwrap_or(0.0);
    let min = vec
        .iter()
        .fold(None, |r, &val| match r {
            Some(p) => Some(f32::min(p, val)),
            None => Some(val),
        })
        .unwrap_or(0.0);
    f32::max(f32::abs(max), f32::abs(min))
}

pub fn user_input(prompt: &str) -> String {
    println!("{}", prompt);

    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => {}
        Err(error) => println!("{}", error),
    };
    input
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XYPairs<K: Ord + Copy, V: Copy> {
    xs: Vec<K>,
    ys: Vec<V>,
}

impl<K: Copy + Ord, V: Copy> XYPairs<K, V> {
    pub fn new() -> Self {
        Self {
            xs: Vec::new(),
            ys: Vec::new(),
        }
    }

    pub fn from_vecs(mut xs: Vec<K>, mut ys: Vec<V>) -> Self {
        let mut permutation = permutation::sort(&xs);
        permutation.apply_slice_in_place(&mut xs);
        permutation.apply_slice_in_place(&mut ys);
        Self { xs, ys }
    }

    pub fn from_point(x: K, y: V) -> Self {
        Self {
            xs: vec![x],
            ys: vec![y],
        }
    }

    pub fn push_replace(&mut self, x: K, y: V) {
        match floor_and_ciel(&self.xs, x) {
            MyRes::Ok(low, high) => {
                self.xs.insert(low, x);
                self.ys.insert(high, y)
            }
            MyRes::Equal(i) => self.ys[i] = y,
            MyRes::ToLow(_) => {
                self.xs.insert(0, x);
                self.ys.insert(0, y)
            }
            MyRes::ToHigh(_) => {
                self.xs.push(x);
                self.ys.push(y);
            }
        }
    }

    pub fn push(&mut self, x: K, y: V) -> Result<(), Error> {
        if self.is_empty() {
            self.xs = vec![x];
            self.ys = vec![y];
            return Ok(());
        }
        match floor_and_ciel(&self.xs, x) {
            MyRes::Equal(_) => return Err(Error::Overwrite),
            MyRes::Ok(low, high) => {
                self.xs.insert(low, x);
                self.ys.insert(high, y)
            }
            MyRes::ToLow(_) => {
                self.xs.insert(0, x);
                self.ys.insert(0, y)
            }
            MyRes::ToHigh(_) => {
                self.xs.push(x);
                self.ys.push(y);
            }
        }
        Ok(())
    }

    pub fn upto(&self, x: K) -> (&[K], &[V]) {
        match floor_and_ciel(&self.xs, x) {
            MyRes::Ok(_, high) => (&self.xs[..high], &self.ys[..high]),
            MyRes::Equal(idx) => (&self.xs[..(idx + 1)], &self.ys[..(idx + 1)]),
            MyRes::ToLow(_) => (&[], &[]),
            MyRes::ToHigh(_) => (&self.xs, &self.ys),
        }
    }

    pub fn slices(&self) -> (&[K], &[V]) {
        (&self.xs, &self.ys)
    }

    pub fn get_pairs(&self, x: K) -> MyRes<Pair<K, V>> {
        floor_and_ciel(&self.xs, x).map(|idx| Pair(self.xs[idx], self.ys[idx]))
    }

    pub fn is_empty(&self) -> bool {
        self.xs.is_empty()
    }

    pub fn map_keys<T: Ord + Copy>(self, f: impl FnMut(K) -> T) -> XYPairs<T, V> {
        XYPairs {
            xs: self.xs.into_iter().map(f).collect(),
            ys: self.ys,
        }
    }

    pub fn map_values<T: Copy>(self, f: impl FnMut(V) -> T) -> XYPairs<K, T> {
        XYPairs {
            xs: self.xs,
            ys: self.ys.into_iter().map(f).collect(),
        }
    }
}

impl<K: Copy + Ord, V: Copy> Default for XYPairs<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MyRes<T> {
    Ok(T, T),
    Equal(T),
    ToLow(T),
    ToHigh(T),
}

impl<T> MyRes<T> {
    fn map<F: Fn(T) -> S, S>(self, f: F) -> MyRes<S> {
        match self {
            MyRes::Ok(a, b) => MyRes::Ok(f(a), f(b)),
            MyRes::Equal(a) => MyRes::Equal(f(a)),
            MyRes::ToLow(a) => MyRes::ToLow(f(a)),
            MyRes::ToHigh(a) => MyRes::ToHigh(f(a)),
        }
    }
}

fn floor_and_ciel<T: Ord>(vec: &[T], val: T) -> MyRes<usize> {
    if let Some(i) = vec.iter().position(|e| *e == val) {
        return MyRes::Equal(i);
    }
    if val <= vec[0] {
        MyRes::ToLow(0)
    } else if &val >= vec.last().unwrap() {
        MyRes::ToHigh(vec.len() - 1)
    } else {
        let mut low = 0;
        let mut high = vec.len() - 1;
        while !(low + 1 == high || low == high) {
            let current = (low + high) / 2;
            if vec[current] < val {
                low = current;
            } else {
                high = current;
            }
        }
        MyRes::Ok(low, high)
    }
}

#[derive(Debug)]
pub struct Pair<K: Copy, V: Copy>(K, V);

impl<K: Copy, V: Copy> Pair<K, V> {
    pub fn x(&self) -> K {
        self.0
    }
    pub fn y(&self) -> V {
        self.1
    }
}
