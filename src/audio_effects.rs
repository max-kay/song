
pub trait Effect{
    fn apply(&self, wave: Vec<f64>);
    fn apply_automated(&self, wave: Vec<f64>, automation: Vec<Vec<f64>>);
}

struct Delay{
    time: f64,
    decay: f64,
}

impl Effect for Delay{
    fn apply(&self, sound: Vec<f64>, ) -> Vec<f64>{

    }
}