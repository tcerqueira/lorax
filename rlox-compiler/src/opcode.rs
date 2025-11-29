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
    OpReturn = 0x01,
    OpConstant(Addr) = 0x02,
}

impl Decode for OpCode {
    type Err = OpDecodeError;

    fn decode(buf: &[u8]) -> Result<(Self, usize), Self::Err> {
        debug_assert!(!buf.is_empty());

        match (buf[0], buf.len()) {
            (0x00, _) => Ok((OpCode::NoOp, 1)),
            (0x01, _) => Ok((OpCode::OpReturn, 1)),
            (0x02, 2..) => Ok((OpCode::OpConstant(buf[1]), 2)),
            (0x02, few) => Err(OpDecodeError::insufficient(2, few)),
            (unknown, _) => Err(OpDecodeError::unknown(unknown)),
        }
    }
}

impl Encode for OpCode {
    fn encode<W: Write + ?Sized>(&self, writer: &mut W) -> io::Result<usize> {
        match self {
            OpCode::NoOp => writer.write_all(&[0x00]).map(|_| 1),
            OpCode::OpReturn => writer.write_all(&[0x01]).map(|_| 1),
            OpCode::OpConstant(addr) => writer.write_all(&[0x02, *addr]).map(|_| 2),
        }
    }
}

impl OpCode {
    pub fn disassemble(&self, f: &mut fmt::Formatter<'_>, chunk: &Chunk) -> fmt::Result {
        match self {
            OpCode::NoOp => write!(f, "NOOP"),
            OpCode::OpReturn => write!(f, "OP_RETURN"),
            OpCode::OpConstant(addr) => {
                let constant = chunk.constants[*addr as usize];
                write!(f, "{:<16} {:<4?}[{addr:<03}]", "OP_CONSTANT", constant)
            }
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
