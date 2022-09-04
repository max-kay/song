use serde::{Serialize, Deserialize};

use super::{GenId, Generator};

#[derive(Debug, Clone, Serialize , Deserialize)]
pub struct Constant {
    id: GenId,
    val: f64,
}

impl Constant {
    pub fn w_default() -> Generator {
        Generator::Constant(Self::default())
    }

    pub fn new() -> Self {
        Self {
            id: GenId::Unbound,
            val: 0.0,
        }
    }

    pub fn get_sub_ids(&self) -> Vec<GenId> {
        Vec::new()
    }

    pub(crate) fn set_id(&mut self, id: GenId) {
        self.id = id
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

impl Default for Constant {
    fn default() -> Self {
        Self::new()
    }
}
