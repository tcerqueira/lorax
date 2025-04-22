use std::{
    any::Any,
    fmt::{Debug, Display},
    ops::{Deref, DerefMut},
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

pub trait ObjectExt: Any + Debug + Display {}
impl<T: Any + Debug + Display> ObjectExt for T {}

#[derive(Debug, Clone)]
pub struct Object(Option<Rc<dyn ObjectExt>>);

impl Object {
    pub fn new(value: impl ObjectExt) -> Self {
        Self(Some(Rc::new(value)))
    }

    pub fn nil() -> Self {
        Self(None)
    }

    #[expect(dead_code)]
    pub fn downcast<T: 'static>(&self) -> &T {
        (self.as_deref().unwrap() as &dyn Any)
            .downcast_ref::<T>()
            .unwrap()
    }

    pub fn try_downcast<T: 'static>(&self) -> Result<&T, DowncastError<T>> {
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
            true => writeln!(f, "Object is nil"),
            false => writeln!(f, "Object is not of type {}", std::any::type_name::<T>()),
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
    type Target = Option<Rc<dyn ObjectExt>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Object {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
