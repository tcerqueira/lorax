use std::{
    fmt::{self, Debug},
    io::Cursor,
};

use crate::{debug::Disassembler, enconding::OpEncoder, opcode::OpCode};

#[derive(Default)]
pub struct Chunk {
    pub(crate) code: Cursor<Vec<u8>>,
}

impl Chunk {
    pub fn write(&mut self, instruction: OpCode) {
        self.code
            .encode_op(&instruction)
            .expect("what could go wrong :)");
    }
}

impl Debug for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut disassembler = Disassembler::new(f, self, "Chunk");
        disassembler.disassemble_chunk()
    }
}
