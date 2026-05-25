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

fn read<const N: usize>(buf: &mut &[u8]) -> Result<[u8; N], DecodeError> {
    let Some((head, rest)) = buf.split_first_chunk::<N>() else {
        return Err(DecodeError::InsufficientBytes {
            needed: N,
            available: buf.len(),
        });
    };
    *buf = rest;
    Ok(*head)
}

fn read_one(buf: &mut &[u8]) -> Result<u8, DecodeError> {
    read::<1>(buf).map(|[b]| b)
}

impl Decode for OpCode {
    fn decode(buf: &mut &[u8]) -> Result<Self, DecodeError> {
        let tag = read_one(buf)?;
        let op = match tag {
            0x00 => OpCode::NoOp,
            0x01 => OpCode::Return,
            0x02 => OpCode::Constant(read_one(buf)?),
            0x03 => OpCode::Neg,
            0x04 => OpCode::Add,
            0x05 => OpCode::Sub,
            0x06 => OpCode::Mul,
            0x07 => OpCode::Div,
            0x08 => OpCode::True,
            0x09 => OpCode::False,
            0x0A => OpCode::Nil,
            0x0B => OpCode::Not,
            0x0C => OpCode::Equal,
            0x0D => OpCode::Greater,
            0x0E => OpCode::Less,
            0x0F => OpCode::Print,
            0x10 => OpCode::Pop,
            0x11 => OpCode::DefineGlobal(read_one(buf)?),
            0x12 => OpCode::GetGlobal(read_one(buf)?),
            0x13 => OpCode::SetGlobal(read_one(buf)?),
            0x14 => OpCode::GetLocal(read_one(buf)?),
            0x15 => OpCode::SetLocal(read_one(buf)?),
            0x16 => OpCode::PopN(read_one(buf)?),
            unknown => return Err(DecodeError::UnknownOpCode(unknown)),
        };
        Ok(op)
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
