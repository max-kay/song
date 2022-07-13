use super::{Control, Controler, EffCtrlMarker, EffMarker, Effect};
use crate::{
    time::{TimeKeeper, TimeManager, TimeStamp},
    utils,
    wave::Wave,
};
use std::{cell::RefCell, marker::PhantomData, rc::Rc};

const SMALLEST_GAIN_ALLOWED: f64 = 0.05;
const GAIN_RANGE: (f64, f64) = (0.0, 0.95);
const DELTA_T_RANGE: (f64, f64) = (0.001, 6.0);

#[derive(Debug)]
pub struct Delay<W: Wave> {
    phantom: PhantomData<W>,
    time_manager: Rc<RefCell<TimeManager>>,
    on: bool,
    controler: DelayCrtl,
}

impl<W: Wave> Delay<W> {
    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
            time_manager: Rc::new(RefCell::new(TimeManager::default())),
            on: true,
            controler: DelayCrtl::default(),
        }
    }
}

impl<W: Wave> Default for Delay<W> {
    fn default() -> Self {
        Self::new()
    }
}

impl<W: Wave> TimeKeeper for Delay<W> {
    fn set_time_manager(&mut self, time_manager: Rc<RefCell<TimeManager>>) {
        self.time_manager = Rc::clone(&time_manager);
        self.controler.set_time_manager(time_manager)
    }
}

impl<W: Wave> Effect<W> for Delay<W> {
    fn apply(&self, wave: &mut W, time_triggered: TimeStamp) {
        let mut source = wave.clone();
        let mut current_time = time_triggered;
        let mut gain: f64 = self.controler.get_gain_ctrl().get_value(time_triggered);
        let mut delta_t = self.controler.get_delta_t_ctrl().get_value(time_triggered);
        while gain > SMALLEST_GAIN_ALLOWED {
            source.scale(gain);
            wave.add(&source, utils::seconds_to_samples(delta_t));
            current_time = self
                .time_manager
                .borrow()
                .add_seconds_to_stamp(current_time, delta_t);
            delta_t += self.controler.get_delta_t_ctrl().get_value(current_time);
            gain *= self.controler.get_gain_ctrl().get_value(current_time);
        }
    }

    fn set_defaults(&mut self) {
        self.controler.set_defaults()
    }

    fn get_controls(&mut self) -> &mut dyn Controler {
        &mut self.controler
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

#[derive(Debug)]
pub struct DelayCrtl {
    gain: Control,
    delta_t: Control,
}

impl DelayCrtl {
    pub fn new() -> Self {
        let mut ctrl = Self {
            gain: Default::default(),
            delta_t: Default::default(),
        };
        ctrl.set_defaults();
        ctrl
    }
}

impl DelayCrtl {
    fn get_gain_ctrl(&self) -> &Control {
        &self.gain
    }
    fn get_delta_t_ctrl(&self) -> &Control {
        &self.delta_t
    }
}

impl Default for DelayCrtl {
    fn default() -> Self {
        Self::new()
    }
}

impl TimeKeeper for DelayCrtl {
    fn set_time_manager(&mut self, time_manager: Rc<RefCell<TimeManager>>) {
        self.gain.set_time_manager(Rc::clone(&time_manager));
        self.delta_t.set_time_manager(Rc::clone(&time_manager));
    }
}

impl Controler for DelayCrtl {
    fn set_defaults(&mut self) {
        self.gain = Control::from_val_in_range(0.6, GAIN_RANGE);
        self.delta_t = Control::from_val_in_range(0.6, DELTA_T_RANGE)
    }
}

impl<W: Wave> EffMarker<W> for Delay<W> {}
impl EffCtrlMarker for DelayCrtl {}
