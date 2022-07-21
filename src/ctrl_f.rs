use crate::{
    control::{ControlError, FunctionKeeper},
    time::{TimeKeeper, TimeManager, TimeStamp},
};
use std::{cell::RefCell, collections::HashMap, fmt::Debug, rc::Rc};

pub mod constant;
pub mod envelope;
pub mod lfo;
pub mod phantom;
pub mod point_defined;

pub use constant::Constant;
pub use envelope::Envelope;
pub use lfo::Lfo;
pub use point_defined::PointDefined;

pub type IdMapOrErr = Result<HashMap<usize, Rc<RefCell<dyn CtrlFunction>>>, ControlError>;
pub type IdMap = HashMap<usize, Rc<RefCell<dyn CtrlFunction>>>;

pub fn make_ctrl_function<'a, T>(
    ctrl_function: Rc<RefCell<T>>,
) -> Rc<RefCell<dyn CtrlFunction + 'a>>
where
    T: CtrlFunction + 'a,
{
    Rc::clone(&ctrl_function) as Rc<RefCell<dyn CtrlFunction>>
}

pub fn try_insert_id(
    ctrl_func: Rc<RefCell<dyn CtrlFunction>>,
    map: &mut HashMap<usize, Rc<RefCell<dyn CtrlFunction>>>,
) -> Result<(), ControlError> {
    if let Some(func) = map.insert(ctrl_func.borrow().get_id(), Rc::clone(&ctrl_func)) {
        return Err(ControlError::new_double_id_err(func.borrow().get_id()));
    }
    Ok(())
}

pub trait CtrlFunction: Debug + FunctionKeeper {
    fn get_value(&self, time: TimeStamp) -> f64;
    fn get_vec(&self, start: TimeStamp, samples: usize) -> Vec<f64>;
    fn get_id(&self) -> usize;
    // fn get_sub_ids(&self) -> Vec<usize>;
    /// .
    ///
    /// # Safety
    /// This function sets a new id for the CtrlFunction.
    /// If this is done without recieving the new ids in all ControlKeepers Serialization is possible
    /// but there will be errors when Deserializing!
    /// .
    unsafe fn new_id_f(&mut self);
}

pub trait FunctionOwner: TimeKeeper {
    /// .
    ///
    /// # Safety
    /// This function sets a new id for each function in the FuctionKeeper.
    /// If this is done without recieving the new ids in all ControlKeepers Serialization is possible
    /// but there will be errors when Deserializing!
    /// .
    unsafe fn new_ids(&mut self);
    fn get_id_map(&self) -> IdMapOrErr;
}

#[derive(Debug)]
pub struct FunctionManager {
    channels: HashMap<u8, Rc<RefCell<dyn CtrlFunction>>>,
}

impl FunctionManager {
    pub fn new() -> Self {
        Self {
            channels: HashMap::new(),
        }
    }

    pub fn all_channels(&self) -> Vec<u8> {
        self.channels.keys().into_iter().copied().collect()
    }

    pub fn get_channel(&self, channel: u8) -> Option<Rc<RefCell<dyn CtrlFunction>>> {
        self.channels.get(&channel).map(Rc::clone)
    }
}

impl Default for FunctionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TimeKeeper for FunctionManager {
    fn set_time_manager(&mut self, time_manager: Rc<RefCell<TimeManager>>) {
        for control in self.channels.values_mut() {
            control
                .borrow_mut()
                .set_time_manager(Rc::clone(&time_manager))
        }
    }
}

impl FunctionOwner for FunctionManager {
    unsafe fn new_ids(&mut self) {
        for func in self.channels.values() {
            func.borrow_mut().new_id_f()
        }
    }

    fn get_id_map(&self) -> IdMapOrErr {
        let mut map = IdMap::new();
        for func in self.channels.values() {
            try_insert_id(Rc::clone(func), &mut map)
                .map_err(|err| err.push_location("FunctionManager"))?;
        }
        Ok(map)
    }
}

impl FunctionKeeper for FunctionManager {
    fn heal_sources(&mut self, id_map: &IdMap) -> Result<(), ControlError> {
        for func in self.channels.values() {
            func.borrow_mut()
                .heal_sources(id_map)
                .map_err(|err| err.push_location("FunctionManager"))?;
        }
        Ok(())
    }

    fn test_sources(&self) -> Result<(), ControlError> {
        for func in self.channels.values() {
            func.borrow()
                .test_sources()
                .map_err(|err| err.push_location("FunctionManager"))?;
        }
        Ok(())
    }

    fn set_ids(&mut self) {
        for func in self.channels.values() {
            func.borrow_mut().set_ids()
        }
    }

    fn get_ids(&self) -> Vec<usize> {
        let mut ids = Vec::new();
        for func in self.channels.values() {
            ids.append(&mut func.borrow().get_ids())
        }
        ids
    }
}

pub trait FunctionMngrKeeper: FunctionOwner {
    fn set_fuction_manager(&mut self, function_manager: Rc<RefCell<FunctionManager>>);
}
