use crate::{
    auto::Control,
    time::{TimeKeeper, TimeManager, TimeStamp},
    wave::Wave,
};
use std::{cell::RefCell, fmt::Debug, rc::Rc};

pub mod delay;
pub mod reverb;
pub mod volume;

pub use delay::Delay;

trait EffMarker<W: Wave>: Effect<W> + Default {}
trait EffCtrlMarker: Controler + Default {
    
}

pub trait Effect<W: Wave>: TimeKeeper + Debug {
    fn apply(&self, wave: &mut W, time_triggered: TimeStamp);
    fn set_defaults(&mut self);
    fn get_controls(&mut self) -> &mut dyn Controler;
    fn on(&mut self);
    fn off(&mut self);
    fn toggle(&mut self);
}

pub trait Controler: TimeKeeper {
    fn set_defaults(&mut self);
}

#[derive(Debug)]
pub enum EffectNode<W: Wave> {
    Effect(Box<dyn Effect<W>>),
    Node(Vec<EffectNode<W>>),
    Bypass,
}

impl<W: Wave> TimeKeeper for EffectNode<W> {
    fn set_time_manager(&mut self, time_manager: Rc<RefCell<TimeManager>>) {
        match self {
            EffectNode::Effect(eff) => eff.set_time_manager(time_manager),
            EffectNode::Node(vec) => {
                for node in vec {
                    node.set_time_manager(Rc::clone(&time_manager))
                }
            }
            EffectNode::Bypass => (),
        }
    }
}

impl<W: Wave> EffectNode<W> {
    pub fn apply(&self, wave: &mut W, time_triggered: TimeStamp) {
        match self {
            EffectNode::Effect(eff) => eff.apply(wave, time_triggered),
            EffectNode::Node(nodes) => {
                let original = wave.clone();
                wave.clear();
                for node in nodes {
                    let mut this_wave = original.clone();
                    node.apply(&mut this_wave, time_triggered);
                    wave.add_consuming(this_wave, 0)
                }
            }
            EffectNode::Bypass => (),
        }
    }
}
