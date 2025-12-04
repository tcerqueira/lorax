use std::io::Cursor;

use thiserror::Error;

use crate::{
    chunk::Chunk,
    enconding::{DecodeError, OpDecoder},
    opcode::{OpCode, OpDecodeError},
    value::Value,
};

#[derive(Default)]
pub struct VirtualMachine {
    stack: Vec<Value>,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Decode(#[from] DecodeError<OpDecodeError>),
    #[error("runtime error: {0}")]
    Runtime(String),
}

impl VirtualMachine {
    pub fn interpret(&mut self, chunk: Chunk) -> Result<(), Error> {
        self.run(chunk)
    }

    pub fn run(&mut self, chunk: Chunk) -> Result<(), Error> {
        let mut pc = Cursor::new(chunk.code.as_slice());
        while let Some(ins) = pc.decode_op::<OpCode>()? {
            // self.trace(ins);
            match ins {
                OpCode::NoOp => {}
                OpCode::Return => {
                    let a = self.stack_pop()?;
                    println!("{a}");
                    return Ok(());
                }
                OpCode::Constant(addr) => {
                    let constant = chunk.constant(addr);
                    self.stack_push(constant);
                }
                OpCode::Neg => {
                    let a = self.stack_pop()?;
                    self.stack.push(-a);
                }
                OpCode::Add => {
                    let b = self.stack_pop()?;
                    let a = self.stack_pop()?;
                    let res = a + b;
                    self.stack_push(res);
                }
                OpCode::Sub => {
                    let b = self.stack_pop()?;
                    let a = self.stack_pop()?;
                    let res = a - b;
                    self.stack_push(res);
                }
                OpCode::Mul => {
                    let b = self.stack_pop()?;
                    let a = self.stack_pop()?;
                    let res = a * b;
                    self.stack_push(res);
                }
                OpCode::Div => {
                    let b = self.stack_pop()?;
                    let a = self.stack_pop()?;
                    let res = a / b;
                    self.stack_push(res);
                }
            }
        }
        Ok(())
    }

    fn stack_push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn stack_pop(&mut self) -> Result<Value, Error> {
        let stack_err_msg = "nothing on stack when attempted to pop";
        self.stack.pop().ok_or(Error::Runtime(stack_err_msg.into()))
    }

    #[expect(dead_code)]
    fn trace(&self, ins: OpCode) {
        println!("--> {:?}", ins);
        print!("--> {:>16}", "stack: [ ");
        let mut stack_iter = self.stack.iter().peekable();
        while let Some(value) = stack_iter.next() {
            print!("{value}");
            if stack_iter.peek().is_some() {
                print!(", ");
            }
        }
        println!(" ]");
    }
}
