use std::{cell::RefCell, marker::PhantomData, rc::Rc};

use crate::{
    auto::Control,
    time::{TimeKeeper, TimeManager, TimeStamp},
    wave::Wave,
};

use super::{Controler, EffCtrlMarker, EffMarker, Effect};

const VOL_RANGE: (f64, f64) = (0.0, 5.0);

#[derive(Debug)]
pub struct Volume<W> {
    phantom: PhantomData<W>,
    controler: VolCtrl,
}

impl<W: Wave> Volume<W>{
    pub fn new()-> Self{
        Self { phantom: PhantomData, controler: VolCtrl::default() }
    }
}

impl<W: Wave> Default for Volume<W> {
    fn default() -> Self {
        Self::new()
    }
}

impl<W: Wave> TimeKeeper for Volume<W> {
    fn set_time_manager(&mut self, time_manager: Rc<RefCell<TimeManager>>) {
        self.controler.set_time_manager(time_manager)
    }
}

impl<W: Wave> Effect<W> for Volume<W> {
    fn apply(&self, wave: &mut W, time_triggered: TimeStamp) {
        let vol = self
            .controler
            .get_vol_control()
            .get_vec(time_triggered, wave.len());
        wave.scale_by_vec(vol)
    }

    fn set_defaults(&mut self) {
        todo!()
    }

    fn get_controls(&mut self) -> &mut dyn super::Controler {
        &mut self.controler
    }

    fn on(&mut self) {
        todo!()
    }

    fn off(&mut self) {
        todo!()
    }

    fn toggle(&mut self) {
        todo!()
    }
}

#[derive(Debug)]
pub struct VolCtrl {
    volume: Control,
}

impl VolCtrl {
    pub fn new() -> Self {
        let mut ctrl = Self {
            volume: Control::default(),
        };
        ctrl.set_defaults();
        ctrl
    }
}

impl VolCtrl {
    pub fn get_vol_control(&self) -> &Control {
        &self.volume
    }
}

impl Default for VolCtrl {
    fn default() -> Self {
        Self::new()
    }
}

impl TimeKeeper for VolCtrl {
    fn set_time_manager(&mut self, time_manager: Rc<RefCell<TimeManager>>) {
        self.volume.set_time_manager(time_manager)
    }
}

impl Controler for VolCtrl {
    fn set_defaults(&mut self) {
        self.volume = Control::from_val_in_range(1.0, VOL_RANGE)
    }
}

impl<W: Wave> EffMarker<W> for Volume<W> {}
impl EffCtrlMarker for VolCtrl {}
