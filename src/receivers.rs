use crate::network::{Receiver, Transform};

pub const VOL_RECEIVER: Receiver = Receiver::new(1.0, (0.0, 5.0), Transform::Linear);
