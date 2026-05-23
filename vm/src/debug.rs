use std::borrow::Cow;
use std::ops::Range;
use std::{fmt, io::Cursor};

use report::Span;

use crate::chunk::Chunk;
use crate::enconding::OpDecoder;
use crate::opcode::{OpCode, Slot};
use crate::value::Addr;

pub struct LineInfo {
    pub line: u32,
    pub byte_range: Range<u64>,
}

impl LineInfo {
    pub fn to_span(&self) -> Span {
        Span {
            start: 0,
            end: 0,
            line_start: self.line,
            line_end: self.line,
        }
    }
}

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
        let mut decoder = Cursor::new(self.chunk.code.as_slice());
        let mut line_iter = self.chunk.lines.iter();

        writeln!(self.f, "{:<6}== {} ==", "", self.name)?;
        let mut offset = decoder.position();
        let mut curr_line = line_iter.next();
        let mut prev_line = 0;

        while let Ok(Some(instruction)) = decoder.decode_op::<OpCode>() {
            let line_str = loop {
                break match curr_line {
                    Some(line_info) if line_info.byte_range.contains(&offset) => {
                        if line_info.line != prev_line {
                            prev_line = line_info.line;
                            line_info.line.to_string().into()
                        } else {
                            "|".into()
                        }
                    }
                    Some(line_info) if line_info.byte_range.start > offset => "?".into(),
                    None => "?".into(),
                    Some(line_info) if line_info.byte_range.end <= offset => {
                        curr_line = line_iter.next();
                        continue;
                    }
                    _ => unreachable!("did i not cover all the cases?"),
                };
            };

            self.disassemble_instruction(instruction, offset, line_str)?;
            offset = decoder.position();
            writeln!(self.f)?;
        }
        Ok(())
    }

    pub fn disassemble_instruction(
        &mut self,
        opcode: OpCode,
        offset: u64,
        line_str: Cow<'_, str>,
    ) -> fmt::Result {
        write!(self.f, "{offset:04} {line_str:>4} ")?;
        opcode.disassemble(self.f, self.chunk)
    }
}

impl OpCode {
    pub fn disassemble(&self, f: &mut fmt::Formatter<'_>, chunk: &Chunk) -> fmt::Result {
        let write_addr = |f: &mut fmt::Formatter<'_>, verb: &'static str, addr: &Addr| {
            let constant = &chunk.constants[*addr as usize];
            write!(f, "{:<16} {:<4}[{addr:<03}]", verb, constant)
        };
        let write_slot = |f: &mut fmt::Formatter<'_>, verb: &'static str, slot: &Slot| {
            write!(f, "{:<16} [{slot:<03}]", verb)
        };
        match self {
            OpCode::NoOp => write!(f, "NOOP"),
            OpCode::Return => write!(f, "OP_RETURN"),
            OpCode::Constant(addr) => write_addr(f, "OP_CONSTANT", addr),
            OpCode::Neg => write!(f, "OP_NEG"),
            OpCode::Add => write!(f, "OP_ADD"),
            OpCode::Sub => write!(f, "OP_SUB"),
            OpCode::Mul => write!(f, "OP_MUL"),
            OpCode::Div => write!(f, "OP_DIV"),
            OpCode::True => write!(f, "OP_TRUE"),
            OpCode::False => write!(f, "OP_FALSE"),
            OpCode::Nil => write!(f, "OP_NIL"),
            OpCode::Not => write!(f, "OP_NOT"),
            OpCode::Equal => write!(f, "OP_EQUAL"),
            OpCode::Greater => write!(f, "OP_GREATER"),
            OpCode::Less => write!(f, "OP_LESS"),
            OpCode::Print => write!(f, "OP_PRINT"),
            OpCode::Pop => write!(f, "OP_POP"),
            OpCode::DefineGlobal(addr) => write_addr(f, "OP_DEFINE_GLOBAL", addr),
            OpCode::GetGlobal(addr) => write_addr(f, "OP_GET_GLOBAL", addr),
            OpCode::SetGlobal(addr) => write_addr(f, "OP_SET_GLOBAL", addr),
            OpCode::GetLocal(slot) => write_slot(f, "OP_GET_LOCAL", slot),
            OpCode::SetLocal(slot) => write_slot(f, "OP_SET_LOCAL", slot),
        }
    }
}
