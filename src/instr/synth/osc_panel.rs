use super::PITCH_WHEEL_RANGE;
use crate::{
    control::{Control, ControlError, FunctionKeeper},
    ctrl_f::IdMap,
    time::TimeStamp,
    utils,
    utils::oscs::Oscillator,
    wave::Wave,
};
use std::{marker::PhantomData, result::Result};

#[derive(Debug)]
pub struct OscPanel<W: Wave> {
    phantom: PhantomData<W>,
    oscillators: Vec<Oscillator>,
    weights: Vec<Control>,
    pitch_offsets: Vec<Control>,
}

impl<W: Wave> Default for OscPanel<W> {
    fn default() -> Self {
        Self {
            phantom: PhantomData::<W>,
            oscillators: vec![Oscillator::default()],
            weights: vec![Control::from_val_in_unit(1.0).unwrap()],
            pitch_offsets: vec![Control::from_val_in_range(0.0, (-4800.0, 4800.0)).unwrap()],
        }
    }
}

impl<W: Wave> OscPanel<W> {
    pub fn play(
        &self,
        freq: Vec<f64>,
        modulation: Vec<f64>,
        start: TimeStamp,
        samples: usize,
    ) -> W {
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
        W::from_vec(wave)
    }

    pub fn add_osc(&mut self, oscillator: Oscillator) {
        self.oscillators.push(oscillator);
        self.pitch_offsets
            .push(Control::from_val_in_range(0.0, PITCH_WHEEL_RANGE).unwrap());
        self.weights.push(Control::from_val_in_unit(1.0).unwrap());
    }
}

impl<W: Wave> FunctionKeeper for OscPanel<W> {
    fn heal_sources(&mut self, id_map: &IdMap) -> Result<(), ControlError> {
        for w in &mut self.weights {
            w.heal_sources(id_map)
                .map_err(|err| err.push_location("OscPanel"))?;
        }
        for p in &mut self.pitch_offsets {
            p.heal_sources(id_map)
                .map_err(|err| err.push_location("OscPanel"))?;
        }
        Ok(())
    }

    fn set_ids(&mut self) {
        for w in &mut self.weights {
            w.set_ids()
        }
        for p in &mut self.pitch_offsets {
            p.set_ids()
        }
    }

    fn test_sources(&self) -> Result<(), ControlError> {
        for w in &self.weights {
            w.test_sources()
                .map_err(|err| err.push_location("OscPanel"))?;
        }
        for p in &self.pitch_offsets {
            p.test_sources()
                .map_err(|err| err.push_location("OscPanel"))?;
        }
        Ok(())
    }

    fn get_ids(&self) -> Vec<usize> {
        let mut ids = Vec::new();
        for w in &self.weights {
            ids.append(&mut w.get_ids())
        }
        for p in &self.pitch_offsets {
            ids.append(&mut p.get_ids())
        }
        ids
    }
}
