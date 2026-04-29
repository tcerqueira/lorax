use std::{
    alloc::{self, Layout},
    mem,
    ops::Deref,
    ptr,
};

use intrusive_collections::UnsafeRef;

use crate::object::{Object, ObjectCast, OwnedObject};

#[repr(C)]
#[derive(Debug)]
pub struct StringObj {
    obj: Object,
    len: usize,
    buf: [u8],
}

impl StringObj {
    pub fn new(s: &str) -> Box<Self> {
        let bytes = s.as_bytes();
        let layout = Self::layout(bytes.len());

        unsafe {
            let raw = alloc::alloc(layout);
            if raw.is_null() {
                alloc::handle_alloc_error(layout);
            }
            let fat: *mut StringObj = ptr::from_raw_parts_mut(raw.cast::<()>(), bytes.len());

            ptr::write(&raw mut (*fat).obj, Object::string());
            ptr::write(&raw mut (*fat).len, bytes.len());
            ptr::copy_nonoverlapping(
                bytes.as_ptr(),
                (&raw mut (*fat).buf).cast::<u8>(),
                bytes.len(),
            );

            Box::from_raw(fat)
        }
    }

    pub fn as_str(&self) -> &str {
        Self::as_ref(self)
    }

    pub fn as_bytes(&self) -> &[u8] {
        Self::as_ref(self)
    }

    /// # Safety
    ///
    /// `self` must be the unique owning reference to a heap-allocated `StringObj`
    /// originally produced by `Self::new`. After this call, the allocation is freed.
    pub(crate) unsafe fn free(self: UnsafeRef<Self>) {
        let raw = UnsafeRef::into_raw(self);
        drop(unsafe { Box::from_raw(raw) });
    }

    fn layout(len: usize) -> Layout {
        let (l, _) = Layout::new::<Object>()
            .extend(Layout::new::<usize>())
            .unwrap();
        let (l, _) = l.extend(Layout::array::<u8>(len).unwrap()).unwrap();
        l.pad_to_align()
    }
}

impl ObjectCast for StringObj {
    fn upcast(self: Box<Self>) -> OwnedObject {
        let fat: *mut StringObj = Box::into_raw(self);
        unsafe { OwnedObject::from_raw(fat.cast()) }
    }

    unsafe fn downcast(obj: UnsafeRef<Object>) -> UnsafeRef<Self> {
        let thin: *const Object = UnsafeRef::into_raw(obj);
        let len = unsafe { ptr::read(thin.byte_add(mem::offset_of!(StringObj, len)).cast()) };
        let fat: *const StringObj = ptr::from_raw_parts(thin.cast::<()>(), len);
        unsafe { UnsafeRef::from_raw(fat) }
    }
}

impl Deref for StringObj {
    type Target = str;

    fn deref(&self) -> &str {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::OwnedObject;

    #[test]
    fn new_preserves_ascii() {
        let s = StringObj::new("hello");
        assert_eq!(&**s, "hello");
        assert_eq!(s.len, 5);
    }

    #[test]
    fn new_empty_string() {
        let s = StringObj::new("");
        assert_eq!(&**s, "");
        assert_eq!(s.len, 0);
    }

    #[test]
    fn new_preserves_utf8() {
        let input = "héllo 世界 🦀";
        let s = StringObj::new(input);
        assert_eq!(&**s, input);
        assert_eq!(s.len, input.len());
    }

    #[test]
    fn as_ref_bytes_matches() {
        let s = StringObj::new("abc");
        let bytes = s.as_bytes();
        assert_eq!(bytes, b"abc");
    }

    #[test]
    fn box_drops_cleanly() {
        // Smoke test: dropping Box<StringObj> uses fat-pointer Layout::for_value
        // to dealloc the full alloc. Miri verifies no leak/UB.
        let _ = StringObj::new("dropped via Box");
    }

    #[test]
    fn upcast_downcast_roundtrip() {
        let owned: OwnedObject = StringObj::new("roundtrip").upcast();
        let obj_ref = owned.into_ref();
        let downcast = unsafe { StringObj::downcast(obj_ref) };
        assert_eq!(&**downcast, "roundtrip");

        let raw = UnsafeRef::into_raw(downcast);
        drop(unsafe { Box::from_raw(raw) });
    }
}
