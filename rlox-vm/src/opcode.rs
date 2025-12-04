use std::{
    fmt::{self, Debug},
    io::{self, Write},
};

use thiserror::Error;

use crate::{
    chunk::Chunk,
    enconding::{Decode, Encode},
    value::Addr,
};

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum OpCode {
    NoOp = 0x00,
    Return = 0x01,
    Constant(Addr) = 0x02,
    Neg = 0x03,
    Add = 0x04,
    Sub = 0x05,
    Mul = 0x06,
    Div = 0x07,
}

impl Decode for OpCode {
    type Err = OpDecodeError;

    fn decode(buf: &[u8]) -> Result<(Self, usize), Self::Err> {
        assert!(!buf.is_empty());
        match (buf[0], buf.len()) {
            (0x00, _) => Ok((OpCode::NoOp, 1)),
            (0x01, _) => Ok((OpCode::Return, 1)),
            (0x02, 2..) => Ok((OpCode::Constant(buf[1]), 2)),
            (0x02, few) => Err(OpDecodeError::insufficient(2, few)),
            (0x03, _) => Ok((OpCode::Neg, 1)),
            (0x04, _) => Ok((OpCode::Add, 1)),
            (0x05, _) => Ok((OpCode::Sub, 1)),
            (0x06, _) => Ok((OpCode::Mul, 1)),
            (0x07, _) => Ok((OpCode::Div, 1)),
            (unknown, _) => Err(OpDecodeError::unknown(unknown)),
        }
    }
}

impl Encode for OpCode {
    fn encode<W: Write + ?Sized>(&self, writer: &mut W) -> io::Result<usize> {
        match self {
            OpCode::NoOp => writer.write_all(&[0x00]).map(|_| 1),
            OpCode::Return => writer.write_all(&[0x01]).map(|_| 1),
            OpCode::Constant(addr) => writer.write_all(&[0x02, *addr]).map(|_| 2),
            OpCode::Neg => writer.write_all(&[0x03]).map(|_| 1),
            OpCode::Add => writer.write_all(&[0x04]).map(|_| 1),
            OpCode::Sub => writer.write_all(&[0x05]).map(|_| 1),
            OpCode::Mul => writer.write_all(&[0x06]).map(|_| 1),
            OpCode::Div => writer.write_all(&[0x07]).map(|_| 1),
        }
    }
}

impl OpCode {
    pub fn disassemble(&self, f: &mut fmt::Formatter<'_>, chunk: &Chunk) -> fmt::Result {
        match self {
            OpCode::NoOp => write!(f, "NOOP"),
            OpCode::Return => write!(f, "OP_RETURN"),
            OpCode::Constant(addr) => {
                let constant = chunk.constants[*addr as usize];
                write!(f, "{:<16} {:<4}[{addr:<03}]", "OP_CONSTANT", constant)
            }
            OpCode::Neg => write!(f, "OP_NEG"),
            OpCode::Add => write!(f, "OP_ADD"),
            OpCode::Sub => write!(f, "OP_SUB"),
            OpCode::Mul => write!(f, "OP_MUL"),
            OpCode::Div => write!(f, "OP_DIV"),
        }
    }
}

#[derive(Debug, Error)]
pub enum OpDecodeError {
    #[error("unknown op code: {0}")]
    UnknownOpCode(u8),
    #[error("needed {needed} bytes, found {available}")]
    InsufficientBytes { needed: usize, available: usize },
}

impl OpDecodeError {
    fn unknown(byte: u8) -> Self {
        Self::UnknownOpCode(byte)
    }

    fn insufficient(needed: usize, available: usize) -> Self {
        Self::InsufficientBytes { needed, available }
    }
}
