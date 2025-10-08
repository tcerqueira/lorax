use std::collections::HashMap;

use thiserror::Error;

use crate::parsing::{
    ast::{AstRef, ExprId},
    expr::{ExprAssign, ExprVariable},
};

use super::object::Object;

#[derive(Debug, Clone)]
pub struct Environment {
    stack: Vec<ScopedEnvironment>,
    var_resolution: HashMap<ExprId, usize>,
}

impl Environment {
    pub fn new() -> Self {
        // global scope
        let stack = vec![ScopedEnvironment::new()];
        Self {
            stack,
            var_resolution: HashMap::new(),
        }
    }

    pub fn push_scope(&mut self) {
        self.stack.push(ScopedEnvironment::new());
    }

    pub fn pop_scope(&mut self) {
        self.stack.pop();
    }

    pub fn define(&mut self, name: Box<str>, object: Object) {
        self.current_scope().define(name, object)
    }

    pub fn define_global(&mut self, name: Box<str>, object: Object) {
        self.global_scope().define(name, object)
    }

    pub fn get(&self, var: AstRef<ExprVariable>) -> Option<Object> {
        let stack_len = self.stack.len();
        self.var_resolution.get(&var.id()).and_then(|&depth| {
            // FIXME: recursion kinda broken, depth is not known until runtime
            self.stack[..=stack_len - depth - 1]
                .iter()
                .rev()
                .find_map(|s| s.get(&var.name.as_str()))
        })
    }

    pub fn assign(&mut self, var: AstRef<ExprAssign>, object: Object) -> Result<Object, EnvError> {
        let name = var.name.as_str();
        let stack_len = self.stack.len();
        self.var_resolution
            .get(&var.id())
            .and_then(|&depth| {
                // FIXME: recursion kinda broken, depth is not known until runtime
                self.stack[..=stack_len - depth - 1]
                    .iter_mut()
                    .rev()
                    .find_map(|s| s.assign(&name, object.clone()))
            })
            .ok_or_else(|| EnvError::UndefinedVar(name.into()))
    }

    pub fn resolve_var(&mut self, expr_id: ExprId, depth: usize) {
        self.var_resolution.insert(expr_id, depth);
    }

    fn current_scope(&mut self) -> &mut ScopedEnvironment {
        self.stack
            .last_mut()
            .expect("must have at least the global scope")
    }

    fn global_scope(&mut self) -> &mut ScopedEnvironment {
        self.stack
            .first_mut()
            .expect("must have at least the global scope")
    }
}

#[derive(Debug, Clone)]
struct ScopedEnvironment {
    values: HashMap<Box<str>, Object>,
}

impl ScopedEnvironment {
    pub fn new() -> Self {
        Self {
            values: HashMap::default(),
        }
    }

    pub fn define(&mut self, name: Box<str>, object: Object) {
        self.values.insert(name, object);
    }

    pub fn get(&self, name: &str) -> Option<Object> {
        self.values.get(name).cloned()
    }

    pub fn assign(&mut self, name: &str, object: Object) -> Option<Object> {
        self.values
            .get_mut(name)
            .map(|o| *o = Object::clone(&object))
            .map(|_| object)
    }
}

#[derive(Debug, Error)]
pub enum EnvError {
    #[error("Undefined variable '{0}'.")]
    UndefinedVar(Box<str>),
}
