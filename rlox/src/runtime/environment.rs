use std::{cell::RefMut, collections::HashMap};

use thiserror::Error;

use crate::runtime::chain::{Chain, SharedNode};

use super::object::Object;

#[derive(Debug, Clone)]
pub struct Environment {
    chain: Chain<ScopedEnvironment>,
    global: SharedNode<ScopedEnvironment>,
}

impl Environment {
    pub fn new() -> Self {
        let mut chain = Chain::new();
        // global scope
        chain.push(ScopedEnvironment::new());
        Self {
            // stack,
            global: chain.head_node().unwrap().clone(),
            chain,
        }
    }

    pub fn push_scope(&mut self) {
        self.chain.push(ScopedEnvironment::new());
    }

    pub fn pop_scope(&mut self) {
        self.chain.pop();
    }

    pub fn define(&mut self, name: Box<str>, object: Object) {
        self.current_scope().define(name, object)
    }

    pub fn define_global(&mut self, name: Box<str>, object: Object) {
        self.global_scope().define(name, object)
    }

    pub fn get_at(&self, depth: usize, name: &str) -> Option<Object> {
        self.chain
            .iter()
            .nth(depth)
            .iter()
            .find_map(|s| s.get(name))
    }

    pub fn get(&self, name: &str) -> Option<Object> {
        self.chain.iter().find_map(|s| s.get(name))
    }

    pub fn assign_at(
        &mut self,
        depth: usize,
        name: &str,
        object: Object,
    ) -> Result<Object, EnvError> {
        self.chain
            .iter()
            .nth(depth)
            .and_then(|mut s| s.assign(name, object.clone()))
            .ok_or_else(|| EnvError::UndefinedVar(name.into()))
    }

    pub fn assign(&mut self, name: &str, object: Object) -> Result<Object, EnvError> {
        self.chain
            .iter()
            // PERF: garbage clone
            .find_map(|mut s| s.assign(name, object.clone()))
            .ok_or_else(|| EnvError::UndefinedVar(name.into()))
    }

    fn current_scope(&mut self) -> RefMut<'_, ScopedEnvironment> {
        self.chain
            .head()
            .expect("must have at least the global scope")
    }

    fn global_scope(&mut self) -> RefMut<'_, ScopedEnvironment> {
        self.global.value()
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
