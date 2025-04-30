use std::collections::HashMap;

use crate::parser::object::Object;

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
}
