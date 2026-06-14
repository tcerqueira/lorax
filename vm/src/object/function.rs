use std::fmt::Display;

use lasso::Spur;

use crate::{
    chunk::Chunk,
    object::{Object, ObjectType},
    storage::WithStorage,
};

#[repr(C)]
#[derive(Debug)]
pub struct LoxFunction {
    obj: Object,
    chunk: Chunk,
    name: Spur,
    arity: u8,
}

// SAFETY: `LoxFunction` is `#[repr(C)]` with `Object` (`obj`) as its first
// field, so an `Object` header at offset 0 is layout-compatible. Construction
// goes through `Self::new`, which sets `obj.kind = ObjKind::Function`.
unsafe impl ObjectType for LoxFunction {}

impl LoxFunction {
    pub fn new(name: Spur, arity: u8, chunk: Chunk) -> Self {
        Self {
            obj: Object::function(),
            chunk,
            name,
            arity,
        }
    }
}

impl Display for LoxFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn Symbol({})>", self.name.into_inner())
    }
}

impl Display for WithStorage<'_, LoxFunction> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn {}>", self.1.resolve(self.0.name))
    }
}
