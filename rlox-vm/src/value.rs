use std::{
    fmt::{self, Debug, Display},
    ops::{Add, Div, Mul, Neg, Sub},
};

#[derive(Debug, Clone, Copy)]
pub struct Value(f64);

impl Value {
    pub fn new(value: f64) -> Self {
        Self(value)
    }
}

impl Neg for Value {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Value(-self.0)
    }
}

impl Add for Value {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Value(self.0 + rhs.0)
    }
}

impl Sub for Value {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Value(self.0 - rhs.0)
    }
}

impl Mul for Value {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Value(self.0 * rhs.0)
    }
}

impl Div for Value {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Value(self.0 / rhs.0)
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub type Addr = u8;
