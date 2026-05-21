use intrusive_collections::{SinglyLinkedList, UnsafeRef};
use lasso::Rodeo;

use crate::object::{Object, ObjectAdapter, ObjectKind, OwnedObject, internal_str::InternalStr};

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

#[derive(Default)]
pub struct ObjectPool(SinglyLinkedList<ObjectAdapter>);

impl ObjectPool {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add<T: ObjectKind + ?Sized>(&mut self, obj: Box<T>) -> UnsafeRef<Object> {
        let raw = obj.upcast().into_raw();
        // SAFETY: `raw` is a unique, non-null owning pointer just produced by
        // `OwnedObject::into_raw`. Ownership transfers to the `UnsafeRef`,
        // which is then handed to the intrusive list; the pool's `Drop`
        // reclaims it. The pool is the sole originator of `UnsafeRef<Object>`
        // in the crate, which keeps the alloc tied to the pool's lifetime.
        let obj_ref = unsafe { UnsafeRef::from_raw(raw) };
        self.0.push_front(obj_ref.clone());
        obj_ref
    }
}

impl Drop for ObjectPool {
    fn drop(&mut self) {
        while let Some(obj_ref) = self.0.pop_front() {
            let raw = UnsafeRef::into_raw(obj_ref);
            // SAFETY: every entry in the list was inserted by `add`, which
            // wrapped the raw pointer from `OwnedObject::into_raw` after
            // upcasting a `Box<T: ObjectKind>`. So `raw` is the unique owning
            // pointer at drop time — callers of `add` are responsible for not
            // retaining the returned `UnsafeRef` past the pool's lifetime (the
            // standard `UnsafeRef` contract).
            drop(unsafe { OwnedObject::from_raw(raw) });
        }
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

    #[test]
    fn add_one_and_drop_pool() {
        let mut pool = ObjectPool::new();
        pool.add(StringObj::boxed("one"));
    }

    #[test]
    fn add_many_and_drop_pool() {
        let mut pool = ObjectPool::new();
        for i in 0..32 {
            pool.add(StringObj::boxed(&format!("str-{i}")));
        }
    }

    #[test]
    fn returned_ref_kind_is_string() {
        let mut pool = ObjectPool::new();
        let obj_ref = pool.add(StringObj::boxed("ref"));
        assert!(obj_ref.kind() == ObjKind::String);
    }

    #[test]
    fn returned_ref_is_alive_until_pool_drop() {
        // The UnsafeRef returned by `add` should remain valid for the pool's lifetime.
        let mut pool = ObjectPool::new();
        let obj_ref = pool.add(StringObj::boxed("alive"));
        // SAFETY: `obj_ref` was just produced from a `StringObj`, so its
        // dynamic kind is `StringObj`.
        let s = unsafe { obj_ref.downcast::<StringObj>() };
        assert_eq!(&**s, "alive");
        // Don't drop `s` — pool owns the alloc.
        let _ = UnsafeRef::into_raw(s);
    }
}
