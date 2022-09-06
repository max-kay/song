use std::vec;

use serde::{Deserialize, Serialize};

use crate::{
    network::{self, Reciever, Transform},
    time::TimeStamp,
    utils,
    utils::oscs::Oscillator,
    wave::Wave,
    Error,
};

use super::PITCH_RECIEVER;

const WEIGHT_RECIEVER: Reciever = Reciever::new(1.0, (0.0, 5.0), Transform::Linear);
const MODULATION_RECIEVER: Reciever = Reciever::new(0.5, (0.0, 1.0), Transform::Linear);
const PITCH_OFFSET_RECIEVER: Reciever = Reciever::new(0.0, (-4800.0, 4800.0), Transform::Linear);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscPanel {
    oscillators: Vec<Oscillator>,
    modulation: Vec<Reciever>,
    weights: Vec<Reciever>,
    pitch_offsets: Vec<Reciever>,
}

impl OscPanel {
    pub fn from_oscs(
        oscillators: Vec<Oscillator>,
        weights: Option<Vec<f64>>,
        modulation: Option<Vec<f64>>,
    ) -> Result<Self, Error> {
        let len = oscillators.len();
        Ok(Self {
            pitch_offsets: vec![PITCH_OFFSET_RECIEVER; oscillators.len()],
            oscillators,
            weights: network::vec_or_none(weights, len, WEIGHT_RECIEVER)?,
            modulation: network::vec_or_none(modulation, len, MODULATION_RECIEVER)?,
        })
    }
}

impl Default for OscPanel {
    fn default() -> Self {
        Self {
            oscillators: vec![Oscillator::default()],
            weights: vec![WEIGHT_RECIEVER],
            pitch_offsets: vec![PITCH_OFFSET_RECIEVER],
            modulation: vec![MODULATION_RECIEVER],
        }
    }
}

impl OscPanel {
    pub fn play(&self, freq: Vec<f64>, start: TimeStamp, samples: usize) -> Wave {
        let mut wave = vec![0.0; samples];

        for (((osc, weigth), modulation), offset) in self
            .oscillators
            .iter()
            .zip(&self.weights)
            .zip(&self.modulation)
            .zip(&self.pitch_offsets)
        {
            let freq = offset
                .get_vec(start, samples)
                .into_iter()
                .zip(&freq)
                .map(|(x, y)| y * 2_f64.powf(x / 1200.0))
                .collect();

            let modulation = modulation.get_vec(start, samples);
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
        self.pitch_offsets.push(PITCH_RECIEVER);
        self.weights.push(WEIGHT_RECIEVER);
    }
}
