use std::{
    any::Any,
    cmp::Ordering,
    fmt::{self, Debug, Display, Formatter},
    ops::{Add, Div, Mul, Neg, Not, Sub},
    rc::Rc,
};

use thiserror::Error;

use crate::runtime::callable::{Function, NativeFunction, ObjCallable};

pub trait AnyExt {
    fn type_name(&self) -> &'static str;
}

impl<T: Any + ?Sized> AnyExt for T {
    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

pub trait ObjDebug {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result;
}

impl<T: Debug> ObjDebug for T {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

pub trait ObjDisplay {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result;
}

impl<T: Display> ObjDisplay for T {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(self, f)
    }
}

pub trait Nil {
    fn is_nil(&self) -> bool;
}

impl<T> Nil for Option<T> {
    fn is_nil(&self) -> bool {
        self.is_none()
    }
}

pub trait ObjPartialEq {
    fn eq(&self, other: &dyn Any) -> bool;
}

impl<T: PartialEq + Any> ObjPartialEq for T {
    fn eq(&self, other: &dyn Any) -> bool {
        other.downcast_ref::<T>().is_some_and(|x| self.eq(x))
    }
}

pub trait ObjPartialOrd {
    fn partial_cmp(&self, other: &dyn Any) -> Option<Ordering>;
}

impl<T: PartialOrd + Any> ObjPartialOrd for T {
    fn partial_cmp(&self, other: &dyn Any) -> Option<Ordering> {
        other.downcast_ref::<T>().and_then(|x| self.partial_cmp(x))
    }
}

pub trait ObjectInternal:
    Any + AnyExt + ObjPartialEq + ObjPartialOrd + ObjDebug + ObjDisplay
{
}
impl<T: Any + AnyExt + ObjPartialEq + ObjPartialOrd + ObjDebug + ObjDisplay> ObjectInternal for T {}

#[derive(Clone)]
pub struct Object(Option<Rc<dyn ObjectInternal>>);

impl Object {
    pub fn new(value: impl ObjectInternal) -> Self {
        Self(Some(Rc::new(value)))
    }

    pub fn nil() -> Self {
        Self(None)
    }

    #[allow(dead_code)]
    pub fn downcast<T: Any>(&self) -> &T {
        (self.0.as_deref().unwrap() as &dyn Any)
            .downcast_ref::<T>()
            .unwrap()
    }

    pub fn try_downcast<T: Any>(&self) -> Result<&T, DowncastError<T>> {
        match self.0.as_deref() {
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

    pub fn as_callable(&self) -> Option<&dyn ObjCallable> {
        if let Ok(callable) = self.try_downcast::<Function>() {
            Some(callable as &dyn ObjCallable)
        } else if let Ok(callable) = self.try_downcast::<NativeFunction>() {
            Some(callable as &dyn ObjCallable)
        } else {
            None
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

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (None, None) => true,
            (Some(this), Some(other)) => this.eq(other.as_ref()),
            _ => false,
        }
    }
}

impl PartialOrd for Object {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (&self.0, &other.0) {
            (None, _) | (_, None) => None,
            (Some(this), Some(other)) => this.partial_cmp(other.as_ref()),
        }
    }
}

impl Debug for Object {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Object {{ ")?;
        match &self.0 {
            None => write!(f, "type: <unknown>, value: nil")?,
            Some(obj) => {
                write!(f, "type: ")?;
                write!(f, "{}", AnyExt::type_name(obj.as_ref()))?;
                write!(f, ", value: ")?;
                ObjDebug::fmt(obj.as_ref(), f)?;
            }
        }
        write!(f, " }}")
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            None => write!(f, "nil"),
            Some(obj) => ObjDisplay::fmt(obj.as_ref(), f),
        }
    }
}

impl Nil for Object {
    fn is_nil(&self) -> bool {
        self.0.is_nil()
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
