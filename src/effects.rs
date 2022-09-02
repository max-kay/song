use crate::{time::TimeStamp, wave::Wave};
use std::fmt::Debug;

pub mod delay;
pub mod reverb;
pub mod volume;

pub use delay::Delay;

trait EffMarker: Effect + Default {}

pub trait Effect: Debug {
    fn apply(&self, wave: &mut Wave, time_triggered: TimeStamp);
    fn set_defaults(&mut self);
    fn on(&mut self);
    fn off(&mut self);
    fn toggle(&mut self);
}

#[derive(Debug)]
pub enum EffectPanel {
    Leaf(Box<dyn Effect>),
    Node(Vec<EffectPanel>),
    EmptyLeaf,
}

impl EffectPanel {
    pub fn apply_to(&self, wave: &mut Wave, time_triggered: TimeStamp) {
        match self {
            EffectPanel::Leaf(eff) => eff.apply(wave, time_triggered),
            EffectPanel::Node(nodes) => {
                let original = wave.clone();
                wave.clear();
                for node in nodes {
                    let mut this_wave = original.clone();
                    node.apply_to(&mut this_wave, time_triggered);
                    wave.add_consuming(this_wave, 0)
                }
            }
            EffectPanel::EmptyLeaf => (),
        }
    }
}
