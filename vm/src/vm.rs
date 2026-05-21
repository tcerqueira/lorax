use std::{
    collections::HashMap,
    io::{Cursor, Seek},
    mem::{self, transmute},
    ops::{Add, Div, Mul, Sub},
};

use lasso::Spur;
use report::error::RuntimeError;
use thiserror::Error;

use crate::{
    chunk::Chunk,
    debug::LineInfo,
    enconding::{DecodeError, OpDecoder},
    object::{ObjKind, internal_str::InternalStr, string::StringObj},
    opcode::{OpCode, OpDecodeError},
    storage::Storage,
    value::{Addr, Value, ValueError},
};

#[derive(Default)]
pub struct VirtualMachine {
    stack: Vec<Value>,
    storage: Storage,
    globals: HashMap<Spur, Value>,
    debug: bool,
}

#[derive(Debug, Error)]
pub enum VirtualMachineError {
    #[error(transparent)]
    Decode(#[from] DecodeError<OpDecodeError>),
    #[error(transparent)]
    Runtime(#[from] RuntimeError),
}

impl VirtualMachine {
    pub fn debug() -> Self {
        Self {
            debug: true,
            ..Default::default()
        }
    }

    pub fn run(&mut self, chunk: Chunk) -> Result<(), VirtualMachineError> {
        let mut pc = Cursor::new(chunk.code.as_slice());
        while let Some(op) = pc.decode_op::<OpCode>()? {
            self.trace(op);

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
                        && o1.is_str()
                        && o2.is_str() =>
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
                OpCode::Equal => self.equal(),
                OpCode::Greater => self
                    .binary_op(Value::greater)
                    .map_err(invalid_operand_err)?,
                OpCode::Less => self.binary_op(Value::less).map_err(invalid_operand_err)?,
                OpCode::Print => {
                    let v = self.stack_pop();
                    match v {
                        Value::Object(v) if v.is_str() => println!("{}", v.as_str(&self.storage)),
                        v => println!("{v}"),
                    };
                }
                OpCode::Pop => _ = self.stack_pop(),
                OpCode::DefineGlobal(addr) => {
                    // Keep the value on the stack as a GC root until after the insert returns
                    let key = self.variable_name(&chunk, addr).key;
                    let value = self.stack_peek(0).clone();
                    self.globals.insert(key, value);
                    self.stack_pop();
                }
                OpCode::GetGlobal(_) => todo!(),
                OpCode::SetGlobal(_) => todo!(),
            }
        }
        Ok(())
    }

    pub fn storage(&mut self) -> &mut Storage {
        &mut self.storage
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

    fn equal(&mut self) {
        let b = self.stack_pop();
        let a = self.stack_pop();
        if mem::discriminant(&a) != mem::discriminant(&b) {
            return self.stack_push(Value::boolean(false));
        }

        let res = match (a, b) {
            (Value::Object(a), Value::Object(b)) => match (a.kind(), b.kind()) {
                // Safety: checked kind before casting.
                (ObjKind::InternalStr, ObjKind::InternalStr) => unsafe {
                    a.downcast_ref::<InternalStr>() == b.downcast_ref::<InternalStr>()
                },
                (s1, s2) if a.is_str() && b.is_str() => {
                    let a = a.as_str(&self.storage);
                    let b = b.as_str(&self.storage);
                    a == b
                }
                _ => unreachable!("missing branch on equal"),
            },
            (a, b) => a == b,
        };
        self.stack_push(Value::boolean(res));
    }

    fn concatenate_str(&mut self) {
        // Build the joined string while the operands stay on the stack as GC roots.
        let s = {
            let (Value::Object(a), Value::Object(b)) = (self.stack_peek(1), self.stack_peek(0))
            else {
                unreachable!("just matched Object in branch");
            };
            // PERF: create constructor that adds in place, reduces one allocation
            let mut s = a.as_str(&self.storage).to_owned();
            s.push_str(b.as_str(&self.storage));
            s
        };
        let obj = self.storage.heap.add(StringObj::boxed(&s));
        self.stack_pop();
        self.stack_pop();
        self.stack_push(Value::Object(obj));
    }

    fn variable_name<'a>(&'a self, chunk: &Chunk, addr: Addr) -> &'a InternalStr {
        let Value::Object(name) = chunk.constant(addr) else {
            panic!("could not get variable name: value is not an object of string type")
        };
        assert_eq!(
            name.kind(),
            ObjKind::InternalStr,
            "could not get variable name: value is not an object of string type"
        );
        // SAFETY: all UnsafeRef objects are bound by Storage which has the same lifetime as &self
        unsafe {
            let name = name.downcast_ref::<InternalStr>();
            transmute::<_, &'a InternalStr>(name)
        }
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

    fn trace(&self, op: OpCode) {
        if !self.debug {
            return;
        }
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
