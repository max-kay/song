use serde::{Serialize, Deserialize};

use crate::{ctrl_f::GenId, globals::GENRATOR_MANAGER, time::TimeStamp, utils, Error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Network {
    Leaf(GenId),
    WeightedAverage(Vec<(f64, Network)>),
    WeightedProduct(Vec<(f64, Network)>),
    Inverted(Box<Network>),
}

impl Network {
    pub fn get_ids(&self) -> Result<Vec<GenId>, Error> {
        match self {
            Network::Leaf(id) => {
                let mut out = GENRATOR_MANAGER.read().unwrap().get_sub_ids(*id)?;
                out.push(*id);
                Ok(out)
            }
            Network::WeightedAverage(vec) => {
                let mut out = Vec::new();
                for (_, net) in vec {
                    out.append(&mut net.get_ids()?)
                }
                Ok(out)
            }
            Network::WeightedProduct(vec) => {
                let mut out = Vec::new();
                for (_, net) in vec {
                    out.append(&mut net.get_ids()?)
                }
                Ok(out)
            }
            Network::Inverted(net) => net.get_ids(),
        }
    }
}

impl Network {
    pub fn get_val(&self, time: TimeStamp) -> f64 {
        match self {
            Network::Leaf(id) => GENRATOR_MANAGER
                .read()
                .unwrap()
                .get_val(*id, time)
                .expect("error in network"),
            Network::WeightedAverage(vec) => {
                let mut out = 0.0;
                let mut sum = 0.0;
                for (weight, net) in vec {
                    sum += weight;
                    out += weight * net.get_val(time)
                }
                out / sum
            }
            Network::WeightedProduct(vec) => {
                let mut out = 1.0;
                let mut sum = 0.0;
                for (weight, net) in vec {
                    sum += weight;
                    out *= net.get_val(time).powf(*weight)
                }
                out.powf(1.0 / sum)
            }
            Network::Inverted(net) => net.get_val(time) * -1.0 + 1.0,
        }
    }

    pub fn get_vec(&self, start: TimeStamp, samples: usize) -> Vec<f64> {
        match self {
            Network::Leaf(id) => GENRATOR_MANAGER
                .read()
                .unwrap()
                .get_vec(*id, start, samples)
                .expect("error in network"),
            Network::WeightedAverage(vec) => {
                let mut out = Vec::new();
                let mut sum = 0.0;
                for (weight, net) in vec {
                    sum += weight;
                    utils::add_elementwise(
                        &mut out,
                        net.get_vec(start, samples)
                            .into_iter()
                            .map(|x| x * weight)
                            .collect(),
                    );
                }
                out.into_iter().map(|x| x / sum).collect()
            }
            Network::WeightedProduct(vec) => {
                let mut out = vec![1.0; samples];
                let mut sum = 0.0;
                for (weight, net) in vec {
                    sum += weight;
                    utils::mul_elementwise(
                        &mut out,
                        net.get_vec(start, samples)
                            .into_iter()
                            .map(|x| x.powf(*weight))
                            .collect(),
                    )
                }
                out.into_iter().map(|x| x.powf(1.0 / sum)).collect()
            }
            Network::Inverted(net) => net
                .get_vec(start, samples)
                .into_iter()
                .map(|x| x * -1.0 + 1.0)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Serialize, Deserialize)]
pub enum Transform {
    Linear,
}

impl Transform {
    pub fn get_fn(&self, range: (f64, f64)) -> Box<dyn Fn(f64) -> f64> {
        match self {
            Transform::Linear => Box::new(move |x: f64| x * (range.1 - range.0) + range.0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reciever {
    value: f64,
    range: (f64, f64),
    transform: Transform,
    network: Option<Network>,
}

impl Reciever {
    pub const fn new(value: f64, range: (f64, f64), transform: Transform) -> Self {
        Self {
            value,
            range,
            network: None,
            transform,
        }
    }
    pub fn set_value(&mut self, value: f64) -> Result<(), Error> {
        if in_range(value, self.range) {
            self.value = value;
            Ok(())
        } else {
            Err(Error::Reciever)
        }
    }
    pub(crate) fn sv(mut self, val: f64) -> Self {
        self.value = val;
        self
    }
    pub(crate) fn csv(mut self, val: f64) -> Result<Self, Error> {
        if in_range(val, self.range) {
            self.value = val;
            Ok(self)
        } else {
            Err(Error::Reciever)
        }
    }

    pub fn compare(&self, other: &Reciever) -> bool {
        self.range == other.range && self.transform == other.transform
    }

    pub fn get_ids(&self) -> Vec<GenId> {
        match &self.network {
            Some(net) => net.get_ids().unwrap(),
            None => Vec::new(),
        }
    }
}

pub fn set_reciever(
    target: &mut Reciever,
    target_id: GenId,
    source: &Reciever,
) -> Result<(), Error> {
    if !target.compare(source) {
        return Err(Error::RecieverMisMatch);
    }
    if !source.get_ids().contains(&target_id) {
        target.value = source.value;
        target.network = source.network.clone();
        Ok(())
    } else {
        Err(Error::Loop)
    }
}

#[inline(always)]
fn in_range(val: f64, range: (f64, f64)) -> bool {
    (val >= range.0 && val <= range.1) | (val >= range.1 && val <= range.0)
}

impl Reciever {
    pub fn get_vec(&self, start: TimeStamp, samples: usize) -> Vec<f64> {
        match &self.network {
            None => vec![self.transform.get_fn(self.range)(self.value); samples],
            Some(net) => net
                .get_vec(start, samples)
                .into_iter()
                .map(self.transform.get_fn(self.range))
                .collect(),
        }
    }

    pub fn get_val(&self, time: TimeStamp) -> f64 {
        self.transform.get_fn(self.range)(match &self.network {
            None => self.value,
            Some(net) => net.get_val(time),
        })
    }
}
