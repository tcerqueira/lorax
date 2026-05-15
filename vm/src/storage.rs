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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::{ObjKind, string::StringObj};

    #[test]
    fn add_obj_routes_through_pool() {
        let mut storage = Storage::new();
        let obj_ref = storage.add_obj(StringObj::boxed("via add_obj"));
        assert_eq!(obj_ref.kind(), ObjKind::String);
        // Pool owns the alloc; release our handle without freeing.
        let _ = UnsafeRef::into_raw(obj_ref);
    }
}
