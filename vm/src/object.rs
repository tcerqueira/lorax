use std::{
    fmt::{self, Debug, Display, Formatter},
    mem,
    ops::Deref,
    ptr::NonNull,
};

use erasable::{Erasable, ErasedPtr, erase};
use intrusive_collections::{SinglyLinkedListLink, UnsafeRef, intrusive_adapter};

use crate::object::string::StringObj;

pub mod pool;
pub mod string;

/// A concrete object kind that can be stored behind an [`Object`] header.
///
/// # Safety
///
/// Implementor must be `#[repr(C)]` with [`Object`] as the first field, no
/// padding before it, and its embedded `Object`'s `kind` must accurately
/// describe the dynamic layout (set during construction). Implementor must
/// also implement [`Erasable`] so a thin pointer can be reconstituted.
pub unsafe trait ObjectKind: Erasable {
    /// Take ownership of a `Box<Self>` as an [`OwnedObject`].
    fn upcast(self: Box<Self>) -> OwnedObject {
        let raw = Box::into_raw(self);
        // SAFETY: `Box::into_raw` yields a unique owning pointer. The trait's
        // `#[repr(C)]` requirement guarantees the cast to `*mut Object` is
        // valid, and the embedded `Object`'s `kind` was set by `Self`'s
        // constructor, so it accurately describes the layout — meeting
        // `OwnedObject::from_raw`'s contract.
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjKind {
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
        // SAFETY: caller upholds the kind invariant, so `T::unerase` returns a
        // valid `NonNull<T>` to the same allocation. We only produce a shared
        // `&T` whose lifetime is tied to `&self`.
        unsafe { T::unerase(erase(NonNull::from(self))).as_ref() }
    }

    /// Downcast a unique reference to a concrete kind.
    ///
    /// # Safety
    ///
    /// The Object's dynamic kind must be `T`.
    pub unsafe fn downcast_mut<T: ObjectKind + ?Sized>(&mut self) -> &mut T {
        // SAFETY: caller upholds the kind invariant, so `T::unerase` returns a
        // valid `NonNull<T>`. The `&mut self` borrow guarantees the resulting
        // `&mut T` is unique and valid for `&mut self`'s lifetime.
        unsafe { T::unerase(erase(NonNull::from(self))).as_mut() }
    }

    /// Downcast an unsafe reference to a concrete kind.
    ///
    /// # Safety
    ///
    /// The Object's dynamic kind must be `T`.
    pub unsafe fn downcast<T: ObjectKind + ?Sized>(self: UnsafeRef<Self>) -> UnsafeRef<T> {
        let raw = UnsafeRef::into_raw(self);
        // SAFETY: `UnsafeRef::into_raw` returned a valid pointer; caller
        // upholds the kind invariant, so `T::unerase` is a valid `NonNull<T>`
        // to the same allocation. We're transferring ownership of the raw
        // pointer to the new `UnsafeRef<T>`.
        unsafe {
            let erased = erase(NonNull::new_unchecked(raw));
            UnsafeRef::from_raw(T::unerase(erased).as_ptr())
        }
    }

    pub fn kind(&self) -> ObjKind {
        self.kind
    }
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        if self.kind != other.kind {
            return false;
        }
        match self.kind {
            ObjKind::String => unsafe {
                self.downcast_ref::<StringObj>() == other.downcast_ref::<StringObj>()
            },
        }
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.kind {
            // SAFETY: `kind == ObjKind::String` is the type-system witness that
            // the dynamic kind is `StringObj` — set at construction by
            // `Object::string()` inside `StringObj::boxed`.
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
    /// layout matches its `kind` (i.e. produced via `ObjectKind::upcast`).
    pub unsafe fn from_raw(ptr: *mut Object) -> Self {
        // SAFETY: caller's contract requires `ptr` to be a valid (non-null)
        // owning pointer.
        Self(unsafe { NonNull::new_unchecked(ptr) })
    }

    pub fn into_raw(self) -> *mut Object {
        let ptr = self.0.as_ptr();
        mem::forget(self);
        ptr
    }

    pub fn into_ref(self) -> UnsafeRef<Object> {
        // SAFETY: `into_raw` yields a non-null pointer to a valid `Object`,
        // and ownership is transferred from this `OwnedObject` to the new
        // `UnsafeRef`.
        unsafe { UnsafeRef::from_raw(self.into_raw()) }
    }
}

impl Deref for OwnedObject {
    type Target = Object;

    fn deref(&self) -> &Object {
        // SAFETY: `OwnedObject`'s invariant guarantees `self.0` points to a
        // live `Object` for `&self`'s lifetime.
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
        let erased: ErasedPtr = erase(self.0);
        match self.kind {
            // SAFETY: `OwnedObject`'s invariant guarantees `erased` is the
            // unique owning thin pointer to a heap object whose layout matches
            // `kind`. Matching `ObjKind::String` confirms the dynamic kind is
            // `StringObj`, so `unerase` reconstructs the correct fat pointer.
            // The original allocation came from `Box::new_slice_dst` in
            // `StringObj::boxed`, so re-boxing here uses the matching dealloc
            // path.
            ObjKind::String => drop(unsafe {
                Box::from_raw(StringObj::unerase(erased).as_ptr())
            }),
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
        let owned = StringObj::boxed("dropped via OwnedObject").upcast();
        drop(owned);
    }

    #[test]
    fn deref_exposes_kind() {
        let owned = StringObj::boxed("k").upcast();
        match &owned.kind {
            ObjKind::String => {}
        }
    }

    #[test]
    fn as_ref_returns_object() {
        let owned = StringObj::boxed("a").upcast();
        let _: &Object = owned.as_ref();
    }

    #[test]
    fn into_ref_then_owned_roundtrip() {
        // into_ref → reconstruct OwnedObject → drop. No double-free.
        let owned = StringObj::boxed("via ref").upcast();
        let obj_ref = owned.into_ref();
        let raw = UnsafeRef::into_raw(obj_ref);
        // SAFETY: `raw` originated from `OwnedObject::into_raw` (via
        // `into_ref` → `UnsafeRef::into_raw`), so it's still the unique
        // owning pointer.
        drop(unsafe { OwnedObject::from_raw(raw) });
    }
}
