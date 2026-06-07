use intrusive_collections::UnsafeRef;

use crate::{
    chunk::Chunk,
    object::{Object, closure::LoxClosure},
};

/// What a [`CallFrame`] executes. The script top level runs as a bare owned
/// `Chunk` (frame 0); a call runs over the chunk inside its closure, reached
/// through the same `UnsafeRef` handle the stack and globals hold — never a
/// tracked `&Chunk`, since the callee's body mutates the very heap that owns its
/// chunk.
pub enum FrameSource {
    TopLevel(Chunk),
    Closure(UnsafeRef<Object>),
}

impl FrameSource {
    pub fn chunk(&self) -> &Chunk {
        match self {
            FrameSource::TopLevel(chunk) => chunk,
            // SAFETY: a `Closure` frame always holds a `LoxClosure` handle; the
            // chunk is immutable for the frame's lifetime and the object stays
            // alive because the frame roots the handle.
            FrameSource::Closure(obj) => unsafe { obj.downcast_ref::<LoxClosure>() }.chunk(),
        }
    }

    pub fn code(&self) -> &[u8] {
        &self.chunk().code
    }
}

/// One activation record: where to read code (`source`), the byte offset of the
/// next instruction (`ip`), and the stack index of this frame's slot 0 (`base`).
pub struct CallFrame {
    pub source: FrameSource,
    pub ip: usize,
    pub base: usize,
}
