use crate::utils::{samples_to_seconds, seconds_to_samples};
use std::{cell::RefCell, rc::Rc};

pub trait TimeKeeper {
    fn set_time_manager(&mut self, time_manager: Rc<RefCell<TimeManager>>);
}

#[derive(Debug, Clone)]
pub struct TimeManager {
    pub ticks_per_beat: u16,
    pub beats_per_bar: u16,
    pub beat_value: u16,
    pub bps: f64,
}

impl TimeManager {
    pub fn set_ticks_per_beat(&mut self, value: u16) {
        self.ticks_per_beat = value
    }
    pub fn set_beats_per_bar(&mut self, value: u16) {
        self.beats_per_bar = value
    }
    pub fn set_beat_value(&mut self, value: u16) {
        self.beat_value = value
    }
    pub fn set_bpm(&mut self, value: f64) {
        self.bps = value / 60.0
    }
    pub fn get_bpm(&self) -> f64 {
        self.bps * 60.0
    }
}

impl Default for TimeManager {
    fn default() -> Self {
        Self {
            ticks_per_beat: 120,
            beats_per_bar: 4,
            beat_value: 4,
            bps: 2.0,
        }
    }
}

impl TimeManager {
    fn stamp_to_ticks(&self, stamp: TimeStamp) -> u16 {
        stamp.tick + (stamp.beat + stamp.bar * self.beats_per_bar) * self.ticks_per_beat
    }

    fn ticks_to_seconds(&self, ticks: u16) -> f64 {
        ticks as f64 / self.ticks_per_beat as f64 / self.bps as f64
    }

    fn ticks_to_stamp(&self, ticks: u16) -> TimeStamp {
        TimeStamp {
            bar: ticks / (self.ticks_per_beat * self.beats_per_bar),
            beat: (ticks / self.ticks_per_beat) % self.beats_per_bar,
            tick: ticks % (self.ticks_per_beat * self.beats_per_bar),
        }
    }

    fn seconds_to_ticks(&self, seconds: f64) -> u16 {
        let ticks_per_second = self.bps / 60.0 * self.ticks_per_beat as f64;
        (seconds * ticks_per_second) as u16
    }
}

impl TimeManager {
    pub fn stamp_to_seconds(&self, time_stamp: TimeStamp) -> f64 {
        self.ticks_to_seconds(self.stamp_to_ticks(time_stamp))
    }

    pub fn stamp_to_samples(&self, time_stamp: TimeStamp) -> usize {
        seconds_to_samples(self.stamp_to_seconds(time_stamp))
    }

    pub fn seconds_to_stamp(&self, seconds: f64) -> TimeStamp {
        self.ticks_to_stamp(self.seconds_to_ticks(seconds))
    }
}

impl TimeManager {
    pub fn zero(&self) -> TimeStamp {
        TimeStamp {
            bar: 0,
            beat: 0,
            tick: 0,
        }
    }
}

impl TimeManager {
    pub fn add_seconds_to_stamp(&self, time_stamp: TimeStamp, seconds: f64) -> TimeStamp {
        self.seconds_to_stamp(seconds + self.stamp_to_seconds(time_stamp))
    }

    pub fn duration_to_seconds(&self, t0: TimeStamp, t1: TimeStamp) -> f64 {
        self.stamp_to_seconds(t1) - self.stamp_to_seconds(t0)
    }

    pub fn duration_to_samples(&self, t0: TimeStamp, t1: TimeStamp) -> usize {
        seconds_to_samples(self.duration_to_seconds(t0, t1))
    }
}

impl TimeManager {
    pub fn get_stamp_vec(&self, onset: TimeStamp, samples: usize) -> Vec<TimeStamp> {
        let onset = self.stamp_to_seconds(onset);
        let mut out = Vec::new();
        for i in 0..samples {
            out.push(self.seconds_to_stamp(samples_to_seconds(i) + onset))
        }
        out
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimeStamp {
    bar: u16,
    beat: u16,
    tick: u16,
}

impl PartialOrd for TimeStamp {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.bar.partial_cmp(&other.bar) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self.beat.partial_cmp(&other.beat) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.tick.partial_cmp(&other.tick)
    }
}

impl TimeStamp {
    pub fn zero() -> Self {
        Self {
            bar: 0,
            beat: 0,
            tick: 0,
        }
    }
}
