use intrusive_collections::UnsafeRef;
use lasso::Rodeo;

use crate::object::{Object, ObjectKind, internal_str::InternalStr, pool::ObjectPool};

#[derive(Default)]
pub struct Storage {
    pub heap: ObjectPool,
    pub strings: Rodeo,
}

impl Storage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_obj<T: ObjectKind + ?Sized>(&mut self, obj: Box<T>) -> UnsafeRef<Object> {
        self.heap.add(obj)
    }

    pub fn add_internal_str(&mut self, s: &str) -> UnsafeRef<Object> {
        let internal_str = Box::new(InternalStr::new(&mut self.strings, s));
        self.heap.add(internal_str)
    }
}
