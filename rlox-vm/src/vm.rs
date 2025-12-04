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
        while let Some(op) = pc.decode_op::<OpCode>()? {
            // self.trace(ins);
            match op {
                OpCode::NoOp => {}
                OpCode::Return => {
                    let a = self.stack_pop();
                    println!("{a}");
                    return Ok(());
                }
                OpCode::Constant(addr) => {
                    let constant = chunk.constant(addr);
                    self.stack_push(constant);
                }
                OpCode::Neg => {
                    let x = self.stack_top();
                    *x = -*x;
                }
                OpCode::Add => {
                    let b = self.stack_pop();
                    let a = self.stack_pop();
                    let res = a + b;
                    self.stack_push(res);
                }
                OpCode::Sub => {
                    let b = self.stack_pop();
                    let a = self.stack_pop();
                    let res = a - b;
                    self.stack_push(res);
                }
                OpCode::Mul => {
                    let b = self.stack_pop();
                    let a = self.stack_pop();
                    let res = a * b;
                    self.stack_push(res);
                }
                OpCode::Div => {
                    let b = self.stack_pop();
                    let a = self.stack_pop();
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

    fn stack_pop(&mut self) -> Value {
        self.stack
            .pop()
            .expect("compiler bug, nothing to pop on the VM stack")
    }

    fn stack_top(&mut self) -> &mut Value {
        // optimization for ops that pop 1 value and push 1 value
        // allows mutation in place
        self.stack
            .last_mut()
            .expect("compiler bug, nothing on top of the VM stack")
    }

    #[expect(dead_code)]
    fn trace(&self, op: OpCode) {
        println!("--> {:?}", op);
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
