use std::fmt::{self, Display, Formatter};

use lasso::Spur;

use crate::{
    object::{Object, ObjectKind},
    storage::Storage,
};

#[repr(C)]
#[derive(Debug)]
pub struct InternalStr {
    _obj: Object,
    pub key: Spur,
}

impl InternalStr {
    pub fn boxed(key: Spur) -> Box<Self> {
        Box::new(Self {
            _obj: Object::internal_str(),
            key,
        })
    }

    pub fn as_str<'a>(&self, storage: &'a Storage) -> &'a str {
        storage.resolve(self.key)
    }
}

// SAFETY: `StringObj` is `#[repr(C)]` with `Object` (`obj`) as its first
// field, so an `Object` header at offset 0 is layout-compatible.
unsafe impl ObjectKind for InternalStr {}

impl PartialEq for InternalStr {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl Display for InternalStr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Symbol({})", self.key.into_inner())
    }
}
