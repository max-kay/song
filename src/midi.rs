#[derive(Clone, Copy)]
pub struct Velocity {
    value: u8,
}
impl Velocity {
    pub fn new(value: u8) -> Option<Self> {
        if value < 0x80 {
            Some(Self { value })
        } else {
            None
        }
    }

    pub fn get(&self) -> u8 {
        self.value
    }
    
    pub fn new_unchecked(value: u8) -> Self {
        Self { value }
    }
}

#[derive(Clone, Copy)]
pub struct Pitch {
    value: u8,
}
impl Pitch {
    pub fn new(value: u8) -> Option<Self> {
        if value < 0x80 {
            Some(Self { value })
        } else {
            None
        }
    }

    pub fn get(&self) -> u8 {
        self.value
    }
    
    pub fn new_unchecked(value: u8) -> Self {
        Self { value }
    }

    pub fn get_freq(&self)-> f64{
        440.0 * 2.0_f64.powf((self.value as f64 - 69.0) / 12.0)
    }
}

// #[derive(Clone, Copy)]
// pub struct Pitchbend {
//     value: u16,
// }

// impl Pitchbend {
//     pub fn new(value: u16) -> Option<Self> {
//         if value < 0x4000 {
//             Some(Self { value })
//         } else {
//             None
//         }
//     }
//     pub fn new_unchecked(value: u16) -> Self {
//         Self { value }
//     }
// }

#[derive(Clone, Copy)]
pub struct Note{
    pub pitch: Pitch,
    pub onset: f64,
    pub duration: f64, 
    pub velocity: Velocity,
}