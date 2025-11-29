use std::{fmt, io::Cursor};

use crate::chunk::Chunk;
use crate::enconding::OpDecoder;
use crate::opcode::OpCode;

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
        let mut decoder = Cursor::new(self.chunk.code.get_ref());

        writeln!(self.f, "{:<6}== {} ==", "", self.name)?;
        let mut offset = decoder.position();
        while let Ok(instruction) = decoder.decode_op::<OpCode>() {
            self.disassemble_instruction(offset, instruction)?;
            offset = decoder.position();
            writeln!(self.f)?;
        }
        Ok(())
    }

    pub fn disassemble_instruction(&mut self, offset: u64, opcode: OpCode) -> fmt::Result {
        write!(self.f, "{offset:04} | ")?;
        opcode.disassemble(self.f, self.chunk)
    }
}
