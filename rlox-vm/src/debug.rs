use std::borrow::Cow;
use std::ops::Range;
use std::{fmt, io::Cursor};

use crate::chunk::Chunk;
use crate::enconding::OpDecoder;
use crate::opcode::OpCode;

pub struct LineInfo {
    pub line: u32,
    pub byte_range: Range<u64>,
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
