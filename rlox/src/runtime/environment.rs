use std::collections::{HashMap, VecDeque};

use thiserror::Error;

use super::object::Object;

#[derive(Debug, Clone)]
pub struct Environment {
    stack: VecDeque<ScopedEnvironment>,
}

impl Environment {
    pub fn new() -> Self {
        let mut stack = VecDeque::new();
        // global scope
        stack.push_front(ScopedEnvironment::new());
        Self { stack }
    }

    pub fn push_scope(&mut self) {
        self.stack.push_front(ScopedEnvironment::new());
    }

    pub fn pop_scope(&mut self) {
        self.stack.pop_front();
    }

    pub fn define(&mut self, name: Box<str>, object: Object) {
        self.current_scope().define(name, object)
    }

    pub fn define_global(&mut self, name: Box<str>, object: Object) {
        self.global_scope().define(name, object)
    }

    pub fn get(&self, name: &str) -> Option<Object> {
        self.stack.iter().find_map(|s| s.get(name))
    }

    pub fn assign(&mut self, name: &str, object: Object) -> Result<Object, EnvError> {
        self.stack
            .iter_mut()
            // PERF: garbage clone
            .find_map(|s| s.assign(name, object.clone()))
            .ok_or_else(|| EnvError::UndefinedVar(name.into()))
    }

    fn current_scope(&mut self) -> &mut ScopedEnvironment {
        self.stack
            .front_mut()
            .expect("must have at least the global scope")
    }

    fn global_scope(&mut self) -> &mut ScopedEnvironment {
        self.stack
            .back_mut()
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
