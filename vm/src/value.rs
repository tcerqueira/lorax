use std::{
    fmt::{self, Debug, Display},
    ops::{Add, Div, Mul, Neg, Sub},
};

pub struct ValueError;

#[derive(Debug, Clone, Copy)]
pub enum Value {
    Nil,
    Boolean(bool),
    Number(f64),
}

impl Value {
    pub fn nil() -> Self {
        Self::Nil
    }

    pub fn boolean(value: bool) -> Self {
        Self::Boolean(value)
    }

    pub fn number(value: f64) -> Self {
        Self::Number(value)
    }
}

impl Neg for Value {
    type Output = Result<Self, ValueError>;

    fn neg(self) -> Self::Output {
        match self {
            Self::Number(n) => Ok(Self::number(-n)),
            _ => Err(ValueError),
        }
    }
}

impl Add for Value {
    type Output = Result<Self, ValueError>;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Number(x), Self::Number(y)) => Ok(Self::number(x + y)),
            _ => Err(ValueError),
        }
    }
}

impl Sub for Value {
    type Output = Result<Self, ValueError>;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Number(x), Self::Number(y)) => Ok(Self::number(x - y)),
            _ => Err(ValueError),
        }
    }
}

impl Mul for Value {
    type Output = Result<Self, ValueError>;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Number(x), Self::Number(y)) => Ok(Self::number(x * y)),
            _ => Err(ValueError),
        }
    }
}

impl Div for Value {
    type Output = Result<Self, ValueError>;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Number(x), Self::Number(y)) => Ok(Self::number(x / y)),
            _ => Err(ValueError),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Boolean(b) => write!(f, "{b}"),
            Value::Number(n) => write!(f, "{n}"),
        }
    }
}

pub type Addr = u8;
