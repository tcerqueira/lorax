use std::fmt::{self, Display, Formatter};

use lasso::{Rodeo, Spur};

use crate::object::{Object, ObjectKind};

#[repr(C)]
#[derive(Debug)]
pub struct InternalStr {
    _obj: Object,
    key: Spur,
}

impl InternalStr {
    pub fn new(strings: &mut Rodeo, s: &str) -> Self {
        let key = strings.get_or_intern(s);
        Self {
            _obj: Object::internal_str(),
            key,
        }
    }

    pub fn as_str<'a>(&self, strings: &'a Rodeo) -> &'a str {
        strings.resolve(&self.key)
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
