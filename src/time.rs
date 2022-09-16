use derive_more::{Add, Sub};
use serde::{Deserialize, Serialize};

use crate::{
    io::TimeDecoder,
    utils::{self, XYPairs},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeManager {
    s_per_tick: XYPairs<ClockTick, f32>,
}

// signatures: XYPairs<ClockTick, Signature>, // stamp musical interpretation conversion

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Signature {
    ticks_per_beat: u32,
    beats_per_bar: u8,
    beat_value: u8,
    subdivision: Option<SubDiv>,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
struct SubDiv {
    arr: [u8; 20],
}

impl Default for TimeManager {
    fn default() -> Self {
        Self {
            s_per_tick: XYPairs::from_point(ClockTick::abs_zero(), 0.00001),
        } // TODO better value
    }
}

impl TimeManager {
    pub fn tick_to_second(&self, tick: ClockTick) -> f32 {
        let (start_ticks, tempos) = self.s_per_tick.upto(tick);
        let mut sum = 0.0;
        for i in 0..(start_ticks.len() - 1) {
            sum += (start_ticks[i + 1] - start_ticks[i]).f32() * tempos[i]
        }
        sum + (tick - *start_ticks.last().unwrap()).f32() * tempos.last().unwrap()
    }

    pub fn tick_to_sample(&self, tick: ClockTick) -> usize {
        utils::seconds_to_samples(self.tick_to_second(tick))
    }

    pub fn second_to_tick(&self, mut second: f32) -> ClockTick {
        let (start_ticks, tempos) = self.s_per_tick.slices();
        for i in 0..start_ticks.len() - 1 {
            let section_seconds = (start_ticks[i + 1] - start_ticks[i]).f32() * tempos[i];
            if section_seconds > second {
                return start_ticks[i] + ClockTick((second / tempos[i]) as u32);
            }
            second -= section_seconds
        }
        *start_ticks.last().unwrap() + ClockTick((second / tempos.last().unwrap()) as u32)
    }

    pub fn sample_to_tick(&self, sample: usize) -> ClockTick {
        self.second_to_tick(utils::samples_to_seconds(sample))
    }

    pub fn duration_to_seconds(&self, start: ClockTick, end: ClockTick) -> f32 {
        self.tick_to_second(end) - self.tick_to_second(start)
    }

    pub fn duration_to_samples(&self, start: ClockTick, end: ClockTick) -> usize {
        utils::seconds_to_samples(self.duration_to_seconds(start, end))
    }

    pub fn add_seconds_to_stamp(&self, tick: ClockTick, seconds: f32) -> ClockTick {
        self.second_to_tick(self.tick_to_second(tick) + seconds)
    }

    pub fn abs_start(&self) -> ClockTick {
        ClockTick(0)
    }

    pub fn get_tick_vec(&self, tick: ClockTick, samples: usize) -> Vec<ClockTick> {
        (0..samples)
            .into_iter()
            .map(|sample| self.sample_to_tick(sample) + tick)
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Add, Sub, Serialize, Deserialize)]
pub struct ClockTick(u32);

impl ClockTick {
    pub(crate) fn new(tick: u32) -> Self {
        Self(tick)
    }

    pub fn f32(&self) -> f32 {
        self.0 as f32
    }

    pub fn abs_zero() -> Self {
        Self(0)
    }
}

impl From<TimeDecoder> for TimeManager {
    fn from(decoder: TimeDecoder) -> Self {
        Self {
            s_per_tick: decoder
                .mus_per_beat
                .clone()
                .map_keys(ClockTick::new)
                .map_values(|y| decoder.convert_mus_beat_to_s_tick(&y)),
        }
    }
}
