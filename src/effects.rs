use crate::{
    ctrl_f::{Control, ControlError},
    time::TimeStamp,
    wave::Wave,
};
use std::fmt::Debug;

pub mod delay;
pub mod reverb;
pub mod volume;

pub use delay::Delay;

trait EffMarker<W: Wave>: Effect<W> + Default {}

pub trait Effect<W: Wave>: Debug {
    fn apply(&self, wave: &mut W, time_triggered: TimeStamp);
    fn set_defaults(&mut self);
    fn on(&mut self);
    fn off(&mut self);
    fn toggle(&mut self);
}

#[derive(Debug)]
pub enum EffectPanel<W: Wave> {
    Leaf(Box<dyn Effect<W>>),
    Node(Vec<EffectPanel<W>>),
    EmptyLeaf,
}

impl<W: Wave> EffectPanel<W> {
    pub fn apply_to(&self, wave: &mut W, time_triggered: TimeStamp) {
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
