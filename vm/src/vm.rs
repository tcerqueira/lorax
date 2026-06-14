use std::{
    collections::hash_map::Entry,
    fmt::Display,
    io::Cursor,
    mem,
    ops::{Add, Div, Mul, Sub},
};

use anyhow::Context;
use lasso::Spur;
use report::{Span, error::RuntimeError};

use crate::{
    chunk::Chunk,
    debug::LineInfo,
    enconding::{Addr, LocalSlot, OpCode, OpDecoder},
    object::{Object, string::LoxString},
    storage::{Storage, SymbolMap, WithStorage},
    value::{Value, ValueError},
    vm::{error::VirtualMachineError, frame::CallFrame, stack::Stack},
};

pub mod error;
pub mod frame;
pub mod stack;

#[derive(Default)]
pub struct VirtualMachine {
    stack: Stack,
    storage: Storage,
    globals: SymbolMap<Value>,
    frames: Vec<CallFrame>,
    debug: bool,
}

impl VirtualMachine {
    pub fn debug() -> Self {
        Self {
            debug: true,
            ..Default::default()
        }
    }

    pub fn storage(&mut self) -> &mut Storage {
        &mut self.storage
    }

    fn frame(&self) -> &CallFrame {
        self.frames.last().expect("always has top level call frame")
    }

    fn frame_mut(&mut self) -> &mut CallFrame {
        self.frames
            .last_mut()
            .expect("always has top level call frame")
    }

    fn chunk(&self) -> &Chunk {
        self.frame().chunk()
    }

    fn pc(&mut self) -> &mut Cursor<Chunk> {
        &mut self.frame_mut().pc
    }

    fn make_span(&self) -> Span {
        let pos = self.frame().pc.position().saturating_sub(1);
        self.frame()
            .chunk()
            .get_line(pos)
            .map(LineInfo::to_span)
            .unwrap_or_default()
    }

    fn local(&self, slot: LocalSlot) -> &Value {
        let start = self.frame().stack_start;
        self.stack.get(start + slot.0 as usize)
    }

    fn local_mut(&mut self, slot: LocalSlot) -> &mut Value {
        let start = self.frame().stack_start;
        self.stack.get_mut(start + slot.0 as usize)
    }

    fn runtime_err(&self, message: impl Display) -> RuntimeError {
        RuntimeError::custom(self.make_span(), message)
    }

