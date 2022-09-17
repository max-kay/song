use serde::{Deserialize, Serialize};

use crate::{gens::GenId, globals::GENRATOR_MANAGER, time::ClockTick, utils, Error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Network {
    Leaf(GenId),
    WeightedAverage(Vec<(f32, Network)>),
    WeightedProduct(Vec<(f32, Network)>),
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

    pub fn extract(&self) -> Option<Self> {
        match self {
            Network::Leaf(id) => {
                if let Ok(new_id) = id.extract() {
                    Some(Self::Leaf(new_id))
                } else {
                    None
                }
            }
            Network::WeightedAverage(vec) => {
                let new_vec: Vec<_> = vec
                    .iter()
                    .filter_map(|(w, net)| Some((*w, net.extract()?)))
                    .collect();
                if !new_vec.is_empty() {
                    Some(Self::WeightedAverage(new_vec))
                } else {
                    None
                }
            }
            Network::WeightedProduct(vec) => {
                let new_vec: Vec<_> = vec
                    .iter()
                    .filter_map(|(w, net)| Some((*w, net.extract()?)))
                    .collect();
                if !new_vec.is_empty() {
                    Some(Self::WeightedProduct(new_vec))
                } else {
                    None
                }
            }
            Network::Inverted(net) => net.extract(),
        }
    }

    pub fn set_id(&mut self, track_id: u8) {
        match self {
            Network::Leaf(id) => id.set_id(track_id),
            Network::WeightedAverage(vec) => {
                vec.iter_mut().for_each(|(_, net)| net.set_id(track_id))
            }
            Network::WeightedProduct(vec) => {
                vec.iter_mut().for_each(|(_, net)| net.set_id(track_id))
            }
            Network::Inverted(net) => net.set_id(track_id),
        }
    }
}

impl Network {
    pub fn get_val(&self, time: ClockTick) -> f32 {
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

    pub fn get_vec(&self, start: ClockTick, samples: usize) -> Vec<f32> {
        match self {
            Network::Leaf(id) => GENRATOR_MANAGER
                .read()
                .unwrap()
                .get_vec(*id, start, samples)
                .expect("error in network"),
            Network::WeightedAverage(vec) => {
                let mut out = Vec::with_capacity(samples);
                let mut sum = 0.0;
                for (weight, net) in vec {
                    sum += weight;
                    let part: Vec<f32> = net
                        .get_vec(start, samples)
                        .into_iter()
                        .map(|x| x * weight)
                        .collect();
                    utils::add_elementwise(&mut out, &part);
                }
                out.into_iter().map(|x| x / sum).collect()
            }
            Network::WeightedProduct(vec) => {
                let mut out = vec![1.0; samples];
                let mut sum = 0.0;
                for (weight, net) in vec {
                    sum += weight;
                    let part: Vec<f32> = net
                        .get_vec(start, samples)
                        .into_iter()
                        .map(|x| x.powf(*weight))
                        .collect();
                    utils::mul_elementwise(&mut out, &part)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Transform {
    Linear,
}

impl Transform {
    pub fn get_fn(&self, range: (f32, f32)) -> Box<dyn Fn(f32) -> f32> {
        match self {
            Transform::Linear => Box::new(move |x: f32| x * (range.1 - range.0) + range.0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Receiver {
    value: f32,
    range: (f32, f32),
    transform: Transform,
    network: Option<Network>,
}

impl Receiver {
    pub const fn new(value: f32, range: (f32, f32), transform: Transform) -> Self {
        Self {
            value,
            range,
            network: None,
            transform,
        }
    }

    pub fn change_network(
        &mut self,
        network: Network,
        parent_id: Option<GenId>,
    ) -> Result<(), Error> {
        match parent_id {
            Some(id) => {
                if network.get_ids()?.contains(&id) {
                    Err(Error::Loop)
                } else {
                    self.network = Some(network);
                    Ok(())
                }
            }
            None => {
                self.network = Some(network);
                Ok(())
            }
        }
    }

    pub fn delete_network(&mut self) {
        self.network = None
    }

    pub fn set_value(&mut self, value: f32) -> Result<(), Error> {
        if in_range(value, self.range) {
            self.value = value;
            Ok(())
        } else {
            Err(Error::Receiver)
        }
    }

    pub(crate) fn sv(mut self, val: f32) -> Self {
        self.value = val;
        self
    }

    pub(crate) fn csv(mut self, val: f32) -> Result<Self, Error> {
        if in_range(val, self.range) {
            self.value = val;
            Ok(self)
        } else {
            Err(Error::Receiver)
        }
    }

    pub fn compare(&self, other: &Receiver) -> bool {
        self.range == other.range && self.transform == other.transform
    }

    pub fn get_ids(&self) -> Vec<GenId> {
        match &self.network {
            Some(net) => net.get_ids().unwrap(),
            None => Vec::new(),
        }
    }

    pub fn extract(&self) -> Self {
        let network = if let Some(net) = &self.network {
            net.extract()
        } else {
            None
        };
        Self {
            value: self.value,
            range: self.range,
            transform: self.transform,
            network,
        }
    }

    pub fn set_id(&mut self, track_id: u8) {
        if let Some(net) = &mut self.network {
            net.set_id(track_id)
        }
    }
}

pub fn set_receiver(
    receiver_in_target: &mut Receiver,
    target_id: GenId,
    source: &Receiver,
) -> Result<(), Error> {
    if !receiver_in_target.compare(source) {
        return Err(Error::ReceiverMisMatch);
    }
    if !source.get_ids().contains(&target_id) {
        receiver_in_target.value = source.value;
        receiver_in_target.network = source.network.clone();
        Ok(())
    } else {
        Err(Error::Loop)
    }
}

#[inline(always)]
fn in_range(val: f32, range: (f32, f32)) -> bool {
    (val >= range.0 && val <= range.1) | (val >= range.1 && val <= range.0)
}

impl Receiver {
    pub fn get_vec(&self, start: ClockTick, samples: usize) -> Vec<f32> {
        match &self.network {
            None => vec![self.value; samples],
            Some(net) => net
                .get_vec(start, samples)
                .into_iter()
                .map(self.transform.get_fn(self.range))
                .collect(),
        }
    }

    pub fn get_val(&self, time: ClockTick) -> f32 {
        match &self.network {
            None => self.value,
            Some(net) => self.transform.get_fn(self.range)(net.get_val(time)),
        }
    }
}

pub fn vec_or_none(
    vec: Option<Vec<f32>>,
    len: usize,
    std_receiver: Receiver,
) -> Result<Vec<Receiver>, Error> {
    let mut out = vec![std_receiver; len];
    if let Some(vec) = vec {
        for (r, val) in out.iter_mut().zip(vec) {
            r.set_value(val)?;
        }
    }
    Ok(out)
}
