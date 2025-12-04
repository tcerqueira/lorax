use std::io::Cursor;

use thiserror::Error;

use crate::{
    chunk::Chunk,
    enconding::{DecodeError, OpDecoder},
    opcode::{OpCode, OpDecodeError},
};

pub struct VirtualMachine;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Decode(#[from] DecodeError<OpDecodeError>),
}

impl VirtualMachine {
    pub fn interpret(&mut self, chunk: Chunk) -> Result<(), Error> {
        self.run(chunk)
    }

    pub fn run(&mut self, chunk: Chunk) -> Result<(), Error> {
        let mut pc = Cursor::new(chunk.code.as_slice());
        while let Some(ins) = pc.decode_op::<OpCode>()? {
            // println!("--> {:?}", ins);
            match ins {
                OpCode::NoOp => {}
                OpCode::OpReturn => {}
                OpCode::OpConstant(addr) => {
                    let constant = chunk.constant(addr);
                    println!("{constant}");
                }
            }
        }
        Ok(())
    }
}
