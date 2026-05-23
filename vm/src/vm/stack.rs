use std::slice;

use crate::{opcode::Slot, value::Value};

// PERF: all accessors (`top`, `top_mut`, `peek`, `peek_mut`, `get`, `get_mut`,
// `pop`) bounds-check on every dispatch. The compiler guarantees the indices
// are in range, so a `debug_assert!` + `get_unchecked` pair could shave the
// check from the inner loop once benchmarks justify it.
pub struct Stack {
    inner: Vec<Value>,
}

impl Default for Stack {
    fn default() -> Self {
        // Avoid early-push reallocs. 256 matches the local-slot space; once
        // call frames land, pick a bigger default scaled to frame budget.
        Self {
            inner: Vec::with_capacity(256),
        }
    }
}

impl Stack {
    pub fn push(&mut self, value: Value) {
        self.inner.push(value);
    }

    pub fn pop(&mut self) -> Value {
        self.inner
            .pop()
            .expect("compiler bug, nothing to pop on the VM stack")
    }

    pub fn top(&self) -> &Value {
        // optimization for ops that pop 1 value and push 1 value
        // allows mutation in place
        self.inner
            .last()
            .expect("compiler bug, nothing on top of the VM stack")
    }

    pub fn top_mut(&mut self) -> &mut Value {
        // optimization for ops that pop 1 value and push 1 value
        // allows mutation in place
        self.inner
            .last_mut()
            .expect("compiler bug, nothing on top of the VM stack")
    }

    pub fn peek(&self, distance: usize) -> &Value {
        let len = self.inner.len();
        self.inner
            .get(len - distance - 1)
            .expect("compiler bug, nothing to peek on the VM stack")
    }

    pub fn peek_mut(&mut self, distance: usize) -> &mut Value {
        let len = self.inner.len();
        self.inner
            .get_mut(len - distance - 1)
            .expect("compiler bug, nothing to peek on the VM stack")
    }

    /// Absolute-slot read. Used by `OP_GET_LOCAL` — local slot `n` lives at
    /// stack index `n` (no call frames yet; when frames land this becomes
    /// `frame.base + n`).
    pub fn get(&self, slot: Slot) -> &Value {
        self.inner
            .get(slot as usize)
            .expect("compiler bug, local slot out of range")
    }

    /// Absolute-slot mutable access. Used by `OP_SET_LOCAL`.
    pub fn get_mut(&mut self, slot: Slot) -> &mut Value {
        self.inner
            .get_mut(slot as usize)
            .expect("compiler bug, local slot out of range")
    }

    pub fn iter(&self) -> slice::Iter<'_, Value> {
        self.inner.iter()
    }
}
