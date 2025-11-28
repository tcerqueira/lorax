use std::fmt::{self, Debug};

use crate::debug::Disassembler;

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum OpCode {
    OpReturn,
}

impl Debug for OpCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OpCode::OpReturn => f.write_str("OP_RETURN"),
        }
    }
}

#[derive(Default)]
pub struct Chunk {
    pub(crate) code: Vec<OpCode>,
}

impl Chunk {
    pub fn write(&mut self, instruction: OpCode) {
        self.code.push(instruction);
    }
}

impl Debug for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut disassembler = Disassembler::new(f, self, "Chunk");
        disassembler.disassemble_chunk()
    }
}
