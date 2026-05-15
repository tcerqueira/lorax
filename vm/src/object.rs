use std::{
    fmt::{self, Debug, Formatter},
    mem,
    ops::Deref,
    ptr::NonNull,
};

use erasable::{Erasable, ErasedPtr, erase};
use intrusive_collections::{SinglyLinkedListLink, UnsafeRef, intrusive_adapter};

use crate::{
    object::{internal_str::InternalStr, string::StringObj},
    storage::Storage,
};

pub mod internal_str;
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
    InternalStr,
}

impl Object {
    pub fn string() -> Self {
        Self {
            kind: ObjKind::String,
            link: SinglyLinkedListLink::new(),
        }
    }

    pub fn internal_str() -> Self {
        Self {
            kind: ObjKind::InternalStr,
            link: SinglyLinkedListLink::new(),
        }
    }

    /// Downcast a shared reference to a concrete kind.
    ///
    /// # Safety
    ///
    /// The Object's dynamic kind must be `T`.
    pub unsafe fn downcast_ref<T: ObjectKind + ?Sized>(self: &UnsafeRef<Self>) -> &T {
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
    /// The Object's dynamic kind must be `T`.
    pub unsafe fn downcast_mut<T: ObjectKind + ?Sized>(self: &mut UnsafeRef<Self>) -> &mut T {
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

    pub fn is_str(&self) -> bool {
        self.kind == ObjKind::String || self.kind == ObjKind::InternalStr
    }

    pub fn as_str<'s>(self: &UnsafeRef<Self>, storage: &'s Storage) -> &'s str {
        match self.kind() {
            ObjKind::String => unsafe {
                // SAFETY: the alloc is owned by `storage.heap` (an ObjectPool),
                // which is borrowed shared via `&'s Storage`, so it cannot be
                // dropped or mutated for the duration of `'s`. The buffer sits
                // inline in that alloc, so the `&str` is valid for `'s`.
                let s: &str = self.downcast_ref::<StringObj>().as_str();
                mem::transmute::<&str, &'s str>(s)
            },
            ObjKind::InternalStr => unsafe {
                self.downcast_ref::<InternalStr>().as_str(&storage.strings)
            },
        }
    }
}

impl Object {
    pub fn eq(self: &UnsafeRef<Self>, other: &UnsafeRef<Self>) -> bool {
        match (self.kind(), other.kind()) {
            // SAFETY: matched kind witnesses the dynamic type on each side.
            (ObjKind::String, ObjKind::String) => unsafe {
                self.downcast_ref::<StringObj>() == other.downcast_ref::<StringObj>()
            },
            (ObjKind::InternalStr, ObjKind::InternalStr) => unsafe {
                self.downcast_ref::<InternalStr>() == other.downcast_ref::<InternalStr>()
            },
            _ => panic!(
                "cant compare {:?} and {:?} with just the object",
                self.kind(),
                other.kind()
            ),
        }
    }

    pub fn display_fmt(self: &UnsafeRef<Self>, f: &mut Formatter<'_>) -> fmt::Result {
        match self.kind() {
            // SAFETY: matched kind witnesses the dynamic type.
            ObjKind::String => {
                <StringObj as fmt::Display>::fmt(unsafe { self.downcast_ref::<StringObj>() }, f)
            }
            ObjKind::InternalStr => {
                <InternalStr as fmt::Display>::fmt(unsafe { self.downcast_ref::<InternalStr>() }, f)
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
            ObjKind::String => drop(unsafe { Box::from_raw(StringObj::unerase(erased).as_ptr()) }),
            ObjKind::InternalStr => {
                drop(unsafe { Box::from_raw(InternalStr::unerase(erased).as_ptr()) })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use lasso::Rodeo;

    use super::*;

    fn boxed_internal_str(s: &str) -> OwnedObject {
        let mut rodeo = Rodeo::new();
        Box::new(InternalStr::new(&mut rodeo, s)).upcast()
    }

    #[test]
    fn drop_frees_string_alloc() {
        // Smoke test: OwnedObject::drop dispatches and frees the full alloc.
        // Miri catches leaks/UB.
        let owned = StringObj::boxed("dropped via OwnedObject").upcast();
        drop(owned);
    }

    #[test]
    fn drop_frees_internal_str_alloc() {
        // Smoke test for the `InternalStr` Drop branch.
        drop(boxed_internal_str("drop me"));
    }

    #[test]
    fn deref_exposes_kind() {
        let owned = StringObj::boxed("k").upcast();
        assert_eq!(owned.kind(), ObjKind::String);
    }

    #[test]
    fn as_ref_returns_object() {
        let owned = StringObj::boxed("a").upcast();
        let obj: &Object = owned.as_ref();
        assert!(obj.is_str());
    }

    #[test]
    fn as_str_for_internal_str_kind() {
        let mut storage = Storage::new();
        let obj_ref = storage.add_internal_str("interned");
        assert_eq!(obj_ref.as_str(&storage), "interned");
    }

    #[test]
    fn as_str_for_string_kind() {
        // Exercises the `StringObj` arm (including the `&str` lifetime
        // launder). Pool owns the alloc; storage Drop reclaims it.
        let mut storage = Storage::new();
        let obj_ref = storage.add_obj(StringObj::boxed("plain"));
        assert_eq!(obj_ref.as_str(&storage), "plain");
    }

    #[test]
    fn equal_internal_strings_compare_equal_via_object() {
        let mut storage = Storage::new();
        let a = storage.add_internal_str("eq");
        let b = storage.add_internal_str("eq");
        assert!(a.eq(&b));
    }

    #[test]
    fn distinct_internal_strings_not_equal_via_object() {
        let mut storage = Storage::new();
        let a = storage.add_internal_str("a");
        let b = storage.add_internal_str("b");
        assert!(!a.eq(&b));
    }

    #[test]
    #[should_panic(expected = "cant compare")]
    fn cross_kind_string_eq_panics() {
        // `Object::eq` deliberately panics for mixed string kinds — callers
        // must route through `as_str` (the VM does this in `equal`).
        let mut storage = Storage::new();
        let s = storage.add_obj(StringObj::boxed("x"));
        let i = storage.add_internal_str("x");
        let _ = s.eq(&i);
    }
}
