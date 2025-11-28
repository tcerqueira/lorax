use std::fmt::{Debug, Display};

use crate::{
    parsing::{
        ast::{AstArena, AstRef, StmtId},
        stmt::StmtFunction,
    },
    runtime::{
        Interpreter, control_flow::ControlFlow, environment::Environment, error::RuntimeError,
        object::Object,
    },
};

pub trait ObjCallable {
    fn arity(&self) -> u8;

    fn call(
        &self,
        interpreter: &mut Interpreter,
        ast_arena: &AstArena,
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
        _ast_arena: &AstArena,
        args: Vec<Object>,
    ) -> Result<Object, RuntimeError> {
        let mut interpreter = interpreter.new_env();
        (self.func)(&mut interpreter, args)
    }
}

#[derive(Clone)]
pub struct Function {
    decl: StmtId,
    name: Box<str>,
    arity: u8,
    enclosing_env: Environment,
}

impl Function {
    pub fn new(decl: AstRef<StmtFunction>, env: Environment) -> Self {
        let name = decl.name.as_str().into();
        let arity = decl.params.len().try_into().expect("arity always < 256");
        let decl = decl.id();
        Self {
            decl,
            name,
            arity,
            enclosing_env: env,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl ObjCallable for Function {
    fn arity(&self) -> u8 {
        self.arity
    }

    fn call(
        &self,
        interpreter: &mut Interpreter,
        arena: &AstArena,
        args: Vec<Object>,
    ) -> Result<Object, RuntimeError> {
        let callsite_env = interpreter.env.clone();
        interpreter.env = self.enclosing_env.clone();

        let call_result = {
            let mut interpreter = interpreter.new_env();
            let decl = arena.stmt_ref(self.decl).cast::<StmtFunction>();

            // recursive hack, redefine function inside function body
            // interpreter
            //     .env
            //     .define(self.name.clone(), Object::new(self.clone()));
            for (param, arg) in std::iter::zip(&decl.params, args) {
                interpreter.env.define(param.as_str().into(), arg);
            }
            match interpreter.execute_block(decl.body.iter().map(|&s| arena.stmt_ref(s))) {
                Ok(_) => Ok(Object::nil()),
                Err(ControlFlow::Return(object)) => Ok(object),
                Err(ControlFlow::Error(err)) => Err(err),
                Err(control_flow) => Err(RuntimeError::invalid_break_or_continue(
                    interpreter.current_span(),
                    control_flow,
                )),
            }
        };

        interpreter.env = callsite_env;
        call_result
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
        self.name == other.name
    }
}

impl PartialOrd for Function {
    fn partial_cmp(&self, _other: &Self) -> Option<std::cmp::Ordering> {
        None
    }
}
