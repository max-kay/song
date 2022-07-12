use crate::{auto::Control, time, wave};
use std::{cell::RefCell, collections::HashMap, fmt, iter::zip, rc::Rc};

pub mod delay;
pub mod reverb;

pub use delay::Delay;

#[derive(Debug, Clone)]
pub struct ControlError;
impl fmt::Display for ControlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "mismatch between an EffectNode and an CtrlPanel occured")
    }
}

impl std::error::Error for ControlError {}

pub enum CtrlPanel<'a> {
    Map(HashMap<&'a str, Control>),
    Node(Vec<CtrlPanel<'a>>),
    Bypass,
}

impl CtrlPanel<'_> {
    pub fn map(&self) -> Result<&HashMap<&str, Control>, ControlError> {
        match self {
            CtrlPanel::Map(map) => Ok(map),
            _ => Err(ControlError),
        }
    }

    pub fn node(&self) -> Result<&Vec<CtrlPanel>, ControlError> {
        match self {
            CtrlPanel::Node(vec) => Ok(vec),
            _ => Err(ControlError),
        }
    }
}
impl time::TimeKeeper for CtrlPanel<'_> {
    fn set_time_manager(&mut self, time_manager: Rc<RefCell<time::TimeManager>>) {
        match self {
            CtrlPanel::Map(map) => {
                for entry in map.values_mut() {
                    entry.set_time_manager(Rc::clone(&time_manager))
                }
            }
            CtrlPanel::Node(vec) => {
                for panel in vec {
                    panel.set_time_manager(Rc::clone(&time_manager))
                }
            }
            CtrlPanel::Bypass => (),
        }
    }
}

pub enum EffectNode<W: wave::Wave> {
    Effect(Box<dyn Effect<W>>),
    Node(Vec<EffectNode<W>>),
    Bypass,
}

impl<W: wave::Wave> time::TimeKeeper for EffectNode<W> {
    fn set_time_manager(&mut self, time_manager: Rc<RefCell<time::TimeManager>>) {
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

impl<W: wave::Wave> EffectNode<W> {
    pub fn apply(&self, wave: &mut W, control_panel: &CtrlPanel, time_triggered: time::TimeStamp) {
        match self {
            EffectNode::Effect(effect) => {
                let ctrl_map = control_panel
                    .map()
                    .expect("mismatch between effectnode and effectcontrol");
                effect.apply(wave, ctrl_map, time_triggered)
            }
            EffectNode::Node(nodes) => {
                let ctrl_vec = control_panel
                    .node()
                    .expect("mismatch between effectnode and effectcontrol");
                let original = wave.clone();
                wave.clear();
                for (node, ctrl) in zip(nodes, ctrl_vec) {
                    let mut this_wave = original.clone();
                    node.apply(&mut this_wave, ctrl, time_triggered);
                    wave.add_consuming(this_wave, 0)
                }
            }
            EffectNode::Bypass => (),
        }
    }
    pub fn generate_default_controls(&self) -> CtrlPanel {
        match self {
            EffectNode::Effect(effect) => CtrlPanel::Map(effect.default_controls()),
            EffectNode::Node(nodes) => {
                let mut controls = Vec::new();
                for node in nodes {
                    controls.push(node.generate_default_controls());
                }
                CtrlPanel::Node(controls)
            }
            EffectNode::Bypass => CtrlPanel::Bypass,
        }
    }
}

pub trait Effect<W: wave::Wave>: time::TimeKeeper {
    fn apply(
        &self,
        wave: &mut W,
        controls: &HashMap<&str, Control>,
        time_triggered: time::TimeStamp,
    );
    fn controls(&self) -> Vec<&str>;
    fn default_controls(&self) -> HashMap<&str, Control>;
    fn on(&mut self);
    fn off(&mut self);
    fn toggle(&mut self);
}
