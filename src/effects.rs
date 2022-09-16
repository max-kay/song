use crate::{time::ClockTick, wave::Wave};
use std::fmt::Debug;

pub mod delay;
pub mod reverb;
pub mod volume;

pub use delay::Delay;
use serde::{Deserialize, Serialize};

use self::volume::Volume;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Effect {
    Delay(Delay),
    Volume(Volume),
}

impl Effect {
    pub fn apply(&self, wave: &mut Wave, time_triggered: ClockTick) {
        match self {
            Effect::Delay(eff) => eff.apply(wave, time_triggered),
            Effect::Volume(eff) => eff.apply(wave, time_triggered),
        }
    }

    pub fn set_defaults(&mut self) {
        match self {
            Effect::Delay(eff) => eff.set_defaults(),
            Effect::Volume(eff) => eff.set_defaults(),
        }
    }

    pub fn on(&mut self) {
        match self {
            Effect::Delay(eff) => eff.on(),
            Effect::Volume(eff) => eff.on(),
        }
    }

    pub fn off(&mut self) {
        match self {
            Effect::Delay(eff) => eff.off(),
            Effect::Volume(eff) => eff.off(),
        }
    }

    pub fn toggle(&mut self) {
        match self {
            Effect::Delay(eff) => eff.toggle(),
            Effect::Volume(eff) => eff.toggle(),
        }
    }
}

impl Effect {
    pub fn extract(&self) -> Self {
        match self {
            Effect::Delay(eff) => Effect::Delay(eff.extract()),
            Effect::Volume(eff) => Effect::Volume(eff.extract()),
        }
    }

    pub fn set_id(&mut self, track_id: u8) {
        match self {
            Effect::Delay(eff) => eff.set_id(track_id),
            Effect::Volume(eff) => eff.set_id(track_id),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffectPanel {
    Leaf(Effect),
    Node(Vec<EffectPanel>),
    EmptyLeaf,
}

impl EffectPanel {
    pub fn apply_to(&self, wave: &mut Wave, time_triggered: ClockTick) {
        match self {
            EffectPanel::Leaf(eff) => eff.apply(wave, time_triggered),
            EffectPanel::Node(nodes) => {
                let original = wave.clone();
                wave.clear();
                for node in nodes {
                    let mut this_wave = original.clone();
                    node.apply_to(&mut this_wave, time_triggered);
                    wave.add(&this_wave, 0)
                }
            }
            EffectPanel::EmptyLeaf => (),
        }
    }

    pub fn extract(&self) -> Self {
        match self {
            EffectPanel::Leaf(eff) => Self::Leaf(eff.extract()),
            EffectPanel::Node(vec) => Self::Node(vec.iter().map(|panel| panel.extract()).collect()),
            EffectPanel::EmptyLeaf => EffectPanel::EmptyLeaf,
        }
    }

    pub fn set_id(&mut self, track_id: u8) {
        match self {
            EffectPanel::Leaf(eff) => eff.set_id(track_id),
            EffectPanel::Node(vec) => vec.iter_mut().for_each(|panel| panel.set_id(track_id)),
            EffectPanel::EmptyLeaf => (),
        }
    }
}
