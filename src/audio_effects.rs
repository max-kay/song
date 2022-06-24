use crate::automation::{AutomationManager, ValAndCh};
use crate::utils::{add_from_index_by_ref, seconds_to_samples};

pub trait Effect {
    fn apply(&self, wave: &mut Vec<f64>, automation: &AutomationManager);
}

pub struct Delay {
    pub time_delta: ValAndCh,
    pub gain: ValAndCh,
}

impl Effect for Delay {
    fn apply(&self, wave: &mut Vec<f64>, automation: &AutomationManager) {
        let mut source = wave.clone();
        let mut gain: f64 = self.gain.get_value(automation, 0.0);
        let mut offset = self.time_delta.get_value(automation, 0.0);
        while gain > 0.005 {
            //test this value
            source = source.into_iter().map(|x| x * gain).collect();
            add_from_index_by_ref(wave, &source, seconds_to_samples(offset));
            offset += self.time_delta.get_value(automation, offset);
            gain *= self.gain.get_value(automation, offset);
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
