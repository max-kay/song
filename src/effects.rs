use std::marker::PhantomData;
use std::rc::Rc;

use crate::auto::Control;
use crate::time::{TimeKeeper, TimeStamp};
use crate::utils::seconds_to_samples;
use crate::wave::Wave;

pub enum EffectNode<W: Wave> {
    Parallel(Vec<Box<dyn Effect<W>>>, Box<Self>),
    Series(Box<dyn Effect<W>>, Box<Self>),
    End,
}

impl<W: Wave> EffectNode<W> {
    pub fn apply(&self, wave: &mut W, time_triggered: TimeStamp) {
        match self {
            EffectNode::Parallel(effects, node) => {
                let old_wave = wave.clone();
                wave.clear();
                for e in effects {
                    let mut applied = old_wave.clone();
                    e.apply(&mut applied, time_triggered);
                    wave.add_consuming(applied, 0)
                }
                node.apply(wave, time_triggered)
            }
            EffectNode::Series(effect, node) => {
                effect.apply(wave, time_triggered);
                node.apply(wave, time_triggered)
            }
            EffectNode::End => (),
        }
    }
}

pub trait Effect<W: Wave> {
    fn apply(&self, wave: &mut W, time_triggered: TimeStamp);
}

pub struct Delay<W: Wave> {
    phantom: PhantomData<W>,
    time_keeper: Rc<TimeKeeper>,
    gain_ctrl: Control,
    delta_t_ctrl: Control,
}

impl<W: Wave> Effect<W> for Delay<W> {
    fn apply(&self, wave: &mut W, time_triggered: TimeStamp) {
        let mut source = wave.clone();

        let mut current_time = time_triggered;
        let mut gain: f64 = self.gain_ctrl.get_value(time_triggered);
        let mut delta_t = self.delta_t_ctrl.get_value(time_triggered);
        while gain > 0.005 {
            // test this value
            source.scale(gain);
            wave.add(&source, seconds_to_samples(delta_t));
            current_time = self.time_keeper.add_seconds_to_stamp(current_time, delta_t);
            delta_t += self.delta_t_ctrl.get_value(current_time);
            gain *= self.gain_ctrl.get_value(current_time);
        }
    }
}

// impl Reverb {

//     fn comb_delay(wave: Vec<f64>, delta_ms: f64, gain: f64, loops : u8) -> Vec<f64>{ //loops 5
//         let mut out = Vec::with_capacity(wave.len() + seconds_to_samples(delta_ms * 1000.0) * loops as usize);
//         for i in 0..loops{
//             add_with_index(out, wave.into_iter().map(|x| x*gain.powi(i)).collect(), round(delta_ms*SAMPLE_RATE/1000)*(i+1));}
//         return out}

//     fn all_pass_delay(arr: Vec<f64>, delta_ms: f64, gain: f64, loops: u8) -> Vec<f64>: //loops 20
//         out = np.zeros(SAMPLE_RATE)
//         for i in range(loops):
//             out = af.add_with_index(out, arr*(gain**i), round(delta_ms*SAMPLE_RATE/1000)*(i+1))
//         return af.add_dif_len(out*(1-gain**2), -gain*arr)

//     def reverb(arr: Vec<f64>, wet=1, dry=1) -> Vec<f64>:
//         a = .7
//         parallel = [(a+.042, 4.799), (a+.033, 4.999), (a+.015, 5.399), (a-0.003, 5.801)]
//         series = [(a, 1.051), (a, 0.337)]
//         out = np.zeros(SAMPLE_RATE)
//         for gain, delta in parallel:
//             out = af.add_dif_len(out, comb_delay(arr, gain, delta))
//         out = out/5000
//         for gain, delta in series:
//             out = all_pass_delay(out, gain, delta)
//         return af.add_dif_len(out * wet, arr * dry)/3
// }
