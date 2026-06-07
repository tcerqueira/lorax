use std::cell::RefCell;
use std::fmt::{self, Display, Formatter};

use lasso::Spur;

use crate::{
    object::{Object, ObjectType},
    storage::{SymbolMap, WithStorage},
    value::Value,
};

/// A class value: a name plus a table of methods (each a `LoxClosure` value).
/// `methods` is a `RefCell` because `OP_METHOD`/`OP_INHERIT` mutate it through a
/// shared `UnsafeRef` after the class is already heap-reachable.
#[repr(C)]
#[derive(Debug)]
pub struct LoxClass {
    obj: Object,
    name: Spur,
    methods: RefCell<SymbolMap<Value>>,
}

// SAFETY: `#[repr(C)]` with `Object` first; `Self::new` sets `obj.kind = Class`.
unsafe impl ObjectType for LoxClass {}

impl LoxClass {
    pub fn new(name: Spur) -> Self {
        Self {
            obj: Object::class(),
            name,
            methods: RefCell::new(SymbolMap::default()),
        }
    }

    pub fn name(&self) -> Spur {
        self.name
    }

    pub fn define_method(&self, name: Spur, method: Value) {
        self.methods.borrow_mut().insert(name, method);
    }

    pub fn method(&self, name: Spur) -> Option<Value> {
        self.methods.borrow().get(&name).cloned()
    }

    /// GC trace edge: every method value.
    pub fn trace_methods(&self, mut visit: impl FnMut(&Value)) {
        for method in self.methods.borrow().values() {
            visit(method);
        }
    }

    /// Copy-down inheritance: snapshot `other`'s methods into this subclass.
    /// The subclass is freshly created (its table is empty at `OP_INHERIT`), so a
    /// straight replace inherits everything and its own later methods override.
    /// Clones into a local first so the two tables are never borrowed at once.
    pub fn copy_methods_from(&self, other: &LoxClass) {
        let inherited = other.methods.borrow().clone();
        *self.methods.borrow_mut() = inherited;
    }
}

impl Display for LoxClass {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Symbol({})", self.name.into_inner())
    }
}

impl Display for WithStorage<'_, LoxClass> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.1.resolve(self.0.name))
    }
}
