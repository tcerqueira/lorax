use intrusive_collections::{SinglyLinkedList, UnsafeRef};

use crate::object::{Object, ObjectAdapter, ObjectKind, OwnedObject};

#[derive(Default)]
pub struct ObjectPool(SinglyLinkedList<ObjectAdapter>);

impl ObjectPool {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add<T: ObjectKind + ?Sized>(&mut self, obj: Box<T>) -> UnsafeRef<Object> {
        let obj_ref = obj.upcast().into_ref();
        self.0.push_front(obj_ref.clone());
        obj_ref
    }
}

impl Drop for ObjectPool {
    fn drop(&mut self) {
        while let Some(obj_ref) = self.0.pop_front() {
            let raw = UnsafeRef::into_raw(obj_ref);
            // SAFETY: every entry in the list was inserted by `add`, which
            // received the `UnsafeRef` from `OwnedObject::into_ref` after
            // upcasting a `Box<T: ObjectKind>`. So `raw` originates from
            // `OwnedObject::into_raw` and is the unique owning pointer at drop
            // time — callers of `add` are responsible for not retaining the
            // returned `UnsafeRef` past the pool's lifetime (the standard
            // `UnsafeRef` contract).
            drop(unsafe { OwnedObject::from_raw(raw) });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::{ObjKind, string::StringObj};

    #[test]
    fn add_one_and_drop_pool() {
        let mut pool = ObjectPool::new();
        pool.add(StringObj::new("one"));
    }

    #[test]
    fn add_many_and_drop_pool() {
        let mut pool = ObjectPool::new();
        for i in 0..32 {
            pool.add(StringObj::new(&format!("str-{i}")));
        }
    }

    #[test]
    fn returned_ref_kind_is_string() {
        let mut pool = ObjectPool::new();
        let obj_ref = pool.add(StringObj::new("ref"));
        match &obj_ref.kind {
            ObjKind::String => {}
        }
    }

    #[test]
    fn returned_ref_is_alive_until_pool_drop() {
        // The UnsafeRef returned by `add` should remain valid for the pool's lifetime.
        let mut pool = ObjectPool::new();
        let obj_ref = pool.add(StringObj::new("alive"));
        // SAFETY: `obj_ref` was just produced from a `StringObj`, so its
        // dynamic kind is `StringObj`.
        let s = unsafe { obj_ref.downcast::<StringObj>() };
        assert_eq!(&**s, "alive");
        // Don't drop `s` — pool owns the alloc.
        let _ = UnsafeRef::into_raw(s);
    }
}
