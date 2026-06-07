use std::borrow::Cow;
use std::fmt;
use std::fmt::Display;
use std::ops::Range;

use report::Span;
use serde::{Deserialize, Serialize};

use crate::chunk::Chunk;
use crate::enconding::OpCode;
use crate::value::Addr;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
        let code = self.chunk.code.as_slice();
        let mut line_iter = self.chunk.lines.iter();

        writeln!(self.f, "{:<6}== {} ==", "", self.name)?;
        let mut ip = 0usize;
        let mut curr_line = line_iter.next();
        let mut prev_line = 0;

        while ip < code.len() {
            let offset = ip as u64;
            let Ok(instruction) = OpCode::decode_at(code, &mut ip) else {
                break;
            };
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
            writeln!(self.f)?;

            // A closure carries `2 * upvalue_count` trailing operand bytes that
            // are not opcodes; step the cursor past them so decoding stays aligned.
            if let OpCode::Closure(addr) = instruction {
                ip += 2 * self.chunk.closure_upvalue_count(addr);
            }
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

fn write_args1(f: &mut fmt::Formatter<'_>, verb: &'static str, arg: impl Display) -> fmt::Result {
    write!(f, "{:<16} [{arg:<05}]", verb)
}

impl OpCode {
    pub fn disassemble(&self, f: &mut fmt::Formatter<'_>, chunk: &Chunk) -> fmt::Result {
        let write_addr = |f: &mut fmt::Formatter<'_>, verb: &'static str, addr: &Addr| {
            let constant = &chunk.constants[*addr as usize];
            write!(f, "{:<16} {:<4}[{addr:<03}]", verb, constant)
        };

        match self {
            OpCode::NoOp => write!(f, "NOOP"),
            OpCode::Ret => write!(f, "OP_RETURN"),
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
            OpCode::DefGlobal(addr) => write_addr(f, "OP_DEFINE_GLOBAL", addr),
            OpCode::GetGlobal(addr) => write_addr(f, "OP_GET_GLOBAL", addr),
            OpCode::SetGlobal(addr) => write_addr(f, "OP_SET_GLOBAL", addr),
            OpCode::GetLocal(slot) => write_args1(f, "OP_GET_LOCAL", slot),
            OpCode::SetLocal(slot) => write_args1(f, "OP_SET_LOCAL", slot),
            OpCode::PopN(n) => write_args1(f, "OP_POPN", n),
            OpCode::JmpIfFalse(offset) => write_args1(f, "OP_JMP_IF_FALSE", offset),
            OpCode::Jmp(offset) => write_args1(f, "OP_JMP", offset),
            OpCode::Loop(offset) => write_args1(f, "OP_LOOP", offset),
            OpCode::Call(arg_count) => write_args1(f, "OP_CALL", arg_count),
            OpCode::Closure(addr) => write_addr(f, "OP_CLOSURE", addr),
            OpCode::GetUpvalue(slot) => write_args1(f, "OP_GET_UPVALUE", slot),
            OpCode::SetUpvalue(slot) => write_args1(f, "OP_SET_UPVALUE", slot),
            OpCode::CloseUpvalue => write!(f, "OP_CLOSE_UPVALUE"),
            OpCode::Class(addr) => write_addr(f, "OP_CLASS", addr),
            OpCode::GetProperty(addr) => write_addr(f, "OP_GET_PROPERTY", addr),
            OpCode::SetProperty(addr) => write_addr(f, "OP_SET_PROPERTY", addr),
            OpCode::Method(addr) => write_addr(f, "OP_METHOD", addr),
            OpCode::Invoke(addr, arg_count) => {
                let name = &chunk.constants[*addr as usize];
                write!(
                    f,
                    "{:<16} {name:<4}[{addr:<03}] ({arg_count} args)",
                    "OP_INVOKE"
                )
            }
            OpCode::Inherit => write!(f, "OP_INHERIT"),
            OpCode::GetSuper(addr) => write_addr(f, "OP_GET_SUPER", addr),
            OpCode::SuperInvoke(addr, arg_count) => {
                let name = &chunk.constants[*addr as usize];
                write!(
                    f,
                    "{:<16} {name:<4}[{addr:<03}] ({arg_count} args)",
                    "OP_SUPER_INVOKE"
                )
            }
        }
    }
}
