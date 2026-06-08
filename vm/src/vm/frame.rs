use std::ptr;

use intrusive_collections::UnsafeRef;

use crate::{
    chunk::Chunk,
    object::{Object, closure::LoxClosure},
};

/// What a [`CallFrame`] executes. The script top level runs over a boxed owned
/// `Chunk` (frame 0); a call runs over the chunk inside its closure, reached
/// through the same `UnsafeRef` handle the stack and globals hold — never a
/// tracked `&Chunk`, since the callee's body mutates the very heap that owns its
/// chunk. The top-level chunk is boxed (not inline) so its address is heap-stable
/// regardless of the frames `Vec`, which lets [`CallFrame`] cache a pointer to it.
pub enum FrameSource {
    TopLevel(Box<Chunk>),
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
    /// Cached pointer to `source.chunk()`, populated by [`cache_chunk`] once the
    /// frame sits in its final `Vec` slot. The hot dispatch loop fetches code and
    /// constants through this instead of re-walking closure→function→chunk (two
    /// `UnsafeRef` downcasts) on every instruction.
    ///
    /// Sound for the frame's lifetime: the chunk is immutable, the GC never moves
    /// objects (mark-sweep, no compaction) and the frame roots the owning object,
    /// and the `TopLevel` chunk is boxed so it is heap-stable independent of the
    /// frames `Vec`.
    ///
    /// [`cache_chunk`]: CallFrame::cache_chunk
    chunk: *const Chunk,
}

impl CallFrame {
    /// Build a frame with an unset chunk cache. The caller must invoke
    /// [`cache_chunk`](Self::cache_chunk) after pushing it into the frames `Vec`.
    pub fn new(source: FrameSource, base: usize) -> Self {
        Self {
            source,
            ip: 0,
            base,
            chunk: ptr::null(),
        }
    }

    /// Populate the cached chunk pointer from `source`. The pointer targets the
    /// chunk *inside the heap object* — a `Closure`'s `LoxFunction`, or the boxed
    /// `TopLevel` chunk — which is heap-stable, so this only needs `source` to be
    /// initialized; moving the `CallFrame` within the frames `Vec` afterwards does
    /// not invalidate it. Must run before the frame executes.
    pub fn cache_chunk(&mut self) {
        let ptr = self.source.chunk() as *const Chunk;
        self.chunk = ptr;
    }

    /// The frame's chunk, via the cached pointer (no downcast).
    #[inline]
    pub fn chunk(&self) -> &Chunk {
        debug_assert!(!self.chunk.is_null(), "cache_chunk not called before use");
        // SAFETY: `chunk` was set by `cache_chunk` to `source.chunk()`, valid for
        // the frame's lifetime (see the field doc).
        unsafe { &*self.chunk }
    }

    /// The frame's code bytes, via the cached pointer (no downcast).
    #[inline]
    pub fn code(&self) -> &[u8] {
        &self.chunk().code
    }
}