    pub fn run(&mut self, chunk: Chunk) -> Result<(), VirtualMachineError> {
        // top level call frame
        self.frames.push(CallFrame::new(chunk, 0));

        while let Some(op) = self.pc().decode_op::<OpCode>()? {
            self.trace(op);

            match op {
                OpCode::NoOp => {}
                OpCode::Ret => {
                    return Ok(());
                }
                OpCode::Constant(addr) => {
                    let constant = self.chunk().constant(addr);
                    self.stack.push(constant.clone());
                }
                OpCode::Neg => {
                    let v = self.stack.top_mut();
                    match -v.clone() {
                        Ok(res) => *v = res,
                        Err(_) => return Err(self.runtime_err("invalid operand").into()),
                    }
                }
                OpCode::Add if self.stack.peek(0).is_str() && self.stack.peek(1).is_str() => {
                    self.concatenate_str()
                }
                OpCode::Add => self
                    .binary_op(Value::add)
                    .map_err(|_| self.runtime_err("invalid operand"))?,
                OpCode::Sub => self
                    .binary_op(Value::sub)
                    .map_err(|_| self.runtime_err("invalid operand"))?,
                OpCode::Mul => self
                    .binary_op(Value::mul)
                    .map_err(|_| self.runtime_err("invalid operand"))?,
                OpCode::Div => self
                    .binary_op(Value::div)
                    .map_err(|_| self.runtime_err("invalid operand"))?,
                OpCode::True => {
                    self.stack.push(Value::boolean(true));
                }
                OpCode::False => {
                    self.stack.push(Value::boolean(false));
                }
                OpCode::Nil => {
                    self.stack.push(Value::nil());
                }
                OpCode::Not => {
                    let v = self.stack.top_mut();
                    *v = Value::Boolean(v.is_falsey());
                }
                OpCode::Equal => self.equal(),
                OpCode::Greater => self
                    .binary_op(Value::greater)
                    .map_err(|_| self.runtime_err("invalid operand"))?,
                OpCode::Less => self
                    .binary_op(Value::less)
                    .map_err(|_| self.runtime_err("invalid operand"))?,
                OpCode::Print => {
                    let v = self.stack.pop();
                    println!("{}", WithStorage(&v, self.storage()));
                }
                OpCode::Pop => _ = self.stack.pop(),
                OpCode::PopN(n) => self.stack.pop_n(n),
                OpCode::DefGlobal(addr) => {
                    self.with_variable(addr, |vm, key, value| {
                        vm.globals.insert(key, value);
                    });
                    self.stack.pop();
                }
                OpCode::GetGlobal(addr) => {
                    let key = self.variable_name(self.chunk(), addr);
                    match self.globals.get(&key) {
                        Some(value) => {
                            let value = value.clone();
                            self.stack.push(value);
                        }
                        None => return Err(RuntimeError::undefined(self.make_span()).into()),
                    }
                }
                OpCode::SetGlobal(addr) => {
                    self.with_variable(addr, |vm, key, value| {
                        #[allow(clippy::unit_arg)]
                        match vm.globals.entry(key) {
                            Entry::Occupied(mut e) => Ok(*e.get_mut() = value),
                            Entry::Vacant(_) => Err(RuntimeError::undefined(vm.make_span())),
                        }
                    })?;
                }
                OpCode::GetLocal(slot) => {
                    let v = self.local(slot).clone();
                    self.stack.push(v);
                }
                OpCode::SetLocal(slot) => {
                    // Assignment is an expression — leave the value on top so
                    // chained uses like `print a = 1;` work.
                    let v = self.stack.top().clone();
                    *self.local_mut(slot) = v;
                }
                OpCode::JmpIfFalse(offset) => {
                    let condition = self.stack.top().is_falsey();
                    if condition {
                        self.pc()
                            .relative_jump(offset as i64)
                            .with_context(|| "could not jump to offset {offset} {e}")?;
                    }
                }
                OpCode::Jmp(offset) => self
                    .pc()
                    .relative_jump(offset as i64)
                    .with_context(|| format!("could not jump to offset {offset}"))?,
                OpCode::Loop(offset) => self
                    .pc()
                    .relative_jump(-(offset as i64))
                    .with_context(|| format!("could not loop to offset {}", -(offset as i64)))?,
            }
        }
        Ok(())
    }

    fn binary_op<F>(&mut self, op: F) -> Result<(), ValueError>
    where
        F: Fn(Value, Value) -> Result<Value, ValueError>,
    {
        let b = self.stack.pop();
        let a = self.stack.pop();
        let res = op(a, b)?;
        self.stack.push(res);
        Ok(())
    }

    fn equal(&mut self) {
        let b = self.stack.pop();
        let a = self.stack.pop();
        let res = match (&a, &b) {
            (Value::Symbol(x), Value::Symbol(y)) => x == y,
            _ if a.is_str() && b.is_str() => a.as_str(&self.storage) == b.as_str(&self.storage),
            _ if mem::discriminant(&a) != mem::discriminant(&b) => false,
            (Value::Object(a), Value::Object(b)) => Object::eq(a, b),
            _ => a == b,
        };
        self.stack.push(Value::boolean(res));
    }

    fn concatenate_str(&mut self) {
        // Build the joined string while the operands stay on the stack as GC roots.
        let s = {
            let a = self.stack.peek(1).as_str(&self.storage);
            let b = self.stack.peek(0).as_str(&self.storage);
            // PERF: create constructor that adds in place, reduces one allocation
            let mut s = a.to_owned();
            s.push_str(b);
            s
        };
        let obj = self.storage.add_obj(LoxString::boxed(&s));
        self.stack.pop();
        self.stack.pop();
        self.stack.push(Value::Object(obj));
    }

    fn with_variable<T>(
        &mut self,
        addr: Addr,
        f: impl FnOnce(&mut VirtualMachine, Spur, Value) -> T,
    ) -> T {
        // Value stays on the stack across `f` so it remains a GC root if a
        // future collector triggers during a globals rehash.
        let key = self.variable_name(self.frame().chunk(), addr);
        let value = self.stack.top().clone();
        f(self, key, value)
    }

    fn variable_name(&self, chunk: &Chunk, addr: Addr) -> Spur {
        let Value::Symbol(key) = chunk.constant(addr) else {
            panic!("could not get variable name: constant slot is not a Symbol")
        };
        *key
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
