use std::{
    cmp::Ordering,
    fmt::{self, Debug, Display},
    mem,
    ops::{Add, Div, Mul, Neg, Sub},
};

use intrusive_collections::UnsafeRef;
use lasso::Spur;

use crate::{
    object::{ObjKind, Object},
    storage::Storage,
};

pub struct ValueError;

#[derive(Debug, Clone)]
pub enum Value {
    Nil,
    Boolean(bool),
    Number(f64),
    Symbol(Spur),
    Object(UnsafeRef<Object>),
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

    pub fn symbol(key: Spur) -> Self {
        Self::Symbol(key)
    }

    pub fn object(value: UnsafeRef<Object>) -> Self {
        Self::Object(value)
    }

    pub fn is_falsey(&self) -> bool {
        !match self {
            Self::Boolean(b) => *b,
            Self::Nil => false,
            _ => false,
        }
    }

    pub fn is_str(&self) -> bool {
        match self {
            Self::Symbol(_) => true,
            Self::Object(o) => o.kind() == ObjKind::String,
            _ => false,
        }
    }

    pub fn as_str<'s>(&'s self, storage: &'s Storage) -> &'s str {
        match self {
            Self::Symbol(key) => storage.resolve(*key),
            Self::Object(o) if o.kind() == ObjKind::String => o.as_str(),
            _ => panic!("Value::as_str called on non-string {self:?}"),
        }
    }

    pub fn greater(self, other: Self) -> Result<Self, ValueError> {
        Self::partial_cmp(&self, &other)
            .map(|ord| Self::boolean(ord == Ordering::Greater))
            .ok_or(ValueError)
    }

    pub fn less(self, other: Self) -> Result<Self, ValueError> {
        Self::partial_cmp(&self, &other)
            .map(|ord| Self::boolean(ord == Ordering::Less))
            .ok_or(ValueError)
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

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        if mem::discriminant(self) != mem::discriminant(other) {
            return false;
        }
        match (self, other) {
            (Self::Nil, Self::Nil) => true,
            (Self::Boolean(a), Self::Boolean(b)) => a == b,
            (Self::Number(a), Self::Number(b)) => a == b,
            (Self::Symbol(a), Self::Symbol(b)) => a == b,
            (Self::Object(a), Self::Object(b)) => a.eq(b),
            _ => unreachable!("missing impl for PartialEq"),
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if mem::discriminant(self) != mem::discriminant(other) {
            return None;
        }
        match (self, other) {
            (Value::Nil, Value::Nil) => Some(Ordering::Equal),
            (Value::Boolean(a), Value::Boolean(b)) => a.partial_cmp(b),
            (Value::Number(a), Value::Number(b)) => a.partial_cmp(b),
            _ => unreachable!("missing impl for PartialOrd"),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Boolean(b) => write!(f, "{b}"),
            Value::Number(n) => write!(f, "{n}"),
            Value::Symbol(key) => write!(f, "Symbol({})", key.into_inner()),
            Value::Object(obj) => obj.display_fmt(f),
        }
    }
}

pub type Addr = u8;
