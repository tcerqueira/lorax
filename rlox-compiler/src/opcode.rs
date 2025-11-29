use std::{
    fmt::{self, Debug},
    io::{self, Write},
};

use thiserror::Error;

use crate::enconding::{Decode, Encode};

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum OpCode {
    NoOp = 0x00,
    OpReturn = 0x01,
}

#[derive(Debug, Error)]
pub enum OpDecodeError {
    #[error("unknown op code: {0}")]
    UnknownOpCode(u8),
    #[error("needed {needed} bytes, found {available}")]
    InsufficientBytes { needed: usize, available: usize },
}

impl Decode for OpCode {
    type Err = OpDecodeError;

    fn decode(buf: &[u8]) -> Result<(Self, usize), Self::Err> {
        debug_assert!(!buf.is_empty());

        match buf[0] {
            0x00 => Ok((OpCode::NoOp, 1)),
            0x01 => Ok((OpCode::OpReturn, 1)),
            unknown => Err(OpDecodeError::UnknownOpCode(unknown)),
        }
    }
}

impl Encode for OpCode {
    fn encode<W: Write + ?Sized>(&self, writer: &mut W) -> io::Result<usize> {
        match self {
            OpCode::NoOp => writer.write_all(&[0x00]).map(|_| 1),
            OpCode::OpReturn => writer.write_all(&[0x01]).map(|_| 1),
        }
    }
}

impl Debug for OpCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OpCode::NoOp => f.write_str("NOOP"),
            OpCode::OpReturn => f.write_str("OP_RETURN"),
        }
    }
}
