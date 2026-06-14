use std::io::Cursor;

use crate::chunk::Chunk;

pub struct CallFrame {
    pub pc: Cursor<Chunk>,
    pub stack_start: usize,
}

// enum FrameSource {
//     TopLevel(Chunk),
//     Function(UnsafeRef<LoxFunction>),
// }

impl CallFrame {
    pub fn new(chunk: Chunk, stack_start: usize) -> Self {
        Self {
            pc: Cursor::new(chunk),
            stack_start,
        }
    }

    pub fn chunk(&self) -> &Chunk {
        self.pc.get_ref()
    }
}
