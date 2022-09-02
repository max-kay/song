use crate::time::TimeStamp;


#[derive(Debug)]
pub struct LocalGManager;

impl LocalGManager{
    pub fn new() -> Self{
        todo!()
    }

    pub fn set_velocity(&self, vel: f64){
        todo!()
    }

    pub fn get_main_envelope(&self, note_on: TimeStamp, sus_samples: usize) -> Vec<f64>{
        todo!()
    }
}