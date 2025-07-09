use std::fmt::{Debug, Display};

use crate::{
    parsing::stmt::StmtFunction,
    runtime::{Interpreter, error::RuntimeError, object::Object},
};

pub trait ObjCallable {
    fn arity(&self) -> u8;

    fn call(
        &self,
        interpreter: &mut Interpreter,
        args: Vec<Object>,
    ) -> Result<Object, RuntimeError>;
}

pub type CallableFn = Box<dyn Fn(&mut Interpreter, Vec<Object>) -> Result<Object, RuntimeError>>;

pub struct NativeFunction {
    name: &'static str,
    arity: u8,
    func: CallableFn,
}

impl NativeFunction {
    pub fn new(
        name: &'static str,
        arity: u8,
        f: impl Fn(&mut Interpreter, Vec<Object>) -> Result<Object, RuntimeError> + 'static,
    ) -> Self {
        Self {
            name,
            arity,
            func: Box::new(f),
        }
    }
}

impl ObjCallable for NativeFunction {
    fn arity(&self) -> u8 {
        self.arity
    }

    fn call(
        &self,
        interpreter: &mut Interpreter,
        args: Vec<Object>,
    ) -> Result<Object, RuntimeError> {
        (self.func)(interpreter, args)
    }
}

pub struct Function {
    decl: StmtFunction,
}

impl Function {
    pub fn new(decl: StmtFunction) -> Self {
        Self { decl }
    }

    pub fn name(&self) -> &str {
        self.decl.name.ty().ident()
    }
}

impl ObjCallable for Function {
    fn arity(&self) -> u8 {
        self.decl
            .params
            .len()
            .try_into()
            .expect("arity always < 256")
    }

    fn call(
        &self,
        interpreter: &mut Interpreter,
        args: Vec<Object>,
    ) -> Result<Object, RuntimeError> {
        let mut interpreter = interpreter.new_env();
        std::iter::zip(&self.decl.params, args)
            .for_each(|(param, arg)| interpreter.env.define(param.ty().ident().into(), arg));
        interpreter.execute_block(&self.decl.body)?;
        Ok(Object::nil())
    }
}

impl Debug for NativeFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeFunction")
            .field("name", &self.name)
            .field("arity", &self.arity)
            .finish_non_exhaustive()
    }
}

impl Display for NativeFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<native fn {}>", self.name)
    }
}

impl PartialEq for NativeFunction {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.func.as_ref(), other.func.as_ref())
    }
}

impl PartialOrd for NativeFunction {
    fn partial_cmp(&self, _other: &Self) -> Option<std::cmp::Ordering> {
        None
    }
}

impl Debug for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Function")
            .field("name", &self.name())
            .field("arity", &self.arity())
            .finish_non_exhaustive()
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn {}>", self.name())
    }
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        self.decl.name == other.decl.name
    }
}

impl PartialOrd for Function {
    fn partial_cmp(&self, _other: &Self) -> Option<std::cmp::Ordering> {
        None
    }
}
