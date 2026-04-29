use std::{fmt::Debug, mem, ops::Deref, ptr::NonNull};

use intrusive_collections::{SinglyLinkedListLink, UnsafeRef, intrusive_adapter};

use crate::object::string::StringObj;

pub mod pool;
pub mod string;

pub trait ObjectCast {
    fn upcast(self: Box<Self>) -> OwnedObject;
    /// # Safety
    ///
    /// The object must be the kind of `Self`.
    unsafe fn downcast(obj: UnsafeRef<Object>) -> UnsafeRef<Self>;
}

pub type HeapObject = UnsafeRef<Object>;

#[repr(C)]
#[derive(Debug)]
pub struct Object {
    kind: ObjKind,
    link: SinglyLinkedListLink,
}

intrusive_adapter!(ObjectAdapter = UnsafeRef<Object>: Object { link => SinglyLinkedListLink });

#[derive(Debug)]
enum ObjKind {
    String,
}

impl Object {
    pub fn string() -> Self {
        Self {
            kind: ObjKind::String,
            link: SinglyLinkedListLink::new(),
        }
    }
}

/// Owning thin handle to a heap-allocated object. `Drop` dispatches by `kind`
/// and frees with the correct layout (the alloc is oversized relative to
/// `sizeof(Object)`, so `Box<Object>` cannot do this on its own).
pub struct OwnedObject(NonNull<Object>);

impl OwnedObject {
    /// # Safety
    ///
    /// `ptr` must be a unique owning pointer to a heap object whose actual
    /// layout matches its `kind` (i.e. produced via `ObjectCast::upcast`).
    pub unsafe fn from_raw(ptr: *mut Object) -> Self {
        Self(unsafe { NonNull::new_unchecked(ptr) })
    }

    pub fn into_raw(self) -> *mut Object {
        let ptr = self.0.as_ptr();
        mem::forget(self);
        ptr
    }

    pub fn into_ref(self) -> UnsafeRef<Object> {
        unsafe { UnsafeRef::from_raw(self.into_raw()) }
    }
}

impl Deref for OwnedObject {
    type Target = Object;

    fn deref(&self) -> &Object {
        unsafe { self.0.as_ref() }
    }
}

impl AsRef<Object> for OwnedObject {
    fn as_ref(&self) -> &Object {
        self
    }
}

impl Drop for OwnedObject {
    fn drop(&mut self) {
        let obj_ref = unsafe { UnsafeRef::from_raw(self.0.as_ptr()) };
        match &self.kind {
            ObjKind::String => {
                let s = unsafe { StringObj::downcast(obj_ref) };
                unsafe { s.free() };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drop_frees_string_alloc() {
        // Smoke test: OwnedObject::drop dispatches and frees the full alloc.
        // Miri catches leaks/UB.
        let owned = StringObj::new("dropped via OwnedObject").upcast();
        drop(owned);
    }

    #[test]
    fn deref_exposes_kind() {
        let owned = StringObj::new("k").upcast();
        match &owned.kind {
            ObjKind::String => {}
        }
    }

    #[test]
    fn as_ref_returns_object() {
        let owned = StringObj::new("a").upcast();
        let _: &Object = owned.as_ref();
    }

    #[test]
    fn into_ref_then_owned_roundtrip() {
        // into_ref → reconstruct OwnedObject → drop. No double-free.
        let owned = StringObj::new("via ref").upcast();
        let obj_ref = owned.into_ref();
        let raw = UnsafeRef::into_raw(obj_ref);
        drop(unsafe { OwnedObject::from_raw(raw) });
    }
}
