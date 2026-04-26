use std::{
    fmt::Debug,
    io::{self, Write},
};

use thiserror::Error;

use crate::{
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
    True = 0x08,
    False = 0x09,
    Nil = 0x0A,
    Not = 0x0B,
    Equal = 0x0C,
    Greater = 0x0D,
    Less = 0x0E,
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
            (0x08, _) => Ok((OpCode::True, 1)),
            (0x09, _) => Ok((OpCode::False, 1)),
            (0x0A, _) => Ok((OpCode::Nil, 1)),
            (0x0B, _) => Ok((OpCode::Not, 1)),
            (0x0C, _) => Ok((OpCode::Equal, 1)),
            (0x0D, _) => Ok((OpCode::Greater, 1)),
            (0x0E, _) => Ok((OpCode::Less, 1)),
            (unknown, _) => Err(OpDecodeError::unknown(unknown)),
        }
    }
}

impl Encode for OpCode {
    fn encode<W: Write + ?Sized>(&self, writer: &mut W) -> io::Result<usize> {
        let mut write = |buf: &[u8]| {
            let len = buf.len();
            writer.write_all(buf).map(|_| len)
        };
        match self {
            OpCode::NoOp => write(&[0x00]),
            OpCode::Return => write(&[0x01]),
            OpCode::Constant(addr) => write(&[0x02, *addr]),
            OpCode::Neg => write(&[0x03]),
            OpCode::Add => write(&[0x04]),
            OpCode::Sub => write(&[0x05]),
            OpCode::Mul => write(&[0x06]),
            OpCode::Div => write(&[0x07]),
            OpCode::True => write(&[0x08]),
            OpCode::False => write(&[0x09]),
            OpCode::Nil => write(&[0x0A]),
            OpCode::Not => write(&[0x0B]),
            OpCode::Equal => write(&[0x0C]),
            OpCode::Greater => write(&[0x0D]),
            OpCode::Less => write(&[0x0E]),
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
