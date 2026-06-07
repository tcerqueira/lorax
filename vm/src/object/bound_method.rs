use std::fmt::{self, Display, Formatter};

use intrusive_collections::UnsafeRef;

use crate::{
    object::{Object, ObjectType, closure::LoxClosure},
    storage::WithStorage,
    value::Value,
};

/// A method paired with the receiver it was accessed on, produced by reading a
/// method off an instance. Immutable once created, and prints exactly like the
/// closure it wraps.
#[repr(C)]
#[derive(Debug)]
pub struct LoxBoundMethod {
    obj: Object,
    receiver: Value,
    method: UnsafeRef<Object>,
}

// SAFETY: `#[repr(C)]` with `Object` first; `Self::new` sets `obj.kind = BoundMethod`.
unsafe impl ObjectType for LoxBoundMethod {}

impl LoxBoundMethod {
    pub fn new(receiver: Value, method: UnsafeRef<Object>) -> Self {
        Self {
            obj: Object::bound_method(),
            receiver,
            method,
        }
    }

    pub fn receiver(&self) -> &Value {
        &self.receiver
    }

    pub fn method(&self) -> &UnsafeRef<Object> {
        &self.method
    }

    fn closure(&self) -> &LoxClosure {
        // SAFETY: a bound method always wraps a LoxClosure.
        unsafe { self.method.downcast_ref::<LoxClosure>() }
    }
}

impl Display for LoxBoundMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(self.closure(), f)
    }
}

impl Display for WithStorage<'_, LoxBoundMethod> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        WithStorage(self.0.closure(), self.1).fmt(f)
    }
}
