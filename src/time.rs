use crate::utils::seconds_to_samples;

pub struct TimeKeeper {
    ticks_per_beat: u16,
    beats_per_bar: u16,
    beat_value: u16,
    bpm: f64,
}

impl Default for TimeKeeper{
    fn default() -> Self {
        Self { ticks_per_beat: 120, beats_per_bar: 4, beat_value: 4, bpm: 120.0 }
    }
}

impl TimeKeeper {
    pub fn stamp_to_seconds(&self, time_stamp: TimeStamp) -> f64 {
        time_stamp.bar as f64 * self.beats_per_bar as f64 / self.bpm * 60.0
            + time_stamp.beat as f64 / self.bpm * 60.0
            + time_stamp.tick as f64 / self.bpm * 60.0 / self.ticks_per_beat as f64
    }

    pub fn stamp_to_samples(&self, time_stamp: TimeStamp) -> usize {
        seconds_to_samples(self.stamp_to_seconds(time_stamp))
    }

    pub fn add_seconds_to_stamp(&self, time_stamp: TimeStamp, seconds: f64) -> TimeStamp {
        self.add_duration_to_stamp(time_stamp, self.duration_from_seconds(seconds, time_stamp))
    }

    pub fn add_duration_to_stamp(&self, time_stamp: TimeStamp, duration: Duration) -> TimeStamp{
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

    pub fn duration_to_seconds(&self, duration: Duration, start: TimeStamp)-> f64{
        duration.bars as f64 * self.beats_per_bar as f64 / self.bpm * 60.0
            + duration.beats as f64 / self.bpm * 60.0
            + duration.ticks as f64 / self.bpm * 60.0 / self.ticks_per_beat as f64
    }

    fn seconds_to_ticks(&self, seconds: f64, start: TimeStamp) -> u16 {
        let ticks_per_second = self.bpm / 60.0 * self.ticks_per_beat as f64;
        (seconds * ticks_per_second) as u16
    }

    pub fn duration_from_seconds(&self, seconds: f64, start: TimeStamp) -> Duration{
        let temp_ticks = self.seconds_to_ticks(seconds, start);
        let bars = temp_ticks/(self.ticks_per_beat  * self.beats_per_bar) ;
        let beats = (temp_ticks / self.ticks_per_beat ) % self.ticks_per_beat;
        let ticks = temp_ticks % self.ticks_per_beat;
        Duration { bars , beats, ticks}
    }

    pub fn duration_to_samples(&self, duration: Duration, start: TimeStamp)-> usize{
        seconds_to_samples(self.duration_to_seconds(duration, start))
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct TimeStamp {
    bar: u16,
    beat: u16,
    tick: u16,
}

impl TimeStamp{
    pub fn zero()-> Self{
        Self { bar: 0, beat: 0, tick: 0 }
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct Duration {
    bars: u16,
    beats: u16,
    ticks: u16,
}
