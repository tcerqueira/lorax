use std::fmt::{self, Debug, Display};

#[derive(Debug, Clone, Copy)]
pub struct Value(f64);

impl Value {
    pub fn new(value: f64) -> Self {
        Self(value)
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub type Addr = u8;
