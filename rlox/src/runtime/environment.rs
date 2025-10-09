use std::collections::HashMap;

use thiserror::Error;

use super::object::Object;

#[derive(Debug, Clone)]
pub struct Environment {
    stack: Vec<ScopedEnvironment>,
}

impl Environment {
    pub fn new() -> Self {
        // global scope
        let stack = vec![ScopedEnvironment::new()];
        Self { stack }
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

    pub fn get_at(&self, depth: usize, name: &str) -> Option<Object> {
        let stack_len = self.stack.len();
        self.stack
            .get(stack_len - depth - 1)
            .iter()
            .find_map(|s| s.get(name))
    }

    pub fn get(&self, name: &str) -> Option<Object> {
        self.stack.iter().rev().find_map(|s| s.get(name))
    }

    pub fn assign_at(
        &mut self,
        depth: usize,
        name: &str,
        object: Object,
    ) -> Result<Object, EnvError> {
        let stack_len = self.stack.len();
        self.stack
            .get_mut(stack_len - depth - 1)
            .iter_mut()
            .find_map(|s| s.assign(name, object.clone()))
            .ok_or_else(|| EnvError::UndefinedVar(name.into()))
    }

    pub fn assign(&mut self, name: &str, object: Object) -> Result<Object, EnvError> {
        self.stack
            .iter_mut()
            .rev()
            // PERF: garbage clone
            .find_map(|s| s.assign(name, object.clone()))
            .ok_or_else(|| EnvError::UndefinedVar(name.into()))
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
