use std::cell::RefCell;

use crate::{
    object::{Object, ObjectType},
    value::Value,
};

/// A captured variable. While the variable is still live on the value stack the
/// upvalue is `Open` and holds its absolute stack index (an index, not a raw
/// pointer, so a reallocating stack can't dangle it); once the variable's frame
/// or block exits the upvalue is `Closed` and owns the value itself.
#[derive(Debug)]
enum UpvalueState {
    Open(usize),
    Closed(Value),
}

/// Reached only through shared `UnsafeRef`s (sibling closures alias one
/// upvalue), so its mutable state lives behind a `RefCell` — never a `&mut`.
#[repr(C)]
#[derive(Debug)]
pub struct LoxUpvalue {
    obj: Object,
    state: RefCell<UpvalueState>,
}

// SAFETY: `#[repr(C)]` with `Object` first; `Self::open` sets `obj.kind = Upvalue`.
unsafe impl ObjectType for LoxUpvalue {}

impl LoxUpvalue {
    pub fn open(stack_index: usize) -> Self {
        Self {
            obj: Object::upvalue(),
            state: RefCell::new(UpvalueState::Open(stack_index)),
        }
    }

    /// The captured stack index while still open, else `None`.
    pub fn open_index(&self) -> Option<usize> {
        match &*self.state.borrow() {
            UpvalueState::Open(index) => Some(*index),
            UpvalueState::Closed(_) => None,
        }
    }

    /// The owned value once closed, else `None`.
    pub fn closed_value(&self) -> Option<Value> {
        match &*self.state.borrow() {
            UpvalueState::Closed(value) => Some(value.clone()),
            UpvalueState::Open(_) => None,
        }
    }

    /// Move into the closed state (or overwrite the closed value on assignment).
    pub fn close(&self, value: Value) {
        *self.state.borrow_mut() = UpvalueState::Closed(value);
    }
}
