use std::io::{self, Write};

use thiserror::Error;

use crate::value::Addr;

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
    Loop(Offset) = 0x19,
    Call(u8) = 0x1A,
    /// Wrap the function constant at `Addr` in a closure. Followed in the code
    /// stream by `2 * upvalue_count` raw bytes — an `(is_local, index)` pair per
    /// upvalue — which the VM and disassembler read out of band (the count comes
    /// from the function itself).
    Closure(Addr) = 0x1B,
    GetUpvalue(Slot) = 0x1C,
    SetUpvalue(Slot) = 0x1D,
    CloseUpvalue = 0x1E,
    Class(Addr) = 0x1F,
    GetProperty(Addr) = 0x20,
    SetProperty(Addr) = 0x21,
    /// Bind the closure on top of the stack as a method (by name at `Addr`) on
    /// the class beneath it.
    Method(Addr) = 0x22,
    /// Fused property-get + call for `recv.name(args)`: name constant, arg count.
    Invoke(Addr, u8) = 0x23,
    /// Copy the superclass's methods into the subclass (copy-down inheritance).
    Inherit = 0x24,
    GetSuper(Addr) = 0x25,
    /// Fused super-method get + call: method-name constant, arg count.
    SuperInvoke(Addr, u8) = 0x26,
}

pub type Slot = u8;
pub type Offset = u16;

#[derive(Debug, Error)]
pub enum DecodeError {
    #[error("unknown op code: {0}")]
    UnknownOpCode(u8),
    #[error("unexpected end of bytecode while decoding an operand")]
    UnexpectedEnd,
}

pub trait Encode {
    fn encode<W: Write + ?Sized>(&self, writer: &mut W) -> io::Result<usize>;
}

pub trait OpEncoder: Write {
    fn encode_op<T>(&mut self, opcode: &T) -> io::Result<usize>
    where
        T: Encode,
    {
        opcode.encode(self)
    }
}

impl<W> OpEncoder for W where W: Write {}

fn next_u8(code: &[u8], ip: &mut usize) -> Result<u8, DecodeError> {
    let byte = *code.get(*ip).ok_or(DecodeError::UnexpectedEnd)?;
    *ip += 1;
    Ok(byte)
}

fn next_offset(code: &[u8], ip: &mut usize) -> Result<Offset, DecodeError> {
    let lo = *code.get(*ip).ok_or(DecodeError::UnexpectedEnd)?;
    let hi = *code.get(*ip + 1).ok_or(DecodeError::UnexpectedEnd)?;
    *ip += 2;
    Ok(Offset::from_le_bytes([lo, hi]))
}

impl OpCode {
    /// Decode the opcode at `code[*ip]`, advancing `*ip` past the tag and its
    /// inline operands. The VM dispatch loop and the disassembler share this as
    /// the single decode path; the compiler's `Encode` is its inverse.
    pub fn decode_at(code: &[u8], ip: &mut usize) -> Result<OpCode, DecodeError> {
        let op = match next_u8(code, ip)? {
            0x00 => OpCode::NoOp,
            0x01 => OpCode::Ret,
            0x02 => OpCode::Constant(next_u8(code, ip)?),
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
            0x11 => OpCode::DefGlobal(next_u8(code, ip)?),
            0x12 => OpCode::GetGlobal(next_u8(code, ip)?),
            0x13 => OpCode::SetGlobal(next_u8(code, ip)?),
            0x14 => OpCode::GetLocal(next_u8(code, ip)?),
            0x15 => OpCode::SetLocal(next_u8(code, ip)?),
            0x16 => OpCode::PopN(next_u8(code, ip)?),
            0x17 => OpCode::JmpIfFalse(next_offset(code, ip)?),
            0x18 => OpCode::Jmp(next_offset(code, ip)?),
            0x19 => OpCode::Loop(next_offset(code, ip)?),
            0x1A => OpCode::Call(next_u8(code, ip)?),
            0x1B => OpCode::Closure(next_u8(code, ip)?),
            0x1C => OpCode::GetUpvalue(next_u8(code, ip)?),
            0x1D => OpCode::SetUpvalue(next_u8(code, ip)?),
            0x1E => OpCode::CloseUpvalue,
            0x1F => OpCode::Class(next_u8(code, ip)?),
            0x20 => OpCode::GetProperty(next_u8(code, ip)?),
            0x21 => OpCode::SetProperty(next_u8(code, ip)?),
            0x22 => OpCode::Method(next_u8(code, ip)?),
            0x23 => OpCode::Invoke(next_u8(code, ip)?, next_u8(code, ip)?),
            0x24 => OpCode::Inherit,
            0x25 => OpCode::GetSuper(next_u8(code, ip)?),
            0x26 => OpCode::SuperInvoke(next_u8(code, ip)?, next_u8(code, ip)?),
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
            OpCode::Loop(offset) => {
                let buf = offset.to_le_bytes();
                write(&[0x19, buf[0], buf[1]])
            }
            OpCode::Call(arg_count) => write(&[0x1A, *arg_count]),
            OpCode::Closure(addr) => write(&[0x1B, *addr]),
            OpCode::GetUpvalue(slot) => write(&[0x1C, *slot]),
            OpCode::SetUpvalue(slot) => write(&[0x1D, *slot]),
            OpCode::CloseUpvalue => write(&[0x1E]),
            OpCode::Class(addr) => write(&[0x1F, *addr]),
            OpCode::GetProperty(addr) => write(&[0x20, *addr]),
            OpCode::SetProperty(addr) => write(&[0x21, *addr]),
            OpCode::Method(addr) => write(&[0x22, *addr]),
            OpCode::Invoke(addr, arg_count) => write(&[0x23, *addr, *arg_count]),
            OpCode::Inherit => write(&[0x24]),
            OpCode::GetSuper(addr) => write(&[0x25, *addr]),
            OpCode::SuperInvoke(addr, arg_count) => write(&[0x26, *addr, *arg_count]),
        }
    }
}
