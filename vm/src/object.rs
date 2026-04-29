use std::{
    fmt::{self, Debug, Display, Formatter},
    mem,
    ops::Deref,
    ptr::NonNull,
};

use intrusive_collections::{SinglyLinkedListLink, UnsafeRef, intrusive_adapter};

use crate::object::string::StringObj;

pub mod pool;
pub mod string;

/// A concrete object kind embedded behind an [`Object`] header.
///
/// # Safety
///
/// Implementor must be `#[repr(C)]` with [`Object`] as the first field,
/// no padding before it.
pub unsafe trait ObjectKind {
    /// Recover `*mut Self` from `*mut Object`.
    ///
    /// # Safety
    ///
    /// `obj` must point to an [`Object`] of dynamic kind `Self`.
    unsafe fn from_object_raw(obj: *mut Object) -> *mut Self;

    /// Take ownership of a `Box<Self>` as an [`OwnedObject`].
    fn upcast(self: Box<Self>) -> OwnedObject {
        let raw = Box::into_raw(self);
        unsafe { OwnedObject::from_raw(raw.cast::<Object>()) }
    }
}

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

    /// Downcast a shared reference to a concrete kind.
    ///
    /// # Safety
    ///
    /// The Object's dynamic kind must be `T`.
    pub unsafe fn downcast_ref<T: ObjectKind + ?Sized>(&self) -> &T {
        unsafe { &*T::from_object_raw(self as *const Self as *mut Self) }
    }

    /// Downcast a unique reference to a concrete kind.
    ///
    /// # Safety
    ///
    /// The Object's dynamic kind must be `T`.
    pub unsafe fn downcast_mut<T: ObjectKind + ?Sized>(&mut self) -> &mut T {
        unsafe { &mut *T::from_object_raw(self) }
    }

    /// Downcast a unsafe reference to a concrete kind.
    ///
    /// # Safety
    ///
    /// The Object's dynamic kind must be `T`.
    pub unsafe fn downcast<T: ObjectKind + ?Sized>(self: UnsafeRef<Self>) -> UnsafeRef<T> {
        let raw = UnsafeRef::into_raw(self);
        unsafe { UnsafeRef::from_raw(T::from_object_raw(raw)) }
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.kind {
            ObjKind::String => {
                <StringObj as Display>::fmt(unsafe { self.downcast_ref::<StringObj>() }, f)
            }
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

// impl ObjectPtr for OwnedObject {
//     type Of<T: ObjectKind + ?Sized> = Box<T>;

//     fn into_raw(self) -> *mut Object {
//         OwnedObject::into_raw(self)
//     }

//     unsafe fn from_concrete<T: ObjectKind + ?Sized>(ptr: *mut T) -> Box<T> {
//         unsafe { Box::from_raw(ptr) }
//     }
// }

impl Drop for OwnedObject {
    fn drop(&mut self) {
        let raw = self.0.as_ptr();
        match &self.kind {
            ObjKind::String => drop(unsafe { Box::from_raw(StringObj::from_object_raw(raw)) }),
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
