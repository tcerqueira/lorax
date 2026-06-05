use intrusive_collections::{SinglyLinkedList, UnsafeRef};
use lasso::{Rodeo, Spur};

use crate::object::{Object, ObjectAdapter, ObjectType, OwnedObject};

/// A borrowed value paired with the `Storage` needed to render its objects.
pub struct WithStorage<'a, T: ?Sized>(pub &'a T, pub &'a Storage);

/// Owns the runtime heap (object pool) and the string interner.
#[derive(Default)]
pub struct Storage {
    heap: ObjectPool,
    strings: Rodeo,
}

impl Storage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn intern(&mut self, s: &str) -> Spur {
        self.strings.get_or_intern(s)
    }

    pub fn resolve(&self, key: Spur) -> &str {
        self.strings.resolve(&key)
    }

    pub fn add_obj<T: ObjectType + ?Sized>(&mut self, obj: Box<T>) -> UnsafeRef<Object> {
        self.heap.add(obj)
    }
}

#[derive(Default)]
pub struct ObjectPool(SinglyLinkedList<ObjectAdapter>);

impl ObjectPool {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add<T: ObjectType + ?Sized>(&mut self, obj: Box<T>) -> UnsafeRef<Object> {
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
    use crate::object::{ObjKind, string::LoxString};

    #[test]
    fn add_obj_routes_through_pool() {
        let mut storage = Storage::new();
        let obj_ref = storage.add_obj(LoxString::boxed("via add_obj"));
        assert_eq!(obj_ref.kind(), ObjKind::String);
        // Pool owns the alloc; release our handle without freeing.
        let _ = UnsafeRef::into_raw(obj_ref);
    }

    #[test]
    fn add_one_and_drop_pool() {
        let mut pool = ObjectPool::new();
        pool.add(LoxString::boxed("one"));
    }

    #[test]
    fn add_many_and_drop_pool() {
        let mut pool = ObjectPool::new();
        for i in 0..32 {
            pool.add(LoxString::boxed(&format!("str-{i}")));
        }
    }

    #[test]
    fn returned_ref_kind_is_string() {
        let mut pool = ObjectPool::new();
        let obj_ref = pool.add(LoxString::boxed("ref"));
        assert!(obj_ref.kind() == ObjKind::String);
    }

    #[test]
    fn returned_ref_is_alive_until_pool_drop() {
        // The UnsafeRef returned by `add` should remain valid for the pool's lifetime.
        let mut pool = ObjectPool::new();
        let obj_ref = pool.add(LoxString::boxed("alive"));
        // SAFETY: `obj_ref` was just produced from a `LoxString`, so its
        // dynamic kind is `LoxString`.
        let s = unsafe { obj_ref.downcast::<LoxString>() };
        assert_eq!(&**s, "alive");
        // Don't drop `s` — pool owns the alloc.
        let _ = UnsafeRef::into_raw(s);
    }
}
