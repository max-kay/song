use crate::time::{TimeManager, TimeStamp};


#[derive(Debug)]
pub struct LocalFManager {}

impl LocalFManager {
    pub fn new() -> Self{
        todo!()
    }
    pub fn set_velocity(&self, vel: f64) {
        todo!()
    }
    pub fn get_main_envelope(&self, note_on: TimeStamp, samples: usize) -> Vec<f64> {
        todo!()
    }
}
