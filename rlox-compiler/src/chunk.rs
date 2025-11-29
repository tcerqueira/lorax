use std::{
    fmt::{self, Debug},
    io::Cursor,
};

use crate::{
    debug::Disassembler,
    enconding::OpEncoder,
    opcode::OpCode,
    value::{Addr, Value},
};

#[derive(Default)]
pub struct Chunk {
    pub(crate) code: Cursor<Vec<u8>>,
    pub(crate) constants: Vec<Value>,
}

impl Chunk {
    pub fn add_constant(&mut self, value: Value) -> Addr {
        assert!(
            self.constants.len() < u8::MAX as usize,
            "can't have more than 255 constants per chunk"
        );
        self.constants.push(value);
        (self.constants.len() - 1) as u8
    }

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
