use std::collections::{HashMap, VecDeque};

use thiserror::Error;

use super::object::Object;

pub struct Environment {
    chain: VecDeque<ScopedEnvironment>,
}

impl Environment {
    pub fn new() -> Self {
        let mut chain = VecDeque::new();
        // global scope
        chain.push_front(ScopedEnvironment::new());
        Self { chain }
    }

    pub fn push_scope(&mut self) {
        self.chain.push_front(ScopedEnvironment::new());
    }

    pub fn pop_scope(&mut self) {
        self.chain.pop_front();
    }

    pub fn define(&mut self, name: Box<str>, object: Object) {
        self.current_scope().define(name, object)
    }

    pub fn get(&self, name: &str) -> Option<Object> {
        self.chain.iter().find_map(|s| s.get(name))
    }

    pub fn assign(&mut self, name: &str, object: Object) -> Result<Object, EnvError> {
        self.chain
            .iter_mut()
            // PERF: garbage clone
            .find_map(|s| s.assign(name, object.clone()))
            .ok_or_else(|| EnvError::UndefinedVar(name.into()))
    }

    fn current_scope(&mut self) -> &mut ScopedEnvironment {
        self.chain
            .get_mut(0)
            .expect("must have at least the global scope")
    }
}

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
