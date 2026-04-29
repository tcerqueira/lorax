use std::{
    io::{Cursor, Seek},
    ops::{Add, Div, Mul, Sub},
};

use report::error::RuntimeError;
use thiserror::Error;

use crate::{
    chunk::Chunk,
    debug::LineInfo,
    enconding::{DecodeError, OpDecoder},
    object::pool::ObjectPool,
    opcode::{OpCode, OpDecodeError},
    value::{Value, ValueError},
};

#[derive(Default)]
pub struct VirtualMachine {
    stack: Vec<Value>,
    _heap: ObjectPool,
}

#[derive(Debug, Error)]
pub enum VirtualMachineError {
    #[error(transparent)]
    Decode(#[from] DecodeError<OpDecodeError>),
    #[error(transparent)]
    Runtime(#[from] RuntimeError),
}

impl VirtualMachine {
    pub fn run(&mut self, chunk: Chunk) -> Result<(), VirtualMachineError> {
        // println!("{chunk:?}");
        let mut pc = Cursor::new(chunk.code.as_slice());
        while let Some(op) = pc.decode_op::<OpCode>()? {
            // self.trace(op);
            let invalid_operand_err = |_: ValueError| {
                let span = chunk
                    .get_line(pc.stream_position().unwrap() - 1)
                    .map(LineInfo::to_span)
                    .unwrap_or_default();
                RuntimeError::custom(span, "invalid operand")
            };

            match op {
                OpCode::NoOp => {}
                OpCode::Return => {
                    let v = self.stack_pop();
                    println!("{v}");
                    return Ok(());
                }
                OpCode::Constant(addr) => {
                    let constant = chunk.constant(addr);
                    self.stack_push(constant.clone());
                }
                OpCode::Neg => {
                    let v = self.stack_top();
                    *v = (-v.clone()).map_err(invalid_operand_err)?;
                }
                OpCode::Add => self.binary_op(Value::add).map_err(invalid_operand_err)?,
                OpCode::Sub => self.binary_op(Value::sub).map_err(invalid_operand_err)?,
                OpCode::Mul => self.binary_op(Value::mul).map_err(invalid_operand_err)?,
                OpCode::Div => self.binary_op(Value::div).map_err(invalid_operand_err)?,
                OpCode::True => {
                    self.stack_push(Value::boolean(true));
                }
                OpCode::False => {
                    self.stack_push(Value::boolean(false));
                }
                OpCode::Nil => {
                    self.stack_push(Value::nil());
                }
                OpCode::Not => {
                    let v = self.stack_top();
                    *v = Value::Boolean(v.is_falsey());
                }
                OpCode::Equal => {
                    let b = self.stack_pop();
                    let a = self.stack_pop();
                    self.stack_push(Value::boolean(a == b));
                }
                OpCode::Greater => self
                    .binary_op(Value::greater)
                    .map_err(invalid_operand_err)?,
                OpCode::Less => self.binary_op(Value::less).map_err(invalid_operand_err)?,
            }
        }
        Ok(())
    }

    fn binary_op<F>(&mut self, op: F) -> Result<(), ValueError>
    where
        F: Fn(Value, Value) -> Result<Value, ValueError>,
    {
        let b = self.stack_pop();
        let a = self.stack_pop();
        let res = op(a, b)?;
        self.stack_push(res);
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
    fn stack_peek(&mut self, distance: usize) -> &mut Value {
        let len = self.stack.len();
        self.stack
            .get_mut(len - distance)
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
