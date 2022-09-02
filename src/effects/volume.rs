use std::marker::PhantomData;

use once_cell::sync::Lazy;

use crate::{
    network::{Reciever, Transform},
    time::TimeStamp,
    wave::Wave,
};

use super::{EffMarker, Effect};

static VOL_RECIEVER: Lazy<Reciever> =
    Lazy::new(|| Reciever::new(1.0, (0.0, 5.0), Transform::Linear));

#[derive(Debug)]
pub struct Volume<W> {
    phantom: PhantomData<W>,
    volume: Reciever,
    on: bool,
}

impl<W: Wave> Volume<W> {
    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
            volume: VOL_RECIEVER.clone(),
            on: true,
        }
    }
}

impl<W: Wave> Default for Volume<W> {
    fn default() -> Self {
        Self::new()
    }
}

impl<W: Wave> Effect<W> for Volume<W> {
    fn apply(&self, wave: &mut W, time_triggered: TimeStamp) {
        if self.on {
            let vol = self.volume.get_vec(time_triggered, wave.len());
            wave.scale_by_vec(vol)
        }
    }

    fn set_defaults(&mut self) {
        self.volume = VOL_RECIEVER.clone()
    }

    fn on(&mut self) {
        self.on = true
    }

    fn off(&mut self) {
        self.on = false
    }

    fn toggle(&mut self) {
        self.on = !self.on
    }
}

impl<W: Wave> EffMarker<W> for Volume<W> {}
