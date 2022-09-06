use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    Loop,
    Type,
    Existance,
    Overwrite,
    Unbound,
    Overflow,
    Reciever,
    Unintialised,
    RecieverMisMatch,
    Value,
    Parse,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("my error") // TODO
    }
}

impl std::error::Error for Error {}

// #[derive(Debug)]
// enum ErrorKind {
//     Range {
//         value: f64,
//         range: (f64, f64),
//     },
//     RangeMismatch {
//         trg_range: (f64, f64),
//         src_range: (f64, f64),
//     },
//     CircRef {
//         id: usize,
//     },
//     DoubleId {
//         id: usize,
//     },
//     FNotFound {
//         id: usize,
//     },
//     PhantomF,
// }

// #[derive(Debug)]
// pub struct ControlError {
//     path: Vec<String>,
//     origin: String,
//     control: String,
//     kind: ErrorKind,
// }

// impl ControlError {
//     pub fn new_range_err(value: f64, range: (f64, f64)) -> Self {
//         Self {
//             path: Vec::new(),
//             origin: String::new(),
//             control: String::new(),
//             kind: ErrorKind::Range { value, range },
//         }
//     }

//     pub fn new_range_mismatch_err(trg_range: (f64, f64), src_range: (f64, f64)) -> Self {
//         Self {
//             path: Vec::new(),
//             origin: String::new(),
//             control: String::new(),
//             kind: ErrorKind::RangeMismatch {
//                 trg_range,
//                 src_range,
//             },
//         }
//     }

//     pub fn new_circ_ref_err(id: usize) -> Self {
//         Self {
//             path: Vec::new(),
//             origin: String::new(),
//             control: String::new(),
//             kind: ErrorKind::CircRef { id },
//         }
//     }

//     pub fn new_double_id_err(id: usize) -> Self {
//         Self {
//             path: Vec::new(),
//             origin: String::new(),
//             control: String::new(),
//             kind: ErrorKind::DoubleId { id },
//         }
//     }

//     pub fn new_func_not_found(id: usize) -> Self {
//         Self {
//             path: Vec::new(),
//             origin: String::new(),
//             control: String::new(),
//             kind: ErrorKind::FNotFound { id },
//         }
//     }

//     pub fn new_phantom_f_err() -> Self {
//         Self {
//             path: Vec::new(),
//             origin: String::new(),
//             control: String::new(),
//             kind: ErrorKind::PhantomF,
//         }
//     }

//     pub fn set_origin(mut self, origin: &str, control: &str) -> Self {
//         self.origin.push_str(origin);
//         self.control.push_str(control);
//         self
//     }

//     pub fn push_location(mut self, location: &str) -> Self {
//         self.path.push(location.to_string());
//         self
//     }

//     fn get_string(&self) -> String {
//         match &self.kind {
//             ErrorKind::Range { value, range } => {
//                 format!(
//                     "The value of {} in {} was set to {}, which is not in range from {} to {}!\n    full path to value: {}/{}/{}/{}",
//                     self.control,
//                     self.origin,
//                     value,
//                     range.0,
//                     range.1,
//                     self.path.join("/"),
//                     self.origin,
//                     self.control,
//                     value,
//                 )
//             }
//             ErrorKind::CircRef { id } => {
//                 format!(
//                     "You tried to set {} in {} to {} this leeds to a circular Reference between CtrlFunctions, which is not allowed!\n    full path to control: {}/{}/{}",
//                     self.control,
//                     self.origin,
//                     id,
//                     self.path.join("/"),
//                     self.origin,
//                     self.control,
//                 )
//             }
//             ErrorKind::RangeMismatch {
//                 trg_range,
//                 src_range,
//             } => {
//                 format!(
//                     "You tried to set {} in {} which has a range of ({}, {}) to a control with a range of ({}, {})!\n    full path to control: {}/{}/{}",
//                     self.control,
//                     self.origin,
//                     trg_range.0,
//                     trg_range.1,
//                     src_range.0,
//                     src_range.1,
//                     self.path.join("/"),
//                     self.origin,
//                     self.control,
//                 )
//             }
//             ErrorKind::DoubleId { id } => {
//                 format!(
//                     "The id : {}, was encountered twice while creating IdMap",
//                     id
//                 )
//             }
//             ErrorKind::FNotFound { id } => {
//                 format!(
//                     "The source for {} in {} could not find the function with id: {}!\n    full path to control: {}/{}/{}",
//                     self.control,
//                     self.origin,
//                     id,
//                     self.path.join("/"),
//                     self.origin,
//                     self.control,
//                 )
//             }
//             ErrorKind::PhantomF =>
//             format!(
//                 "The source for {} in {} has no assigned CtrlFunction!\n    full path to control: {}/{}/{}",
//                 self.control,
//                 self.origin,
//                 self.path.join("/"),
//                 self.origin,
//                 self.control,
//             ),
//         }
//     }
// }

// impl Display for ControlError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}", self.get_string())
//     }
// }
