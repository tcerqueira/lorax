use std::fmt::{self, Debug};

use crate::{
    debug::{Disassembler, LineInfo},
    enconding::OpEncoder,
    opcode::OpCode,
    value::{Addr, Value},
};

#[derive(Default)]
pub struct Chunk {
    // TODO: make it generic over Read (?)
    pub(crate) code: Vec<u8>,
    pub(crate) constants: Vec<Value>,
    pub(crate) lines: Vec<LineInfo>,
    pub(crate) label: Option<Box<str>>,
}

impl Chunk {
    pub fn with_label(label: Box<str>) -> Self {
        Self {
            label: Some(label),
            ..Self::default()
        }
    }

    pub fn write(&mut self, instruction: OpCode) {
        self.code
            .encode_op(&instruction)
            .expect("what could go wrong :)");
    }

    pub fn write_with_line(&mut self, instruction: OpCode, line: u32) {
        let start_offset = self.code.len() as u64;
        self.write(instruction);

        let last_byte_offset = self.code.len() as u64;
        match self.lines.last_mut() {
            Some(line_info) if line_info.line == line => {
                line_info.byte_range.end = last_byte_offset
            }
            _ => self.lines.push(LineInfo {
                line,
                byte_range: start_offset..last_byte_offset,
            }),
        };
    }

    pub fn write_constant(&mut self, value: Value) {
        let addr = self.add_constant(value);
        self.write(OpCode::Constant(addr));
    }

    pub fn write_constant_with_line(&mut self, value: Value, line: u32) {
        let addr = self.add_constant(value);
        self.write_with_line(OpCode::Constant(addr), line);
    }

    fn add_constant(&mut self, value: Value) -> Addr {
        assert!(
            self.constants.len() < u8::MAX as usize,
            "can't have more than 255 constants per chunk"
        );
        self.constants.push(value);
        (self.constants.len() - 1) as u8
    }

    pub fn constant(&self, addr: Addr) -> Value {
        self.constants[addr as usize]
    }
}

impl Debug for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut disassembler = Disassembler::new(f, self, self.label.as_deref().unwrap_or("Chunk"));
        disassembler.disassemble_chunk()
    }
}
