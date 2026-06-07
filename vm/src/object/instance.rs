use std::cell::RefCell;
use std::fmt::{self, Display, Formatter};

use intrusive_collections::UnsafeRef;
use lasso::Spur;
use smallvec::SmallVec;

use crate::{
    object::{Object, ObjectType, class::LoxClass},
    storage::{SymbolMap, WithStorage},
    value::Value,
};

/// Above this field count an instance switches from a linear-scan list to a hash
/// map. Lox instances almost always carry a handful of fields (a tree node has 4,
/// a typical object well under a dozen), and for those a contiguous list with a
/// `Spur`-equality scan beats hashing: it avoids the hash map's separate
/// control-byte/bucket allocation, its per-access hash + probe, and the rehash
/// churn as fields are added one at a time in `init`. The rare wide instance
/// (e.g. a 30-field record) spills to the hash map and keeps O(1) lookups.
const FIELD_SPILL_THRESHOLD: usize = 16;

/// Inline capacity of the small field list. The very common tiny instance (≤4
/// fields — a tree node, a pair) then stores its fields inline with *zero* heap
/// allocation; instances with 5–16 fields spill the `SmallVec` to one heap buffer
/// (still a linear scan). Now that `Value` is 8 bytes (NaN-boxed), an inline
/// `(Spur, Value)` is 16 bytes, so the inline budget here is 64 bytes.
const FIELD_INLINE: usize = 4;

/// An instance's field store: a linear-scanned `SmallVec` while small, a hash map
/// once it grows past [`FIELD_SPILL_THRESHOLD`].
#[derive(Debug)]
enum FieldMap {
    Small(SmallVec<[(Spur, Value); FIELD_INLINE]>),
    Large(SymbolMap<Value>),
}

impl FieldMap {
    fn new() -> Self {
        FieldMap::Small(SmallVec::new())
    }

    fn get(&self, key: Spur) -> Option<Value> {
        match self {
            FieldMap::Small(entries) => entries
                .iter()
                .find(|(k, _)| *k == key)
                .map(|(_, value)| *value),
            FieldMap::Large(map) => map.get(&key).cloned(),
        }
    }

    fn set(&mut self, key: Spur, value: Value) {
        match self {
            FieldMap::Small(entries) => {
                if let Some(slot) = entries.iter_mut().find(|(k, _)| *k == key) {
                    slot.1 = value;
                } else if entries.len() >= FIELD_SPILL_THRESHOLD {
                    // Spill to a hash map: move the existing entries over, then
                    // insert the new one.
                    let mut map: SymbolMap<Value> = entries.drain(..).collect();
                    map.insert(key, value);
                    *self = FieldMap::Large(map);
                } else {
                    entries.push((key, value));
                }
            }
            FieldMap::Large(map) => {
                map.insert(key, value);
            }
        }
    }

    fn for_each_value(&self, mut visit: impl FnMut(&Value)) {
        match self {
            FieldMap::Small(entries) => {
                for (_, value) in entries {
                    visit(value);
                }
            }
            FieldMap::Large(map) => {
                for value in map.values() {
                    visit(value);
                }
            }
        }
    }
}

/// An instance of a class. `fields` is a `RefCell` because property get/set
/// mutate it through a shared `UnsafeRef` (the stack value aliases the same
/// handle); the borrows are short and never re-entrant, so it can't panic.
#[repr(C)]
#[derive(Debug)]
pub struct LoxInstance {
    obj: Object,
    class: UnsafeRef<Object>,
    fields: RefCell<FieldMap>,
}

// SAFETY: `#[repr(C)]` with `Object` first; `Self::new` sets `obj.kind = Instance`.
unsafe impl ObjectType for LoxInstance {}

impl LoxInstance {
    pub fn new(class: UnsafeRef<Object>) -> Self {
        Self {
            obj: Object::instance(),
            class,
            fields: RefCell::new(FieldMap::new()),
        }
    }

    fn class(&self) -> &LoxClass {
        // SAFETY: an instance's class handle always points to a LoxClass.
        unsafe { self.class.downcast_ref::<LoxClass>() }
    }

    pub fn field(&self, name: Spur) -> Option<Value> {
        self.fields.borrow().get(name)
    }

    /// Look up a method on this instance's class (no field shadowing — callers
    /// check fields first).
    pub fn find_method(&self, name: Spur) -> Option<Value> {
        self.class().method(name)
    }

    // GC trace edges: the class handle and every field value.
    pub fn class_handle(&self) -> &UnsafeRef<Object> {
        &self.class
    }

    pub fn trace_fields(&self, visit: impl FnMut(&Value)) {
        self.fields.borrow().for_each_value(visit);
    }

    pub fn set_field(&self, name: Spur, value: Value) {
        self.fields.borrow_mut().set(name, value);
    }
}

impl Display for LoxInstance {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} instance", self.class())
    }
}

impl Display for WithStorage<'_, LoxInstance> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} instance", WithStorage(self.0.class(), self.1))
    }
}
