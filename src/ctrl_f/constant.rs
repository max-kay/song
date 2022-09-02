#[derive(Debug, Default)]
pub struct Constant {
    val: f64,
}

impl Constant {
    pub fn new() -> Self {
        Self { val: 0.0 }
    }
    pub fn set(&mut self, value: f64) {
        assert!((0.0..=1.0).contains(&value));
        self.val = value
    }
}

impl Constant {
    pub fn get_val(&self) -> f64 {
        self.val
    }

    pub fn get_vec(&self, samples: usize) -> Vec<f64> {
        vec![self.val; samples]
    }
}
