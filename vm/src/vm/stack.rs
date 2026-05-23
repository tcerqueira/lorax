use std::slice;

use crate::value::Value;

#[derive(Default)]
pub struct Stack {
    inner: Vec<Value>,
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

    pub fn iter(&self) -> slice::Iter<'_, Value> {
        self.inner.iter()
    }
}
