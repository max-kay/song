use crate::{
    control::{ControlError, FunctionKeeper},
    ctrl_f,
    ctrl_f::{
        Constant, CtrlFunction, Envelope, FunctionManager, FunctionMngrKeeper, FunctionOwner,
        IdMap, IdMapOrErr, Lfo,
    },
};
use std::{cell::RefCell, collections::HashMap, rc::Rc, result::Result};

#[derive(Debug)]
pub struct LocalFManager {
    pub(crate) main_envelope: Rc<RefCell<Envelope>>,
    pub(crate) alt_envelope: Rc<RefCell<Envelope>>,
    pub(crate) current_velocity: Rc<RefCell<Constant>>,
    pub(crate) lfo1: Rc<RefCell<Lfo>>,
    pub(crate) lfo2: Rc<RefCell<Lfo>>,
    pub(crate) track_functions: Rc<RefCell<FunctionManager>>,
}

impl LocalFManager {
    pub fn new() -> Self {
        Self {
            main_envelope: Rc::new(RefCell::new(Envelope::default())),
            alt_envelope: Rc::new(RefCell::new(Envelope::default())),
            current_velocity: Rc::new(RefCell::new(Constant::default())),
            lfo1: Rc::new(RefCell::new(Lfo::default())),
            lfo2: Rc::new(RefCell::new(Lfo::default())),
            track_functions: Rc::new(RefCell::new(FunctionManager::new())),
        }
    }
}

impl Default for LocalFManager {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalFManager {
    pub fn set_velocity(&mut self, velocity: f64) {
        self.current_velocity.borrow_mut().set(velocity)
    }
}

impl FunctionKeeper for LocalFManager {
    fn heal_sources(&mut self, id_map: &IdMap) -> Result<(), ControlError> {
        self.main_envelope
            .borrow_mut()
            .heal_sources(id_map)
            .map_err(|err| err.push_location("LocalFManager"))?;
        self.alt_envelope
            .borrow_mut()
            .heal_sources(id_map)
            .map_err(|err| err.push_location("LocalFManager"))?;
        self.current_velocity
            .borrow_mut()
            .heal_sources(id_map)
            .map_err(|err| err.push_location("LocalFManager"))?;
        self.lfo1
            .borrow_mut()
            .heal_sources(id_map)
            .map_err(|err| err.push_location("LocalFManager"))?;
        self.lfo2
            .borrow_mut()
            .heal_sources(id_map)
            .map_err(|err| err.push_location("LocalFManager"))
    }

    fn test_sources(&self) -> Result<(), ControlError> {
        self.main_envelope
            .borrow()
            .test_sources()
            .map_err(|err| err.push_location("LocalFManager"))?;
        self.alt_envelope
            .borrow()
            .test_sources()
            .map_err(|err| err.push_location("LocalFManager"))?;
        self.current_velocity
            .borrow()
            .test_sources()
            .map_err(|err| err.push_location("LocalFManager"))?;
        self.lfo1
            .borrow()
            .test_sources()
            .map_err(|err| err.push_location("LocalFManager"))?;
        self.lfo2
            .borrow()
            .test_sources()
            .map_err(|err| err.push_location("LocalFManager"))
    }

    fn set_ids(&mut self) {
        self.main_envelope.borrow_mut().set_ids();
        self.alt_envelope.borrow_mut().set_ids();
        self.current_velocity.borrow_mut().set_ids();
        self.lfo1.borrow_mut().set_ids();
        self.lfo2.borrow_mut().set_ids()
    }

    fn get_ids(&self) -> Vec<usize> {
        let mut ids = Vec::new();
        ids.append(&mut self.main_envelope.borrow().get_ids());
        ids.append(&mut self.alt_envelope.borrow().get_ids());
        ids.append(&mut self.current_velocity.borrow().get_ids());
        ids.append(&mut self.lfo1.borrow().get_ids());
        ids.append(&mut self.lfo2.borrow().get_ids());
        ids
    }
}

impl FunctionOwner for LocalFManager {
    unsafe fn new_ids(&mut self) {
        self.main_envelope.borrow_mut().new_id_f();
        self.alt_envelope.borrow_mut().new_id_f();
        self.current_velocity.borrow_mut().new_id_f();
        self.lfo1.borrow_mut().new_id_f();
        self.lfo2.borrow_mut().new_id_f();
        // new ids for the track_functions are set in the Track struct
    }

    fn get_id_map(&self) -> IdMapOrErr {
        let mut map = HashMap::<usize, Rc<RefCell<dyn CtrlFunction>>>::new();

        let main_envelope = ctrl_f::make_ctrl_function(Rc::clone(&self.main_envelope));
        let alt_envelope = ctrl_f::make_ctrl_function(Rc::clone(&self.alt_envelope));
        let lfo1 = ctrl_f::make_ctrl_function(Rc::clone(&self.lfo1));
        let lfo2 = ctrl_f::make_ctrl_function(Rc::clone(&self.lfo2));
        let current_velocity = ctrl_f::make_ctrl_function(Rc::clone(&self.current_velocity));

        ctrl_f::try_insert_id(main_envelope, &mut map)
            .map_err(|err| err.push_location("LocalFManager"))?;
        ctrl_f::try_insert_id(alt_envelope, &mut map)
            .map_err(|err| err.push_location("LocalFManager"))?;
        ctrl_f::try_insert_id(lfo1, &mut map).map_err(|err| err.push_location("LocalFManager"))?;
        ctrl_f::try_insert_id(lfo2, &mut map).map_err(|err| err.push_location("LocalFManager"))?;
        ctrl_f::try_insert_id(current_velocity, &mut map)
            .map_err(|err| err.push_location("LocalFManager"))?;
        // track_functions are inserted in the Track struct

        Ok(map)
    }
}

impl FunctionMngrKeeper for LocalFManager {
    fn set_fuction_manager(&mut self, function_manager: Rc<RefCell<FunctionManager>>) {
        self.track_functions = Rc::clone(&function_manager)
    }
}
