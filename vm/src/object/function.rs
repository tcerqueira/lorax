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
    /// How many upvalues a closure over this function captures. `u16` because
    /// the limit is 256 (the full `Slot = u8` upvalue-index space).
    upvalue_count: u16,
}

// SAFETY: `LoxFunction` is `#[repr(C)]` with `Object` (`obj`) as its first
// field, so an `Object` header at offset 0 is layout-compatible. Construction
// goes through `Self::new`, which sets `obj.kind = ObjKind::Function`.
unsafe impl ObjectType for LoxFunction {}

impl LoxFunction {
    pub fn new(name: Spur, arity: u8, upvalue_count: u16, chunk: Chunk) -> Self {
        Self {
            obj: Object::function(),
            chunk,
            name,
            arity,
            upvalue_count,
        }
    }

    pub fn chunk(&self) -> &Chunk {
        &self.chunk
    }

    pub fn name(&self) -> Spur {
        self.name
    }

    pub fn arity(&self) -> u8 {
        self.arity
    }

    pub fn upvalue_count(&self) -> u16 {
        self.upvalue_count
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
