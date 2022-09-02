use once_cell::sync::Lazy;

use crate::{
    network::{Reciever, Transform},
    time::TimeStamp,
    utils,
    utils::oscs::Oscillator,
    wave::Wave,
};


use super::PITCH_RECIEVER;

static WEIGHT_RECIEVER: Lazy<Reciever> =
    Lazy::new(|| Reciever::new(1.0, (0.0, 5.0), Transform::Linear));
static PITCH_OFFSET_RECIEVER: Lazy<Reciever> =
    Lazy::new(|| Reciever::new(0.0, (-4800.0, 4800.0), Transform::Linear));

#[derive(Debug)]
pub struct OscPanel {

    oscillators: Vec<Oscillator>,
    weights: Vec<Reciever>,
    pitch_offsets: Vec<Reciever>,
}

impl Default for OscPanel  {
    fn default() -> Self {
        Self {
            oscillators: vec![Oscillator::default()],
            weights: vec![WEIGHT_RECIEVER.clone()],
            pitch_offsets: vec![PITCH_OFFSET_RECIEVER.clone()],
        }
    }
}

impl OscPanel  {
    pub fn play(
        &self,
        freq: Vec<f64>,
        modulation: Vec<f64>,
        start: TimeStamp,
        samples: usize,
    ) -> Wave {
        let mut wave = vec![0.0; samples];

        for ((osc, weigth), offset) in self
            .oscillators
            .iter()
            .zip(&self.weights)
            .zip(&self.pitch_offsets)
        {
            let freq = offset
                .get_vec(start, samples)
                .into_iter()
                .zip(&freq)
                .map(|(x, y)| y * 2_f64.powf(x / 1200.0))
                .collect();

            let new_wave = osc
                .play(&freq, &modulation, samples)
                .into_iter()
                .zip(weigth.get_vec(start, samples))
                .map(|(x, y)| x * y)
                .collect();

            utils::add_elementwise(&mut wave, new_wave)
        }
        Wave::from_vec(wave)
    }

    pub fn add_osc(&mut self, oscillator: Oscillator) {
        self.oscillators.push(oscillator);
        self.pitch_offsets.push(PITCH_RECIEVER.clone());
        self.weights.push(WEIGHT_RECIEVER.clone());
    }
}
