use std::{
    cell::Cell,
    fmt::{self, Debug, Display, Formatter},
    mem,
    ops::Deref,
    ptr::{self, NonNull},
};

use erasable::{Erasable, ErasedPtr, erase};
use intrusive_collections::{SinglyLinkedListLink, UnsafeRef, intrusive_adapter};

use crate::{
    object::{
        bound_method::LoxBoundMethod, class::LoxClass, closure::LoxClosure, function::LoxFunction,
        instance::LoxInstance, native::LoxNative, string::LoxString, upvalue::LoxUpvalue,
    },
    storage::WithStorage,
};

pub mod bound_method;
pub mod class;
pub mod closure;
pub mod function;
pub mod instance;
pub mod native;
pub mod string;
pub mod upvalue;

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
    /// GC mark bit. `Cell` because the collector flips it through the shared
    /// `&Object` that the heap, stack, and globals all alias. It sits in the
    /// padding between `kind` and `link`, so `Object` stays 16 bytes and the
    /// hand-rolled `LoxString` DST layout is unchanged (asserted in tests).
    mark: Cell<bool>,
    link: SinglyLinkedListLink,
}

intrusive_adapter!(pub ObjectAdapter = UnsafeRef<Object>: Object { link => SinglyLinkedListLink });

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjKind {
    String,
    Function,
    Native,
    Closure,
    Upvalue,
    Class,
    Instance,
    BoundMethod,
}

impl Object {
    fn of(kind: ObjKind) -> Self {
        Self {
            kind,
            mark: Cell::new(false),
            link: SinglyLinkedListLink::new(),
        }
    }

    pub fn string() -> Self {
        Self::of(ObjKind::String)
    }

    pub fn function() -> Self {
        Self::of(ObjKind::Function)
    }

    pub fn native() -> Self {
        Self::of(ObjKind::Native)
    }

    pub fn closure() -> Self {
        Self::of(ObjKind::Closure)
    }

    pub fn upvalue() -> Self {
        Self::of(ObjKind::Upvalue)
    }

    pub fn class() -> Self {
        Self::of(ObjKind::Class)
    }

    pub fn instance() -> Self {
        Self::of(ObjKind::Instance)
    }

    pub fn bound_method() -> Self {
        Self::of(ObjKind::BoundMethod)
    }

    /// GC mark bit. The collector marks reachable objects, then sweeps (and
    /// clears) the unmarked rest.
    pub fn is_marked(&self) -> bool {
        self.mark.get()
    }

    pub fn set_marked(&self, marked: bool) {
        self.mark.set(marked);
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
            // Heap objects with reference identity (everything but strings).
            (ObjKind::Function, ObjKind::Function)
            | (ObjKind::Native, ObjKind::Native)
            | (ObjKind::Closure, ObjKind::Closure)
            | (ObjKind::Upvalue, ObjKind::Upvalue)
            | (ObjKind::Class, ObjKind::Class)
            | (ObjKind::Instance, ObjKind::Instance)
            | (ObjKind::BoundMethod, ObjKind::BoundMethod) => {
                ptr::eq(self.as_ref(), other.as_ref())
            }
            _ => false,
        }
    }

    pub fn display_fmt(self: &UnsafeRef<Self>, f: &mut Formatter<'_>) -> fmt::Result {
        match self.kind() {
            // SAFETY: matched kind witnesses the dynamic type.
            ObjKind::String => Display::fmt(unsafe { self.downcast_ref::<LoxString>() }, f),
            ObjKind::Function => Display::fmt(unsafe { self.downcast_ref::<LoxFunction>() }, f),
            ObjKind::Native => Display::fmt(unsafe { self.downcast_ref::<LoxNative>() }, f),
            ObjKind::Closure => Display::fmt(unsafe { self.downcast_ref::<LoxClosure>() }, f),
            ObjKind::Class => Display::fmt(unsafe { self.downcast_ref::<LoxClass>() }, f),
            ObjKind::Instance => Display::fmt(unsafe { self.downcast_ref::<LoxInstance>() }, f),
            ObjKind::BoundMethod => {
                Display::fmt(unsafe { self.downcast_ref::<LoxBoundMethod>() }, f)
            }
            // Upvalues never surface to user code; this is for debug only.
            ObjKind::Upvalue => write!(f, "<upvalue>"),
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
            ObjKind::Native => {
                // SAFETY: matched kind witnesses the dynamic type.
                WithStorage(unsafe { self.0.downcast_ref::<LoxNative>() }, self.1).fmt(f)
            }
            ObjKind::Closure => {
                // SAFETY: matched kind witnesses the dynamic type.
                WithStorage(unsafe { self.0.downcast_ref::<LoxClosure>() }, self.1).fmt(f)
            }
            ObjKind::Class => {
                // SAFETY: matched kind witnesses the dynamic type.
                WithStorage(unsafe { self.0.downcast_ref::<LoxClass>() }, self.1).fmt(f)
            }
            ObjKind::Instance => {
                // SAFETY: matched kind witnesses the dynamic type.
                WithStorage(unsafe { self.0.downcast_ref::<LoxInstance>() }, self.1).fmt(f)
            }
            ObjKind::BoundMethod => {
                // SAFETY: matched kind witnesses the dynamic type.
                WithStorage(unsafe { self.0.downcast_ref::<LoxBoundMethod>() }, self.1).fmt(f)
            }
            ObjKind::String | ObjKind::Upvalue => self.0.display_fmt(f),
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
            ObjKind::Native => drop(unsafe { Box::from_raw(LoxNative::unerase(erased).as_ptr()) }),
            ObjKind::Closure => {
                drop(unsafe { Box::from_raw(LoxClosure::unerase(erased).as_ptr()) })
            }
            ObjKind::Upvalue => {
                drop(unsafe { Box::from_raw(LoxUpvalue::unerase(erased).as_ptr()) })
            }
            ObjKind::Class => drop(unsafe { Box::from_raw(LoxClass::unerase(erased).as_ptr()) }),
            ObjKind::Instance => {
                drop(unsafe { Box::from_raw(LoxInstance::unerase(erased).as_ptr()) })
            }
            ObjKind::BoundMethod => {
                drop(unsafe { Box::from_raw(LoxBoundMethod::unerase(erased).as_ptr()) })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Storage;

    #[test]
    fn header_layout_absorbs_mark_bit() {
        // The GC mark bit must sit in existing padding so `Object` stays 16
        // bytes and the hand-rolled `LoxString` DST layout is unaffected.
        assert_eq!(mem::size_of::<Object>(), 16);
        assert_eq!(mem::align_of::<Object>(), 8);
    }

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
