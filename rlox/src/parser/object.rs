use std::{
    any::Any,
    fmt::{Debug, Display},
    ops::{Add, Deref, DerefMut, Div, Mul, Neg, Not, Sub},
    rc::Rc,
};

use thiserror::Error;

pub trait Nil {
    fn is_nil(&self) -> bool;
}

impl<T> Nil for Option<T> {
    fn is_nil(&self) -> bool {
        self.is_none()
    }
}

pub trait ObjectInner: Any + Debug + Display {}
impl<T: Any + Debug + Display> ObjectInner for T {}

#[derive(Debug, Clone)]
pub struct Object(Option<Rc<dyn ObjectInner>>);

impl Object {
    pub fn new(value: impl ObjectInner) -> Self {
        Self(Some(Rc::new(value)))
    }

    pub fn nil() -> Self {
        Self(None)
    }

    #[allow(dead_code)]
    pub fn downcast<T: Any>(&self) -> &T {
        (self.as_deref().unwrap() as &dyn Any)
            .downcast_ref::<T>()
            .unwrap()
    }

    pub fn try_downcast<T: Any>(&self) -> Result<&T, DowncastError<T>> {
        match self.as_deref() {
            None => Err(DowncastError::new(true)),
            Some(obj) => (obj as &dyn Any)
                .downcast_ref::<T>()
                .ok_or(DowncastError::new(false)),
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self.try_downcast::<bool>() {
            Ok(boolean) => *boolean,
            Err(e) if e.is_nil() => false,
            Err(_) => true,
        }
    }
}

impl Add for Object {
    type Output = Result<Object, OpError>;

    fn add(self, rhs: Self) -> Self::Output {
        if let (Ok(left), Ok(right)) = (self.try_downcast::<f64>(), rhs.try_downcast::<f64>()) {
            return Ok(Object::new(left + right));
        }
        if let (Ok(left), Ok(right)) = (self.try_downcast::<String>(), rhs.try_downcast::<String>())
        {
            return Ok(Object::new(format!("{left}{right}")));
        }
        Err(OpError::InvalidOperand(
            "Objects not both String or f64".into(),
        ))
    }
}

impl Sub for Object {
    type Output = Result<Object, OpError>;

    fn sub(self, rhs: Self) -> Self::Output {
        let left = self.try_downcast::<f64>()?;
        let right = rhs.try_downcast::<f64>()?;
        Ok(Object::new(left - right))
    }
}

impl Mul for Object {
    type Output = Result<Object, OpError>;

    fn mul(self, rhs: Self) -> Self::Output {
        let left = self.try_downcast::<f64>()?;
        let right = rhs.try_downcast::<f64>()?;
        Ok(Object::new(left * right))
    }
}

impl Div for Object {
    type Output = Result<Object, OpError>;

    fn div(self, rhs: Self) -> Self::Output {
        let left = self.try_downcast::<f64>()?;
        let right = rhs.try_downcast::<f64>()?;
        Ok(Object::new(left / right))
    }
}

impl Neg for Object {
    type Output = Result<Object, OpError>;

    fn neg(self) -> Self::Output {
        Ok(Object::new(-self.try_downcast::<f64>()?))
    }
}

impl Not for Object {
    type Output = Object;

    fn not(self) -> Self::Output {
        Object::new(!self.is_truthy())
    }
}

#[derive(Debug, Error)]
pub enum OpError {
    #[error("Invalid operand: {}", .0)]
    InvalidOperand(String),
}

impl<T> From<DowncastError<T>> for OpError {
    fn from(err: DowncastError<T>) -> Self {
        OpError::InvalidOperand(err.to_string())
    }
}

#[derive(Debug, Error)]
pub struct DowncastError<T> {
    is_nil: bool,
    _p: std::marker::PhantomData<T>,
}

impl<T> DowncastError<T> {
    fn new(is_nil: bool) -> Self {
        Self {
            is_nil,
            _p: std::marker::PhantomData,
        }
    }

    pub fn is_nil(&self) -> bool {
        self.is_nil
    }
}

impl<T> Display for DowncastError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.is_nil() {
            true => write!(f, "Object is nil"),
            false => write!(f, "Object is not of type {}", std::any::type_name::<T>()),
        }
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.is_nil() {
            true => writeln!(f, "nil"),
            false => writeln!(f, "{}", self.as_deref().unwrap()),
        }
    }
}

impl Nil for Object {
    fn is_nil(&self) -> bool {
        self.0.is_nil()
    }
}

impl Deref for Object {
    type Target = Option<Rc<dyn ObjectInner>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Object {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
