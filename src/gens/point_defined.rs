use serde::{Deserialize, Serialize};

use crate::{
    globals::TIME_MANAGER,
    time::ClockTick,
    utils::{self, MyRes, XYPairs},
    Error,
};

use super::{GenId, Generator};

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub enum Interpolation {
    #[default]
    Step,
    Linear,
    Smooth,
}

impl Interpolation {
    pub fn interpolate(&self, val1: f32, val2: f32, progress: f32) -> f32 {
        match self {
            Interpolation::Step => val1,
            Interpolation::Linear => (val2 - val1) * progress + val1,
            Interpolation::Smooth => (val2 - val1) * utils::smooth_step(progress) + val1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointDefined {
    id: GenId,
    points: XYPairs<ClockTick, f32>,
    interpolation: Interpolation,
}

impl PointDefined {
    pub fn new(ticks: Vec<ClockTick>, vals: Vec<f32>, interpolation: Interpolation) -> Self {
        debug_assert!(vals.iter().all(|val| (0.0..=1.0).contains(val)));
        Self {
            id: GenId::Unbound,
            points: XYPairs::from_vecs(ticks, vals),
            interpolation,
        }
    }

    pub fn from_xy_pairs(pairs: XYPairs<ClockTick, f32>, interpolation: Interpolation) -> Self {
        Self {
            id: GenId::Unbound,
            points: pairs,
            interpolation,
        }
    }

    pub fn wrap(self) -> Generator {
        Generator::PointDefined(self)
    }

    pub fn w_default() -> Generator {
        Generator::PointDefined(Self::default())
    }

    pub fn new_val(val: f32) -> Result<Self, Error> {
        debug_assert!((0.0..=1.0).contains(&val));
        Ok(Self {
            id: GenId::Unbound,
            points: XYPairs::from_point(ClockTick::abs_zero(), val),
            interpolation: Default::default(),
        })
    }

    pub fn w_val(val: f32) -> Result<Generator, Error> {
        Ok(Generator::PointDefined(Self::new_val(val)?))
    }

    pub(crate) fn set_id(&mut self, id: GenId) {
        self.id = id
    }
}

impl PointDefined {
    // TODO Performance
    pub fn get_val(&self, tick: ClockTick) -> f32 {
        match self.points.get_pairs(tick) {
            MyRes::Ok(pair_1, pair_2) => self.interpolation.interpolate(
                pair_1.y(),
                pair_2.y(),
                (tick - pair_1.x()).f32() / (pair_2.x() - pair_1.x()).f32(),
            ),
            MyRes::Equal(pair) => pair.y(),
            MyRes::ToLow(pair) => pair.y(),
            MyRes::ToHigh(pair) => pair.y(),
        }
    }

    pub fn get_vec(&self, onset: ClockTick, samples: usize) -> Vec<f32> {
        let time_stamps = TIME_MANAGER.read().unwrap().get_tick_vec(onset, samples);
        let mut out = Vec::with_capacity(samples);
        for t in time_stamps {
            out.push(self.get_val(t))
        }
        out
    }
}
impl Default for PointDefined {
    fn default() -> Self {
        Self {
            id: GenId::Unbound,
            points: XYPairs::from_point(ClockTick::abs_zero(), 1.0),
            interpolation: Default::default(),
        }
    }
}
