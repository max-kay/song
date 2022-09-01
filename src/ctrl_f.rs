use crate::{
    control::{ControlError, FunctionKeeper},
    time::{TimeKeeper, TimeManager, TimeStamp},
};
use dyn_clone::DynClone;
use serde::{
    de::{Deserialize, Deserializer, Visitor},
    ser::{Serialize, SerializeMap, Serializer},
};
use std::{cell::{RefCell, UnsafeCell}, collections::HashMap, fmt::Debug, rc::Rc};
use typetag;

pub mod constant;
pub mod envelope;
pub mod lfo;
pub mod phantom_f;
pub mod point_defined;

pub use constant::Constant;
pub use envelope::Envelope;
pub use lfo::Lfo;
pub use point_defined::PointDefined;

pub(crate) use phantom_f::PhantomF;

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

pub fn default_ctrl_f() -> Rc<RefCell<dyn CtrlFunction>> {
    Rc::new(RefCell::new(PhantomF))
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

#[typetag::serde]
pub trait CtrlFunction: Debug + FunctionKeeper + DynClone {
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

dyn_clone::clone_trait_object!(CtrlFunction);

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

#[derive(Debug, Clone)]
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

    pub fn insert(
        &mut self,
        channel: u8,
        func: &dyn CtrlFunction,
    ) -> Option<Rc<RefCell<dyn CtrlFunction>>> {
        let mut value = Rc::new(RefCell::new(PhantomF)) as Rc<RefCell<dyn CtrlFunction>>;
        // unsafe {
        //     let cell = UnsafeCell::new(*func);

        //     // value.get_mut() = &mut dyn_clone::clone(func)
        // }
        todo!()
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

impl Serialize for FunctionManager {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.channels.len()))?;
        for (k, v) in self.channels.iter() {
            unsafe {
                match &v.try_borrow_unguarded() {
                    Ok(x) => map.serialize_entry(&k, x)?,
                    Err(_) => todo!(),
                }
            }
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for FunctionManager {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        todo!()
    }
}

struct FunctionManagerVisitor;

impl<'de> Visitor<'de> for FunctionManagerVisitor {
    type Value = FunctionManager;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a map of id to CtrlFunctions")
    }

    fn visit_map<A>(self, access: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mangr = FunctionManager::new();
        todo!();
        Ok(mangr)
    }
}

pub trait FunctionMngrKeeper: FunctionOwner {
    fn set_fuction_manager(&mut self, function_manager: Rc<RefCell<FunctionManager>>);
}
