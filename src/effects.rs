use crate::{
    control::{Control, ControlError, FunctionKeeper},
    ctrl_f::IdMap,
    time::{TimeKeeper, TimeManager, TimeStamp},
    wave::Wave,
};
use std::{cell::RefCell, fmt::Debug, rc::Rc};

pub mod delay;
pub mod reverb;
pub mod volume;

pub use delay::Delay;

trait EffMarker<W: Wave>: Effect<W> + Default {}

pub trait Effect<W: Wave>: Debug + TimeKeeper + FunctionKeeper {
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

impl<W: Wave> TimeKeeper for EffectPanel<W> {
    fn set_time_manager(&mut self, time_manager: Rc<RefCell<TimeManager>>) {
        match self {
            EffectPanel::Leaf(eff) => eff.set_time_manager(time_manager),
            EffectPanel::Node(vec) => {
                for node in vec {
                    node.set_time_manager(Rc::clone(&time_manager))
                }
            }
            EffectPanel::EmptyLeaf => (),
        }
    }
}

impl<W: Wave> FunctionKeeper for EffectPanel<W> {
    fn heal_sources(&mut self, id_map: &IdMap) -> Result<(), ControlError> {
        match self {
            EffectPanel::Leaf(eff) => eff
                .heal_sources(id_map)
                .map_err(|err| err.push_location("EffectPanel::Leaf")),
            EffectPanel::Node(vec) => {
                for node in vec {
                    node.heal_sources(id_map)
                        .map_err(|err| err.push_location("EffectPanel::Node"))?;
                }
                Ok(())
            }
            EffectPanel::EmptyLeaf => Ok(()),
        }
    }

    fn test_sources(&self) -> Result<(), ControlError> {
        match self {
            EffectPanel::Leaf(eff) => eff
                .test_sources()
                .map_err(|err| err.push_location("EffectPanel::Leaf")),
            EffectPanel::Node(vec) => {
                for node in vec {
                    node.test_sources()
                        .map_err(|err| err.push_location("EffectPanel::Node"))?;
                }
                Ok(())
            }
            EffectPanel::EmptyLeaf => Ok(()),
        }
    }

    fn set_ids(&mut self) {
        match self {
            EffectPanel::Leaf(eff) => eff.set_ids(),
            EffectPanel::Node(vec) => {
                for node in vec {
                    node.set_ids()
                }
            }
            EffectPanel::EmptyLeaf => (),
        }
    }

    fn get_ids(&self) -> Vec<usize> {
        match self {
            EffectPanel::Leaf(eff) => eff.get_ids(),
            EffectPanel::Node(vec) => {
                let mut ids = Vec::new();
                for panel in vec {
                    ids.append(&mut panel.get_ids())
                }
                ids
            }
            EffectPanel::EmptyLeaf => Vec::new(),
        }
    }
}
