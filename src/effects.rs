use std::marker::PhantomData;
use std::rc::Rc;

use crate::auto::Control;
use crate::time;
use crate::utils::seconds_to_samples;
use crate::wave;

pub mod delay;
pub mod reverb;

pub enum EffectNode<W: wave::Wave> {
    Effect(Box<dyn Effect<W>>),
    Node(Vec<EffectNode<W>>),
    Bypass,
}

impl<W: wave::Wave> EffectNode<W> {
    pub fn apply(&self, wave: &mut W, time_triggered: time::TimeStamp) {
        match self {
            EffectNode::Effect(effect) => effect.apply(wave, time_triggered),
            EffectNode::Node(nodes) => {
                let original = wave.clone();
                wave.clear();
                for node in nodes{
                    let mut this_wave = original.clone();
                    node.apply(&mut this_wave, time_triggered);
                    wave.add_consuming(this_wave, 0)

                }
            },
            EffectNode::Bypass => (),
        }
    }
}

pub trait Effect<W: wave::Wave> {
    fn apply(&self, wave: &mut W, time_triggered: time::TimeStamp);
}
