use std::slice;

use crate::value::Value;

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

    /// Drop the top `n` values in one length-write. Used by `OP_POPN`.
    pub fn pop_n(&mut self, n: u8) {
        let new_len = self
            .inner
            .len()
            .checked_sub(n as usize)
            .expect("compiler bug, popping more values than stack holds");
        self.inner.truncate(new_len);
    }

    pub fn top(&self) -> &Value {
        self.inner
            .last()
            .expect("compiler bug, nothing on top of the VM stack")
    }

    pub fn top_mut(&mut self) -> &mut Value {
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

    /// Absolute-index read. `OP_GET_LOCAL` reads `frame.base + slot`; open
    /// upvalues read the captured stack index directly.
    pub fn at(&self, index: usize) -> &Value {
        self.inner
            .get(index)
            .expect("compiler bug, stack index out of range")
    }

    /// Absolute-index mutable access (`OP_SET_LOCAL`, `OP_SET_UPVALUE`).
    pub fn at_mut(&mut self, index: usize) -> &mut Value {
        self.inner
            .get_mut(index)
            .expect("compiler bug, stack index out of range")
    }

    /// Current height. A frame's `base` is the height captured just before its
    /// callee and arguments were pushed.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Drop everything at or above `index`. `OP_RETURN` truncates to the
    /// returning frame's `base` to discard its whole window in one write.
    pub fn truncate(&mut self, index: usize) {
        self.inner.truncate(index);
    }

    /// Empty the stack (start-of-run reset).
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// The values from `start` to the top, e.g. a native call's argument window.
    pub fn args_from(&self, start: usize) -> &[Value] {
        &self.inner[start..]
    }

    pub fn iter(&self) -> slice::Iter<'_, Value> {
        self.inner.iter()
    }
}
