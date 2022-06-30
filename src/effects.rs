use crate::auto::ValAndCh;
use crate::time::TimeStamp;
use crate::utils::{add_from_index_by_ref, seconds_to_samples};

pub trait Effect {
    fn apply(&self, wave: &mut Vec<f64>, time_functions: Vec<ValAndCh>, time_triggerd: TimeStamp);
    fn number_of_controls(&self) -> usize;
}

pub struct Delay {}

impl Effect for Delay {
    fn apply(&self, wave: &mut Vec<f64>, time_functions: Vec<ValAndCh>, time_triggered: TimeStamp) {
        let gain_ch = &time_functions[0];
        let delta_t_ch = &time_functions[1];

        let mut source = wave.clone();

        let mut current_time = time_triggered;
        let mut gain: f64 = gain_ch.get_value(time_triggered);
        let mut delta_t = delta_t_ch.get_value(time_triggered);
        while gain > 0.005 {
            // test this value
            source = source.into_iter().map(|x| x * gain).collect();
            add_from_index_by_ref(wave, &source, seconds_to_samples(delta_t));
            current_time = current_time.add_seconds(delta_t);
            delta_t += delta_t_ch.get_value(current_time);
            gain *= gain_ch.get_value(current_time);
        }
    }

    fn number_of_controls(&self) -> usize {
        2
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
