use std::cell::RefCell;
use std::fmt::{self, Display, Formatter};

use intrusive_collections::UnsafeRef;
use lasso::Spur;

use crate::{
    object::{Object, ObjectType, class::LoxClass},
    storage::{SymbolMap, WithStorage},
    value::Value,
};

/// An instance of a class. `fields` is a `RefCell` because property get/set
/// mutate it through a shared `UnsafeRef` (the stack value aliases the same
/// handle); the borrows are short and never re-entrant, so it can't panic.
#[repr(C)]
#[derive(Debug)]
pub struct LoxInstance {
    obj: Object,
    class: UnsafeRef<Object>,
    fields: RefCell<SymbolMap<Value>>,
}

// SAFETY: `#[repr(C)]` with `Object` first; `Self::new` sets `obj.kind = Instance`.
unsafe impl ObjectType for LoxInstance {}

impl LoxInstance {
    pub fn new(class: UnsafeRef<Object>) -> Self {
        Self {
            obj: Object::instance(),
            class,
            fields: RefCell::new(SymbolMap::default()),
        }
    }

    fn class(&self) -> &LoxClass {
        // SAFETY: an instance's class handle always points to a LoxClass.
        unsafe { self.class.downcast_ref::<LoxClass>() }
    }

    pub fn field(&self, name: Spur) -> Option<Value> {
        self.fields.borrow().get(&name).cloned()
    }

    /// Look up a method on this instance's class (no field shadowing — callers
    /// check fields first).
    pub fn find_method(&self, name: Spur) -> Option<Value> {
        self.class().method(name)
    }

    // GC trace edges: the class handle and every field value.
    pub fn class_handle(&self) -> &UnsafeRef<Object> {
        &self.class
    }

    pub fn trace_fields(&self, mut visit: impl FnMut(&Value)) {
        for value in self.fields.borrow().values() {
            visit(value);
        }
    }

    pub fn set_field(&self, name: Spur, value: Value) {
        self.fields.borrow_mut().insert(name, value);
    }
}

impl Display for LoxInstance {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} instance", self.class())
    }
}

impl Display for WithStorage<'_, LoxInstance> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} instance", WithStorage(self.0.class(), self.1))
    }
}
