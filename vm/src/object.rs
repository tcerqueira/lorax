use std::{
    fmt::{self, Debug, Display, Formatter},
    mem,
    ops::Deref,
    ptr::{self, NonNull},
};

use erasable::{Erasable, ErasedPtr, erase};
use intrusive_collections::{SinglyLinkedListLink, UnsafeRef, intrusive_adapter};

use crate::{
    object::{function::LoxFunction, string::LoxString},
    storage::WithStorage,
};

pub mod function;
pub mod string;

/// A concrete object kind that can be stored behind an [`Object`] header.
///
/// # Safety
///
/// Implementor must be `#[repr(C)]` with [`Object`] as the first field, no
/// padding before it, and its embedded `Object`'s `kind` must accurately
/// describe the dynamic layout (set during construction). Implementor must
/// also implement [`Erasable`] so a thin pointer can be reconstituted.
pub unsafe trait ObjectType: Erasable {
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

intrusive_adapter!(pub ObjectAdapter = UnsafeRef<Object>: Object { link => SinglyLinkedListLink });

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjKind {
    String,
    Function,
}

impl Object {
    pub fn string() -> Self {
        Self {
            kind: ObjKind::String,
            link: SinglyLinkedListLink::new(),
        }
    }

    pub fn function() -> Self {
        Self {
            kind: ObjKind::Function,
            link: SinglyLinkedListLink::new(),
        }
    }

    /// Downcast a shared reference to a concrete kind.
    ///
    /// # Safety
    ///
    /// The Object's dynamic kind must be `T`.
    pub unsafe fn downcast_ref<T: ObjectType + ?Sized>(self: &UnsafeRef<Self>) -> &T {
        // SAFETY: caller upholds the kind invariant. `UnsafeRef::as_ptr` yields
        // the original heap pointer with its full provenance (set at the alloc
        // site), so `unerase`'s NonNull is a valid `NonNull<T>` to the heap object.
        let ptr = unsafe { NonNull::new_unchecked(UnsafeRef::into_raw(self.clone())) };
        unsafe { T::unerase(erase(ptr)).as_ref() }
    }

    /// Downcast a unique reference to a concrete kind.
    ///
    /// # Safety
    ///
    /// - The Object's dynamic kind must be `T`.
    /// - No other `UnsafeRef<Object>` (or any other path) may alias the pointee
    ///   for the duration of the returned borrow.
    pub unsafe fn downcast_mut<T: ObjectType + ?Sized>(self: &mut UnsafeRef<Self>) -> &mut T {
        // SAFETY: caller upholds the kind invariant, so `T::unerase` returns a
        // valid `NonNull<T>`. The `&mut self` borrow guarantees the resulting
        // `&mut T` is unique and valid for `&mut self`'s lifetime.
        let ptr = unsafe { NonNull::new_unchecked(UnsafeRef::into_raw(self.clone())) };
        unsafe { T::unerase(erase(ptr)).as_mut() }
    }

    /// Downcast an unsafe reference to a concrete kind.
    ///
    /// # Safety
    ///
    /// The Object's dynamic kind must be `T`.
    pub unsafe fn downcast<T: ObjectType + ?Sized>(self: UnsafeRef<Self>) -> UnsafeRef<T> {
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

    pub fn as_str(self: &UnsafeRef<Self>) -> &str {
        // SAFETY: matched kind witnesses the dynamic type on each side.
        match self.kind() {
            ObjKind::String => unsafe { self.downcast_ref::<LoxString>().as_str() },
            o => panic!("Object::as_str called on non-string {o:?}"),
        }
    }
}

impl Object {
    pub fn eq(self: &UnsafeRef<Self>, other: &UnsafeRef<Self>) -> bool {
        match (self.kind(), other.kind()) {
            // SAFETY: matched kind witnesses the dynamic type on each side.
            (ObjKind::String, ObjKind::String) => unsafe {
                self.downcast_ref::<LoxString>() == other.downcast_ref::<LoxString>()
            },
            (ObjKind::Function, ObjKind::Function) => ptr::eq(self.as_ref(), other.as_ref()),
            _ => false,
        }
    }

    pub fn display_fmt(self: &UnsafeRef<Self>, f: &mut Formatter<'_>) -> fmt::Result {
        match self.kind() {
            // SAFETY: matched kind witnesses the dynamic type.
            ObjKind::String => Display::fmt(unsafe { self.downcast_ref::<LoxString>() }, f),
            ObjKind::Function => Display::fmt(unsafe { self.downcast_ref::<LoxFunction>() }, f),
        }
    }
}

impl Display for WithStorage<'_, UnsafeRef<Object>> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.0.kind() {
            ObjKind::Function => {
                // SAFETY: matched kind witnesses the dynamic type.
                WithStorage(unsafe { self.0.downcast_ref::<LoxFunction>() }, self.1).fmt(f)
            }
            ObjKind::String => self.0.display_fmt(f),
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
            // `LoxString`, so `unerase` reconstructs the correct fat pointer.
            // The original allocation came from `Box::new_slice_dst` in
            // `LoxString::boxed`, so re-boxing here uses the matching dealloc
            // path.
            ObjKind::String => drop(unsafe { Box::from_raw(LoxString::unerase(erased).as_ptr()) }),
            ObjKind::Function => {
                drop(unsafe { Box::from_raw(LoxFunction::unerase(erased).as_ptr()) })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Storage;

    #[test]
    fn drop_frees_string_alloc() {
        // Smoke test: OwnedObject::drop dispatches and frees the full alloc.
        // Miri catches leaks/UB.
        let owned = LoxString::boxed("dropped via OwnedObject").upcast();
        drop(owned);
    }

    #[test]
    fn deref_exposes_kind() {
        let owned = LoxString::boxed("k").upcast();
        assert_eq!(owned.kind(), ObjKind::String);
    }

    #[test]
    fn as_ref_returns_object() {
        let owned = LoxString::boxed("a").upcast();
        let obj: &Object = owned.as_ref();
        assert_eq!(obj.kind(), ObjKind::String);
    }

    #[test]
    fn as_str_returns_buffer() {
        let mut storage = Storage::new();
        let obj_ref = storage.add_obj(LoxString::boxed("plain"));
        assert_eq!(obj_ref.as_str(), "plain");
    }
}
