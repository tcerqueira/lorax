use std::io::Cursor;

use intrusive_collections::UnsafeRef;

use crate::{chunk::Chunk, object::function::LoxFunction};

pub struct CallFrame {
    pub pc: Cursor<FrameSource>,
    pub stack_start: usize,
}

pub enum FrameSource {
    TopLevel(Chunk),
    Function(UnsafeRef<LoxFunction>),
}

impl AsRef<[u8]> for FrameSource {
    fn as_ref(&self) -> &[u8] {
        match self {
            FrameSource::TopLevel(chunk) => chunk.as_ref(),
            FrameSource::Function(func) => func.chunk.as_ref(),
        }
    }
}

impl CallFrame {
    fn new(source: FrameSource, stack_start: usize) -> Self {
        Self {
            pc: Cursor::new(source),
            stack_start,
        }
    }

    pub fn top_level(chunk: Chunk, stack_start: usize) -> Self {
        Self::new(FrameSource::TopLevel(chunk), stack_start)
    }

    pub fn function(func: UnsafeRef<LoxFunction>, stack_start: usize) -> Self {
        Self::new(FrameSource::Function(func), stack_start)
    }

    pub fn chunk(&self) -> &Chunk {
        match self.pc.get_ref() {
            FrameSource::TopLevel(chunk) => chunk,
            FrameSource::Function(func) => &func.chunk,
        }
    }
}
