use std::alloc::Layout;
use std::fmt::{self, Display, Formatter};
use std::mem;
use std::ptr::{self, NonNull};

use erasable::{Erasable, ErasedPtr};
use intrusive_collections::UnsafeRef;
use slice_dst::{AllocSliceDst, SliceDst};

use crate::{
    chunk::Chunk,
    object::{Object, ObjectType, function::LoxFunction},
    storage::WithStorage,
};

/// A function paired with the upvalues it captured. Every callable on the stack
/// is a closure (even those that capture nothing), so the call path has one
/// shape.
///
/// The upvalue array is a `#[repr(C)]` DST tail (like [`LoxString`]'s byte
/// buffer): captured upvalues live *inline*, so a closure is a single allocation
/// (no separate boxed slice) and `OP_GET_UPVALUE` reads a slot without a second
/// pointer hop. The array is write-once — filled at `OP_CLOSURE` and never
/// mutated — so only the `LoxUpvalue`s it points at need interior mutability.
///
/// [`LoxString`]: crate::object::string::LoxString
#[repr(C)]
#[derive(Debug)]
pub struct LoxClosure {
    obj: Object,
    function: UnsafeRef<Object>,
    len: usize,
    upvalues: [UnsafeRef<Object>],
}

// SAFETY: `#[repr(C)]` with `Object` first; `Self::boxed` sets `obj.kind = Closure`.
unsafe impl ObjectType for LoxClosure {}

impl LoxClosure {
    /// Allocate a closure with its `upvalues` stored inline. The function handle
    /// and each upvalue handle are copied into the tail.
    pub fn boxed(function: UnsafeRef<Object>, upvalues: &[UnsafeRef<Object>]) -> Box<Self> {
        let len = upvalues.len();
        // The inline upvalue count is the authority for both upvalue reads and GC
        // tracing, so it must match the count the wrapped function declared (which
        // sized the `OP_CLOSURE` tail and the upvalue-read slots).
        // SAFETY: a closure always wraps a `LoxFunction`.
        debug_assert_eq!(
            len,
            unsafe { function.downcast_ref::<LoxFunction>() }.upvalue_count() as usize,
            "closure upvalue count must match the wrapped function's declared count",
        );
        // SAFETY: `Box::new_slice_dst` allocates `Self::layout_for(len)` and hands
        // us a fully-uninitialized pointer. Every field is written exactly once
        // via `ptr::write` (no `Drop` of uninitialized data), and the tail is
        // filled for its whole length from a non-overlapping source.
        unsafe {
            <Box<Self> as AllocSliceDst<Self>>::new_slice_dst(len, |ptr| {
                let p = ptr.as_ptr();
                ptr::write(&raw mut (*p).obj, Object::closure());
                ptr::write(&raw mut (*p).function, function);
                ptr::write(&raw mut (*p).len, len);
                let tail = (&raw mut (*p).upvalues).cast::<UnsafeRef<Object>>();
                for (i, upvalue) in upvalues.iter().enumerate() {
                    ptr::write(tail.add(i), upvalue.clone());
                }
            })
        }
    }

    fn function(&self) -> &LoxFunction {
        // SAFETY: a closure always wraps a LoxFunction.
        unsafe { self.function.downcast_ref::<LoxFunction>() }
    }

    pub fn chunk(&self) -> &Chunk {
        self.function().chunk()
    }

    pub fn arity(&self) -> u8 {
        self.function().arity()
    }

    pub fn upvalue(&self, index: u8) -> &UnsafeRef<Object> {
        &self.upvalues[index as usize]
    }

    // GC trace edges.
    pub fn function_handle(&self) -> &UnsafeRef<Object> {
        &self.function
    }

    pub fn upvalues(&self) -> &[UnsafeRef<Object>] {
        &self.upvalues
    }
}

// SAFETY: `layout_for(len)` produces the exact layout written by `boxed`'s
// initializer, and `retype` is a pure pointer cast as required.
unsafe impl SliceDst for LoxClosure {
    fn layout_for(len: usize) -> Layout {
        let (l, _) = Layout::new::<Object>()
            .extend(Layout::new::<UnsafeRef<Object>>())
            .unwrap();
        let (l, _) = l.extend(Layout::new::<usize>()).unwrap();
        let (l, _) = l
            .extend(Layout::array::<UnsafeRef<Object>>(len).unwrap())
            .unwrap();
        l.pad_to_align()
    }

    fn retype(ptr: NonNull<[()]>) -> NonNull<Self> {
        NonNull::from_raw_parts(ptr.cast::<()>(), ptr.len())
    }
}

// SAFETY: the inline `len` field at `offset_of!(LoxClosure, len)` always equals
// the upvalue-slice length used to allocate the value (set in `boxed`), so
// reading it rebuilds the correct fat pointer. The read is a raw pointer read; no
// reference is materialized.
unsafe impl Erasable for LoxClosure {
    unsafe fn unerase(this: ErasedPtr) -> NonNull<Self> {
        let raw = this.as_ptr();
        // SAFETY: `raw` came from `erase` on a `NonNull<LoxClosure>`; the `len`
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

impl Display for LoxClosure {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(self.function(), f)
    }
}

impl Display for WithStorage<'_, LoxClosure> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // A closure prints exactly like the function it wraps.
        WithStorage(self.0.function(), self.1).fmt(f)
    }
}
