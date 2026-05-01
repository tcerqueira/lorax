use std::{
    alloc::Layout,
    fmt::Display,
    mem,
    ops::Deref,
    ptr::{self, NonNull},
};

use erasable::{Erasable, ErasedPtr};
use slice_dst::{AllocSliceDst, SliceDst};

use crate::object::{Object, ObjectKind};

#[repr(C)]
#[derive(Debug)]
pub struct StringObj {
    obj: Object,
    len: usize,
    buf: [u8],
}

impl StringObj {
    pub fn boxed(s: &str) -> Box<Self> {
        let bytes = s.as_bytes();
        // SAFETY: `Box::new_slice_dst` allocates with `Self::layout_for(len)`
        // and hands us a fully-uninitialized pointer. The initializer writes
        // every field exactly once via `ptr::write` (skipping `Drop` of
        // uninitialized data) and fills `buf` from a non-overlapping source.
        unsafe {
            <Box<Self> as AllocSliceDst<Self>>::new_slice_dst(bytes.len(), |ptr| {
                let p = ptr.as_ptr();
                ptr::write(&raw mut (*p).obj, Object::string());
                ptr::write(&raw mut (*p).len, bytes.len());
                ptr::copy_nonoverlapping(
                    bytes.as_ptr(),
                    (&raw mut (*p).buf).cast::<u8>(),
                    bytes.len(),
                );
            })
        }
    }

    pub fn as_str(&self) -> &str {
        Self::as_ref(self)
    }

    pub fn as_bytes(&self) -> &[u8] {
        Self::as_ref(self)
    }
}

// SAFETY: `StringObj` is `#[repr(C)]` with `Object` (`obj`) as its first
// field, so an `Object` header at offset 0 is layout-compatible. Construction
// goes through `Self::boxed`, which sets `obj.kind = ObjKind::String`.
unsafe impl ObjectKind for StringObj {}

// SAFETY: `layout_for(len)` produces the exact layout written by `boxed`'s
// initializer, and `retype` is a pure pointer cast as required.
unsafe impl SliceDst for StringObj {
    fn layout_for(len: usize) -> Layout {
        let (l, _) = Layout::new::<Object>()
            .extend(Layout::new::<usize>())
            .unwrap();
        let (l, _) = l.extend(Layout::array::<u8>(len).unwrap()).unwrap();
        l.pad_to_align()
    }

    fn retype(ptr: NonNull<[()]>) -> NonNull<Self> {
        NonNull::from_raw_parts(ptr.cast::<()>(), ptr.len())
    }
}

// SAFETY: the inline `len` field at `offset_of!(StringObj, len)` always equals
// the slice length used to allocate the value (set in `boxed`), so reading it
// rebuilds the correct fat pointer. The read is a raw pointer read, no
// reference is materialized.
unsafe impl Erasable for StringObj {
    unsafe fn unerase(this: ErasedPtr) -> NonNull<Self> {
        let raw = this.as_ptr();
        // SAFETY: `raw` came from `erase` on a `NonNull<StringObj>`; the `len`
        // field lives at a fixed `#[repr(C)]` offset and is initialized.
        let len = unsafe {
            ptr::read(
                raw.byte_add(mem::offset_of!(Self, len))
                    .cast::<usize>()
                    .cast_const(),
            )
        };
        NonNull::from_raw_parts(this.cast::<()>(), len)
    }

    const ACK_1_1_0: bool = true;
}

impl PartialEq for StringObj {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Deref for StringObj {
    type Target = str;

    fn deref(&self) -> &str {
        // SAFETY: `buf` is only ever populated from `&str` bytes in `boxed`, so
        // its contents are guaranteed to be valid UTF-8.
        unsafe { std::str::from_utf8_unchecked(&self.buf) }
    }
}

impl AsRef<str> for StringObj {
    fn as_ref(&self) -> &str {
        self
    }
}

impl AsRef<[u8]> for StringObj {
    fn as_ref(&self) -> &[u8] {
        &self.buf
    }
}

impl Display for StringObj {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_str().fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use intrusive_collections::UnsafeRef;

    use super::*;
    use crate::object::OwnedObject;

    #[test]
    fn new_preserves_ascii() {
        let s = StringObj::boxed("hello");
        assert_eq!(&**s, "hello");
        assert_eq!(s.len, 5);
    }

    #[test]
    fn new_empty_string() {
        let s = StringObj::boxed("");
        assert_eq!(&**s, "");
        assert_eq!(s.len, 0);
    }

    #[test]
    fn new_preserves_utf8() {
        let input = "héllo 世界 🦀";
        let s = StringObj::boxed(input);
        assert_eq!(&**s, input);
        assert_eq!(s.len, input.len());
    }

    #[test]
    fn as_ref_bytes_matches() {
        let s = StringObj::boxed("abc");
        let bytes = s.as_bytes();
        assert_eq!(bytes, b"abc");
    }

    #[test]
    fn box_drops_cleanly() {
        // Smoke test: dropping Box<StringObj> uses fat-pointer Layout::for_value
        // to dealloc the full alloc. Miri verifies no leak/UB.
        let _ = StringObj::boxed("dropped via Box");
    }

    #[test]
    fn upcast_downcast_roundtrip() {
        let owned: OwnedObject = StringObj::boxed("roundtrip").upcast();
        let obj_ref = owned.into_ref();
        // SAFETY: `obj_ref` was just produced from a `StringObj`, so its
        // dynamic kind is `StringObj`.
        let downcast = unsafe { obj_ref.downcast::<StringObj>() };
        assert_eq!(&**downcast, "roundtrip");

        let raw = UnsafeRef::into_raw(downcast);
        // SAFETY: `raw` traces back to `Box::into_raw(StringObj::boxed(...))`
        // via the upcast/downcast roundtrip, so `Box::from_raw` is the matching
        // ownership transfer.
        drop(unsafe { Box::from_raw(raw) });
    }
}
