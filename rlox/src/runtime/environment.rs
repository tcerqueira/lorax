use std::collections::HashMap;

use thiserror::Error;

use super::object::Object;

pub struct Environment {
    globals: HashMap<Box<str>, Object>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            globals: HashMap::default(),
        }
    }

    pub fn define(&mut self, name: Box<str>, object: Object) {
        self.globals.insert(name, object);
    }

    pub fn get(&mut self, name: &str) -> Option<Object> {
        self.globals.get(name).cloned()
    }

    pub fn assign(&mut self, name: &str, object: Object) -> Result<Object, EnvError> {
        self.globals
            .get_mut(name)
            .map(|o| *o = Object::clone(&object))
            .map(|_| object)
            .ok_or_else(|| EnvError::UndefinedVar(name.into()))
    }
}

#[derive(Debug, Error)]
pub enum EnvError {
    #[error("Undefined variable '{0}'.")]
    UndefinedVar(Box<str>),
}
