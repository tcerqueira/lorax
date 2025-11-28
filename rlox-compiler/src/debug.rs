use std::fmt;

use crate::chunk::{Chunk, OpCode};

pub struct Disassembler<'a, 'f> {
    f: &'a mut fmt::Formatter<'f>,
    name: &'a str,
    chunk: &'a Chunk,
}

impl<'a, 'f> Disassembler<'a, 'f> {
    pub fn new(f: &'a mut fmt::Formatter<'f>, chunk: &'a Chunk, name: &'a str) -> Self {
        Self { f, name, chunk }
    }

    pub fn disassemble_chunk(&mut self) -> fmt::Result {
        writeln!(self.f, "{:<6}== {} ==", "", self.name)?;
        for (offset, instruction) in self.chunk.code.iter().enumerate() {
            self.disassemble_instruction(offset, *instruction)?;
            writeln!(self.f)?;
        }
        Ok(())
    }

    pub fn disassemble_instruction(&mut self, offset: usize, opcode: OpCode) -> fmt::Result {
        write!(self.f, "{offset:04} | {opcode:<10?}")
    }
}
