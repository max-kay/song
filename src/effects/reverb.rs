// impl Reverb {

//     fn comb_delay(wave: Vec<f32>, delta_ms: f32, gain: f32, loops : u8) -> Vec<f32>{ //loops 5
//         let mut out = Vec::with_capacity(wave.len() + seconds_to_samples(delta_ms * 1000.0) * loops as usize);
//         for i in 0..loops{
//             add_with_index(out, wave.into_iter().map(|x| x*gain.powi(i)).collect(), round(delta_ms*SAMPLE_RATE/1000)*(i+1));}
//         return out}

//     fn all_pass_delay(arr: Vec<f32>, delta_ms: f32, gain: f32, loops: u8) -> Vec<f32>: //loops 20
//         out = np.zeros(SAMPLE_RATE)
//         for i in range(loops):
//             out = af.add_with_index(out, arr*(gain**i), round(delta_ms*SAMPLE_RATE/1000)*(i+1))
//         return af.add_dif_len(out*(1-gain**2), -gain*arr)

//     def reverb(arr: Vec<f32>, wet=1, dry=1) -> Vec<f32>:
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
