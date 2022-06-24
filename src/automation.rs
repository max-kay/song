use std::collections::{BTreeMap, HashMap};

pub struct AutomationPoint {
    value: f64,
    time: f64,
}

impl AutomationPoint {
    pub fn new(value: f64, time: f64) -> Self {
        assert!(
            value >= 0.0 && value <= 1.0,
            "the value of an AutomationPoint has to in [0.0, 1.0] (closed interval)"
        );
        Self { value, time }
    }
}

impl PartialEq for AutomationPoint {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.time == other.time
    }
}

impl Eq for AutomationPoint {}

impl PartialOrd for AutomationPoint {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.time.partial_cmp(&other.time) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.value.partial_cmp(&other.value)
    }
}

impl Ord for AutomationPoint {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other)
            .expect("error in Ord of AutomationPoint")
    }
}

pub enum InterpolationMethod {
    Linear,
    Smooth,
    RepeatingLinear { start: f64, end: f64 },
    RepeatingSmooth { start: f64, end: f64 },
}

impl InterpolationMethod {
    pub fn is_repeating(&self) -> Option<(f64, f64)> {
        match self {
            Self::Linear => None,
            Self::Smooth => None,
            Self::RepeatingLinear { start, end } => Some((*start, *end)),
            Self::RepeatingSmooth { start, end } => Some((*start, *end)),
        }
    }
}

pub struct Channel {
    points: BTreeMap<f64, f64>,
    interpolation: InterpolationMethod,
}

impl Channel {
    pub fn new(mut points: BTreeMap<f64, f64>, interpolation: InterpolationMethod) -> Self {
        match interpolation.is_repeating() {
            None => {}
            Some((start, end)) => {
                if (points.iter().next().unwrap().0 < &start)
                    | (points
                        .iter()
                        .last()
                        .expect("no automation points provided")
                        .0
                        > &end)
                {
                    panic!("automation point not in loop!")
                }
            }
        }
        Self {
            points,
            interpolation,
        }
    }

    pub fn get_value(&self, time: f64) -> f64 {
        match self.interpolation {
            InterpolationMethod::Linear => todo!(),
            InterpolationMethod::Smooth => todo!(),
            InterpolationMethod::RepeatingLinear { start, end } => todo!(),
            InterpolationMethod::RepeatingSmooth { start, end } => todo!(),
        }
    }
}

pub struct AutomationManager {
    channels: HashMap<u8, Channel>,
}

impl AutomationManager {
    pub fn new() -> Self {
        Self {
            channels: HashMap::<u8, Channel>::new(),
        }
    }

    pub fn get_channel(&self, channel: u8) -> Option<&Channel> {
        self.channels.get(&channel)
    }
}

impl Default for AutomationManager {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ValAndCh {
    pub value: f64,
    pub connection: Option<(u8, f64)>, // the f64 here is the prescalar
}

impl ValAndCh {
    pub fn get_value(&self, automation: &AutomationManager, time: f64) -> f64 {
        match self.connection {
            None => self.value,
            Some(connection) => match automation.get_channel(connection.0) {
                None => self.value,
                Some(channel) => channel.get_value(time) * connection.1,
            },
        }
    }
}
