use std::{
    fmt::Debug,
    io::{self, Read, Write},
};

use crate::{
    enconding::{Decode, DecodeError, Encode},
    value::Addr,
};

// PERF: maybe just read bytes in VM instead of storing in the enum
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum OpCode {
    NoOp = 0x00,
    Ret = 0x01,
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
    DefGlobal(Addr) = 0x11,
    GetGlobal(Addr) = 0x12,
    SetGlobal(Addr) = 0x13,
    GetLocal(Slot) = 0x14,
    SetLocal(Slot) = 0x15,
    PopN(u8) = 0x16,
    JmpIfFalse(Offset) = 0x17,
    Jmp(Offset) = 0x18,
}

pub type Slot = u8;
pub type Offset = u16;

fn read<R: Read + ?Sized, const N: usize>(reader: &mut R) -> Result<[u8; N], DecodeError> {
    let mut buf = [0u8; N];
    reader.read_exact(&mut buf)?;
    Ok(buf)
}

fn read_one<R: Read + ?Sized>(reader: &mut R) -> Result<u8, DecodeError> {
    read::<R, 1>(reader).map(|[b]| b)
}

impl Decode for OpCode {
    fn decode<R: Read + ?Sized>(reader: &mut R) -> Result<Self, DecodeError> {
        let tag = read_one(reader)?;
        let op = match tag {
            0x00 => OpCode::NoOp,
            0x01 => OpCode::Ret,
            0x02 => OpCode::Constant(read_one(reader)?),
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
            0x11 => OpCode::DefGlobal(read_one(reader)?),
            0x12 => OpCode::GetGlobal(read_one(reader)?),
            0x13 => OpCode::SetGlobal(read_one(reader)?),
            0x14 => OpCode::GetLocal(read_one(reader)?),
            0x15 => OpCode::SetLocal(read_one(reader)?),
            0x16 => OpCode::PopN(read_one(reader)?),
            0x17 => OpCode::JmpIfFalse(Offset::from_le_bytes(read::<_, 2>(reader)?)),
            0x18 => OpCode::Jmp(Offset::from_le_bytes(read::<_, 2>(reader)?)),
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
            OpCode::Ret => write(&[0x01]),
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
            OpCode::DefGlobal(addr) => write(&[0x11, *addr]),
            OpCode::GetGlobal(addr) => write(&[0x12, *addr]),
            OpCode::SetGlobal(addr) => write(&[0x13, *addr]),
            OpCode::GetLocal(slot) => write(&[0x14, *slot]),
            OpCode::SetLocal(slot) => write(&[0x15, *slot]),
            OpCode::PopN(n) => write(&[0x16, *n]),
            OpCode::JmpIfFalse(offset) => {
                let buf = offset.to_le_bytes();
                write(&[0x17, buf[0], buf[1]])
            }
            OpCode::Jmp(offset) => {
                let buf = offset.to_le_bytes();
                write(&[0x18, buf[0], buf[1]])
            }
        }
    }
}
