use std::{
    collections::{HashMap, hash_map::Entry},
    io::{Cursor, Seek},
    mem::{self, transmute},
    ops::{Add, Div, Mul, Sub},
};

use lasso::Spur;
use report::error::RuntimeError;

use crate::{
    chunk::Chunk,
    debug::LineInfo,
    enconding::OpDecoder,
    object::{ObjKind, internal_str::InternalStr, string::StringObj},
    opcode::OpCode,
    storage::Storage,
    value::{Addr, Value, ValueError},
    vm::{error::VirtualMachineError, stack::Stack},
};

pub mod error;
pub mod stack;

#[derive(Default)]
pub struct VirtualMachine {
    stack: Stack,
    storage: Storage,
    globals: HashMap<Spur, Value>,
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

    pub fn run(&mut self, chunk: Chunk) -> Result<(), VirtualMachineError> {
        let mut pc = Cursor::new(chunk.code.as_slice());
        while let Some(op) = pc.decode_op::<OpCode>()? {
            self.trace(op);

            let mut make_span = || {
                chunk
                    .get_line(pc.stream_position().unwrap() - 1)
                    .map(LineInfo::to_span)
                    .unwrap_or_default()
            };

            let invalid_operand_err =
                |_: ValueError| RuntimeError::custom(make_span(), "invalid operand");

            match op {
                OpCode::NoOp => {}
                OpCode::Return => {
                    return Ok(());
                }
                OpCode::Constant(addr) => {
                    let constant = chunk.constant(addr);
                    self.stack.push(constant.clone());
                }
                OpCode::Neg => {
                    let v = self.stack.top_mut();
                    *v = (-v.clone()).map_err(invalid_operand_err)?;
                }
                OpCode::Add
                    if let (Value::Object(o1), Value::Object(o2)) =
                        (self.stack.peek(0), self.stack.peek(1))
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
                    .map_err(invalid_operand_err)?,
                OpCode::Less => self.binary_op(Value::less).map_err(invalid_operand_err)?,
                OpCode::Print => {
                    let v = self.stack.pop();
                    match v {
                        Value::Object(v) if v.is_str() => println!("{}", v.as_str(&self.storage)),
                        v => println!("{v}"),
                    };
                }
                OpCode::Pop => _ = self.stack.pop(),
                OpCode::DefineGlobal(addr) => {
                    self.with_variable(&chunk, addr, |vm, key, value| {
                        vm.globals.insert(key, value);
                    });
                    self.stack.pop();
                }
                OpCode::GetGlobal(addr) => {
                    let key = self.variable_name(&chunk, addr).key;
                    match self.globals.get(&key) {
                        Some(value) => {
                            let value = value.clone();
                            self.stack.push(value);
                        }
                        None => return Err(RuntimeError::undefined(make_span()).into()),
                    }
                }
                OpCode::SetGlobal(addr) => {
                    self.with_variable(&chunk, addr, |vm, key, value| {
                        #[allow(clippy::unit_arg)]
                        match vm.globals.entry(key) {
                            Entry::Occupied(mut e) => Ok(*e.get_mut() = value),
                            Entry::Vacant(_) => Err(RuntimeError::undefined(make_span())),
                        }
                    })?;
                }
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
        if mem::discriminant(&a) != mem::discriminant(&b) {
            return self.stack.push(Value::boolean(false));
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
        self.stack.push(Value::boolean(res));
    }

    fn concatenate_str(&mut self) {
        // Build the joined string while the operands stay on the stack as GC roots.
        let s = {
            let (Value::Object(a), Value::Object(b)) = (self.stack.peek(1), self.stack.peek(0))
            else {
                unreachable!("just matched Object in branch");
            };
            // PERF: create constructor that adds in place, reduces one allocation
            let mut s = a.as_str(&self.storage).to_owned();
            s.push_str(b.as_str(&self.storage));
            s
        };
        let obj = self.storage.add_obj(StringObj::boxed(&s));
        self.stack.pop();
        self.stack.pop();
        self.stack.push(Value::Object(obj));
    }

    fn with_variable<T>(
        &mut self,
        chunk: &Chunk,
        addr: Addr,
        f: impl FnOnce(&mut VirtualMachine, Spur, Value) -> T,
    ) -> T {
        // Value stays on the stack across `f` so it remains a GC root if a
        // future collector triggers during a globals rehash.
        let key = self.variable_name(chunk, addr).key;
        let value = self.stack.top().clone();
        f(self, key, value)
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
