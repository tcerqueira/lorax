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
    object::{ObjKind, pool::ObjectPool, string::StringObj},
    opcode::{OpCode, OpDecodeError},
    value::{Value, ValueError},
};

#[derive(Default)]
pub struct VirtualMachine {
    stack: Vec<Value>,
    heap: ObjectPool,
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
                    let v = self.stack_top_mut();
                    *v = (-v.clone()).map_err(invalid_operand_err)?;
                }
                OpCode::Add
                    if let (Value::Object(o1), Value::Object(o2)) =
                        (self.stack_peek(0), self.stack_peek(1))
                        && o1.kind() == ObjKind::String
                        && o2.kind() == ObjKind::String =>
                {
                    self.concatenate_str()
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
                    let v = self.stack_top_mut();
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

    pub fn heap(&mut self) -> &mut ObjectPool {
        &mut self.heap
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

    fn concatenate_str(&mut self) {
        let (Value::Object(b), Value::Object(a)) = (self.stack_pop(), self.stack_pop()) else {
            unreachable!("just matched Object in branch");
        };
        // SAFETY: we only call this function on the string objects branch
        let a = unsafe { a.downcast_ref::<StringObj>() };
        let b = unsafe { b.downcast_ref::<StringObj>() };
        // PERF: create constructor that adds in place, reduces one allocation
        let mut s = a.as_str().to_owned();
        s.push_str(b);
        let obj = self.heap.add(StringObj::boxed(&s));
        self.stack_push(Value::Object(obj));
    }

    fn stack_push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn stack_pop(&mut self) -> Value {
        self.stack
            .pop()
            .expect("compiler bug, nothing to pop on the VM stack")
    }

    #[expect(dead_code)]
    fn stack_top(&self) -> &Value {
        // optimization for ops that pop 1 value and push 1 value
        // allows mutation in place
        self.stack
            .last()
            .expect("compiler bug, nothing on top of the VM stack")
    }

    fn stack_top_mut(&mut self) -> &mut Value {
        // optimization for ops that pop 1 value and push 1 value
        // allows mutation in place
        self.stack
            .last_mut()
            .expect("compiler bug, nothing on top of the VM stack")
    }

    fn stack_peek(&self, distance: usize) -> &Value {
        let len = self.stack.len();
        self.stack
            .get(len - distance - 1)
            .expect("compiler bug, nothing to peek on the VM stack")
    }

    #[expect(dead_code)]
    fn stack_peek_mut(&mut self, distance: usize) -> &mut Value {
        let len = self.stack.len();
        self.stack
            .get_mut(len - distance - 1)
            .expect("compiler bug, nothing to peek on the VM stack")
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
