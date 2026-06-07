use std::fmt::{self, Display, Formatter};

use lasso::Spur;
use report::error::RuntimeError;

use crate::{
    object::{Object, ObjectType},
    storage::{Storage, WithStorage},
    value::Value,
};

/// A built-in function implemented in Rust. The argument slice is the call's
/// window on the value stack; `&mut Storage` lets a native allocate result
/// objects. A bare `fn` pointer (not `Box<dyn Fn>`) keeps `LoxNative` sized and
/// vtable-free — natives that need state (e.g. `clock`'s baseline) reach for a
/// module-level `OnceLock`.
pub type NativeFn = fn(&mut Storage, &[Value]) -> Result<Value, RuntimeError>;

#[repr(C)]
#[derive(Debug)]
pub struct LoxNative {
    obj: Object,
    name: Spur,
    func: NativeFn,
}

// SAFETY: `#[repr(C)]` with `Object` first; `Self::new` sets `obj.kind = Native`.
unsafe impl ObjectType for LoxNative {}

impl LoxNative {
    pub fn new(name: Spur, func: NativeFn) -> Self {
        Self {
            obj: Object::native(),
            name,
            func,
        }
    }

    pub fn func(&self) -> NativeFn {
        self.func
    }
}

impl Display for LoxNative {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "<native fn Symbol({})>", self.name.into_inner())
    }
}

impl Display for WithStorage<'_, LoxNative> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "<native fn {}>", self.1.resolve(self.0.name))
    }
}
