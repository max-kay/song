use crate::utils::seconds_to_samples;

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct TimeStamp {
    seconds: f64,
}

impl TimeStamp {
    pub fn to_samples(&self) -> usize {
        seconds_to_samples(self.seconds)
    }

    pub fn to_seconds(&self) -> f64 {
        self.seconds
    }

    pub fn zero() -> Self {
        Self { seconds: 0.0 }
    }

    pub fn add_seconds(&self, seconds: f64) -> Self {
        Self {
            seconds: self.seconds + seconds,
        }
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct Duration {
    seconds: f64,
}

impl Duration {
    pub fn to_samples(&self) -> usize {
        seconds_to_samples(self.seconds)
    }

    pub fn one_sec() -> Self {
        Self { seconds: 1.0 }
    }

    pub fn to_seconds(&self) -> f64 {
        self.seconds
    }
}
