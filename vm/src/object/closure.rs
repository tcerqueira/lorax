use std::fmt::{self, Display, Formatter};

use intrusive_collections::UnsafeRef;

use crate::{
    chunk::Chunk,
    object::{Object, ObjectType, function::LoxFunction},
    storage::WithStorage,
};

/// A function paired with the upvalues it captured. Every callable on the stack
/// is a closure (even those that capture nothing), so the call path has one
/// shape. The upvalue array is write-once — filled at `OP_CLOSURE` and never
/// mutated — so only the `LoxUpvalue`s it points at need interior mutability.
#[repr(C)]
#[derive(Debug)]
pub struct LoxClosure {
    obj: Object,
    function: UnsafeRef<Object>,
    upvalues: Box<[UnsafeRef<Object>]>,
}

// SAFETY: `#[repr(C)]` with `Object` first; `Self::new` sets `obj.kind = Closure`.
unsafe impl ObjectType for LoxClosure {}

impl LoxClosure {
    pub fn new(function: UnsafeRef<Object>, upvalues: Box<[UnsafeRef<Object>]>) -> Self {
        Self {
            obj: Object::closure(),
            function,
            upvalues,
        }
    }

    fn function(&self) -> &LoxFunction {
        // SAFETY: a closure always wraps a LoxFunction.
        unsafe { self.function.downcast_ref::<LoxFunction>() }
    }

    pub fn chunk(&self) -> &Chunk {
        self.function().chunk()
    }

    pub fn arity(&self) -> u8 {
        self.function().arity()
    }

    pub fn upvalue(&self, index: u8) -> &UnsafeRef<Object> {
        &self.upvalues[index as usize]
    }

    // GC trace edges.
    pub fn function_handle(&self) -> &UnsafeRef<Object> {
        &self.function
    }

    pub fn upvalues(&self) -> &[UnsafeRef<Object>] {
        &self.upvalues
    }
}

impl Display for LoxClosure {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(self.function(), f)
    }
}

impl Display for WithStorage<'_, LoxClosure> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // A closure prints exactly like the function it wraps.
        WithStorage(self.0.function(), self.1).fmt(f)
    }
}
