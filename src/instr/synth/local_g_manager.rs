use serde::{Serialize, Deserialize};

use crate::{
    ctrl_f::{Constant, Envelope, GenId, Lfo, PointDefined, SaveId},
    globals::GENRATOR_MANAGER,
    time::TimeStamp,
    Error,
};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LocalGManager {
    track_id: Option<u8>,
    main_enevelope: GenId,
    alt_enevelope: GenId,
    lfo_1: GenId,
    lfo_2: GenId,
    velocity: GenId,
    pitch_wheel: GenId,
    mod_wheel: GenId,
}

impl LocalGManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_global(&self, key: u8) -> GenId {
        GenId::Global(key)
    }

    pub fn get_local(&self, key: u8) -> Result<GenId, Error> {
        match self.track_id {
            Some(track) => Ok(GenId::Track { track, key }),
            None => Err(Error::Unintialised),
        }
    }
    pub fn get_instr(&self, key: u8) -> Result<GenId, Error> {
        match self.track_id {
            Some(track) => Ok(GenId::Instr { track, key }),
            None => Err(Error::Unintialised),
        }
    }

    pub fn set_velocity(&self, vel: f64) {
        GENRATOR_MANAGER
            .write()
            .unwrap()
            .set_const(self.velocity, vel)
            .unwrap()
    }

    pub fn get_main_envelope(&self, note_on: TimeStamp, sus_samples: usize) -> Vec<f64> {
        GENRATOR_MANAGER
            .read()
            .unwrap()
            .get_envelope(self.main_enevelope, note_on, sus_samples)
            .unwrap()
    }

    pub fn init(&mut self, id: u8) -> Result<(), Error> {
        let instr = SaveId::Instr(id);
        let track = SaveId::Track(id);
        let mut manager = GENRATOR_MANAGER.write().unwrap();
        manager.new_track(id)?;
        self.main_enevelope = manager.add_generator(Envelope::w_default(), instr)?;
        self.alt_enevelope = manager.add_generator(Envelope::w_default(), instr)?;
        self.lfo_1 = manager.add_generator(Lfo::w_default(), instr)?;
        self.lfo_2 = manager.add_generator(Lfo::w_default(), instr)?;
        self.velocity = manager.add_generator(Constant::w_default(), track)?;
        self.mod_wheel = manager.add_generator(PointDefined::w_default(), track)?;
        self.pitch_wheel = manager.add_generator(PointDefined::w_default(), track)?;
        Ok(())
    }
}
