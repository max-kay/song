use std::vec;

use serde::{Deserialize, Serialize};

use crate::{
    network::{self, Receiver, Transform},
    time::ClockTick,
    utils,
    utils::oscs::Oscillator,
    wave::Wave,
    Error,
};

use super::PITCH_RECEIVER;

const WEIGHT_RECEIVER: Receiver = Receiver::new(1.0, (0.0, 5.0), Transform::Linear);
const MODULATION_RECEIVER: Receiver = Receiver::new(0.5, (0.0, 1.0), Transform::Linear);
const PITCH_OFFSET_RECEIVER: Receiver = Receiver::new(0.0, (-4800.0, 4800.0), Transform::Linear);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscPanel {
    oscillators: Vec<Oscillator>,
    modulation: Vec<Receiver>,
    weights: Vec<Receiver>,
    pitch_offsets: Vec<Receiver>,
}

impl OscPanel {
    pub fn from_oscs(
        oscillators: Vec<Oscillator>,
        weights: Option<Vec<f32>>,
        modulation: Option<Vec<f32>>,
    ) -> Result<Self, Error> {
        let len = oscillators.len();
        Ok(Self {
            pitch_offsets: vec![PITCH_OFFSET_RECEIVER; oscillators.len()],
            oscillators,
            weights: network::vec_or_none(weights, len, WEIGHT_RECEIVER)?,
            modulation: network::vec_or_none(modulation, len, MODULATION_RECEIVER)?,
        })
    }

    pub fn extract(&self) -> Self {
        Self {
            oscillators: self.oscillators.clone(),
            modulation: self
                .modulation
                .iter()
                .map(|receiver| receiver.extract())
                .collect(),
            weights: self
                .weights
                .iter()
                .map(|receiver| receiver.extract())
                .collect(),
            pitch_offsets: self
                .pitch_offsets
                .iter()
                .map(|receiver| receiver.extract())
                .collect(),
        }
    }

    pub fn set_id(&mut self, track_id: u8) {
        self.modulation
            .iter_mut()
            .for_each(|receiver| receiver.set_id(track_id));
        self.weights
            .iter_mut()
            .for_each(|receiver| receiver.set_id(track_id));
        self.pitch_offsets
            .iter_mut()
            .for_each(|receiver| receiver.set_id(track_id));
    }
}

impl Default for OscPanel {
    fn default() -> Self {
        Self {
            oscillators: vec![Oscillator::default()],
            weights: vec![WEIGHT_RECEIVER],
            pitch_offsets: vec![PITCH_OFFSET_RECEIVER],
            modulation: vec![MODULATION_RECEIVER],
        }
    }
}

impl OscPanel {
    pub fn play(&self, freq: f32, cent_offsets: &[f32], start: ClockTick, samples: usize) -> Wave {
        let mut wave = vec![0.0; samples];

        for (((osc, weigth), modulation), offset) in self
            .oscillators
            .iter()
            .zip(&self.weights)
            .zip(&self.modulation)
            .zip(&self.pitch_offsets)
        {
            // TODO
            let freq: Vec<f32> = offset
                .get_vec(start, samples)
                .into_iter()
                .zip(cent_offsets)
                .map(|(x, y)| freq * utils::fast_pow2((x + y) / 1200.0))
                .collect();

            let modulation = modulation.get_vec(start, samples);
            let new_wave: Vec<f32> = osc
                .play(&freq, &modulation, samples)
                .into_iter()
                .zip(weigth.get_vec(start, samples))
                .map(|(x, y)| x * y)
                .collect();

            utils::add_elementwise(&mut wave, &new_wave)
        }
        Wave::from_vec(wave)
    }

    pub fn add_osc(&mut self, oscillator: Oscillator) {
        self.oscillators.push(oscillator);
        self.pitch_offsets.push(PITCH_RECEIVER);
        self.weights.push(WEIGHT_RECEIVER);
    }
}
