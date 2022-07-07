use crate::utils::{samples_to_seconds, seconds_to_samples};

#[derive(Debug, Clone)]
pub struct TimeKeeper {
    pub ticks_per_beat: u16,
    pub beats_per_bar: u16,
    pub beat_value: u16,
    pub bpm: f64,
}

impl TimeKeeper {
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
        self.bpm = value
    }
}

impl Default for TimeKeeper {
    fn default() -> Self {
        Self {
            ticks_per_beat: 120,
            beats_per_bar: 4,
            beat_value: 4,
            bpm: 120.0,
        }
    }
}

impl TimeKeeper {
    pub fn stamp_to_seconds(&self, time_stamp: TimeStamp) -> f64 {
        time_stamp.bar as f64 * self.beats_per_bar as f64 / self.bpm * 60.0
            + time_stamp.beat as f64 / self.bpm * 60.0
            + time_stamp.tick as f64 / self.bpm * 60.0 / self.ticks_per_beat as f64
    }

    pub fn duration_from_stamps(&self, t0: TimeStamp, t1: TimeStamp) -> Duration {
        let mut carry_beat = 0;
        let ticks = match t1.tick.checked_sub(t0.tick) {
            Some(val) => val,
            None => {
                carry_beat = 1;
                t1.tick + self.ticks_per_beat - t0.tick
            }
        };
        let mut carry_bar = 0;
        let beats = match t1.beat.checked_sub(t0.beat + carry_beat) {
            Some(val) => val,
            None => {
                carry_bar = 1;
                t1.tick + self.beats_per_bar - t0.beat
            }
        };
        let bars = t1.bar - t0.bar + carry_bar;
        Duration { bars, beats, ticks }
    }

    pub fn stamp_to_samples(&self, time_stamp: TimeStamp) -> usize {
        seconds_to_samples(self.stamp_to_seconds(time_stamp))
    }

    pub fn add_seconds_to_stamp(&self, time_stamp: TimeStamp, seconds: f64) -> TimeStamp {
        self.add_duration_to_stamp(time_stamp, self.duration_from_seconds(seconds, time_stamp))
    }

    pub fn seconds_to_stamp(&self, seconds: f64) -> TimeStamp {
        let start = self.zero();
        let temp_tick = self.seconds_to_ticks(seconds, start);
        let bar = temp_tick / (self.ticks_per_beat * self.beats_per_bar);
        let beat = (temp_tick / self.ticks_per_beat) % self.ticks_per_beat;
        let tick = temp_tick % self.ticks_per_beat;
        TimeStamp { bar, beat, tick }
    }

    pub fn add_duration_to_stamp(&self, time_stamp: TimeStamp, duration: Duration) -> TimeStamp {
        let tick = (time_stamp.tick + duration.ticks) % self.ticks_per_beat;
        let carry_beat = (time_stamp.tick + duration.ticks) / self.ticks_per_beat;
        let beat = (time_stamp.beat + duration.beats + carry_beat) % self.beats_per_bar;
        let carry_bar = (time_stamp.beat + duration.beats + carry_beat) / self.beats_per_bar;
        let bar = time_stamp.bar + duration.bars + carry_bar;
        TimeStamp { bar, beat, tick }
    }

    pub fn zero(&self) -> TimeStamp {
        TimeStamp {
            bar: 0,
            beat: 0,
            tick: 0,
        }
    }

    pub fn duration_to_seconds(&self, duration: Duration, start: TimeStamp) -> f64 {
        duration.bars as f64 * self.beats_per_bar as f64 / self.bpm * 60.0
            + duration.beats as f64 / self.bpm * 60.0
            + duration.ticks as f64 / self.bpm * 60.0 / self.ticks_per_beat as f64
    }

    fn seconds_to_ticks(&self, seconds: f64, start: TimeStamp) -> u16 {
        let ticks_per_second = self.bpm / 60.0 * self.ticks_per_beat as f64;
        (seconds * ticks_per_second) as u16
    }

    pub fn duration_from_seconds(&self, seconds: f64, start: TimeStamp) -> Duration {
        let temp_ticks = self.seconds_to_ticks(seconds, start);
        let bars = temp_ticks / (self.ticks_per_beat * self.beats_per_bar);
        let beats = (temp_ticks / self.ticks_per_beat) % self.ticks_per_beat;
        let ticks = temp_ticks % self.ticks_per_beat;
        Duration { bars, beats, ticks }
    }

    pub fn duration_to_samples(&self, duration: Duration, start: TimeStamp) -> usize {
        seconds_to_samples(self.duration_to_seconds(duration, start))
    }

    pub fn get_stamp_vec(&self, onset: TimeStamp, samples: usize) -> Vec<TimeStamp> {
        let mut out = Vec::new();
        for i in 0..samples {
            out.push(self.seconds_to_stamp(samples_to_seconds(i)))
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

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Duration {
    bars: u16,
    beats: u16,
    ticks: u16,
}
