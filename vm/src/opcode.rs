use std::{
    fmt::Debug,
    io::{self, Write},
};

use crate::{
    enconding::{Decode, DecodeError, Encode},
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
    Print = 0x0F,
    Pop = 0x10,
    DefineGlobal(Addr) = 0x11,
    GetGlobal(Addr) = 0x12,
    SetGlobal(Addr) = 0x13,
    GetLocal(Slot) = 0x14,
    SetLocal(Slot) = 0x15,
    PopN(u8) = 0x16,
}

pub type Slot = u8;

impl Decode for OpCode {
    fn decode(buf: &[u8]) -> Result<(Self, usize), DecodeError> {
        assert!(!buf.is_empty());
        match (buf[0], buf.len()) {
            (0x00, _) => Ok((OpCode::NoOp, 1)),
            (0x01, _) => Ok((OpCode::Return, 1)),
            (0x02, 2..) => Ok((OpCode::Constant(buf[1]), 2)),
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
            (0x0F, _) => Ok((OpCode::Print, 1)),
            (0x10, _) => Ok((OpCode::Pop, 1)),
            (0x11, 2..) => Ok((OpCode::DefineGlobal(buf[1]), 2)),
            (0x12, 2..) => Ok((OpCode::GetGlobal(buf[1]), 2)),
            (0x13, 2..) => Ok((OpCode::SetGlobal(buf[1]), 2)),
            (0x14, 2..) => Ok((OpCode::GetLocal(buf[1]), 2)),
            (0x15, 2..) => Ok((OpCode::SetLocal(buf[1]), 2)),
            (0x16, 2..) => Ok((OpCode::PopN(buf[1]), 2)),
            (0x02 | 0x11 | 0x12 | 0x13 | 0x14 | 0x15 | 0x16, few) => {
                Err(DecodeError::InsufficientBytes {
                    needed: 2,
                    available: few,
                })
            }
            (unknown, _) => Err(DecodeError::UnknownOpCode(unknown)),
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
            OpCode::Print => write(&[0x0F]),
            OpCode::Pop => write(&[0x10]),
            OpCode::DefineGlobal(addr) => write(&[0x11, *addr]),
            OpCode::GetGlobal(addr) => write(&[0x12, *addr]),
            OpCode::SetGlobal(addr) => write(&[0x13, *addr]),
            OpCode::GetLocal(slot) => write(&[0x14, *slot]),
            OpCode::SetLocal(slot) => write(&[0x15, *slot]),
            OpCode::PopN(n) => write(&[0x16, *n]),
        }
    }
}

