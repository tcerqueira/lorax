use std::{
    cmp::Ordering,
    fmt::{self, Debug, Display},
    mem,
    ops::{Add, Div, Mul, Neg, Sub},
    ptr,
};

use intrusive_collections::UnsafeRef;
use lasso::{Key, Spur};

use crate::{
    object::{ObjKind, Object},
    storage::{Storage, WithStorage},
};

pub struct ValueError;

/// A NaN-boxed Lox value packed into a single `u64` (8 bytes, `Copy`, register-
/// sized — half the footprint of the former tagged enum). A non-NaN `f64` *is* a
/// number, stored as its bits with zero overhead; every other kind hides in the
/// payload of a quiet NaN, distinguished by a tag:
///
/// * **Number** — any bit pattern that is not the quiet-NaN tag (`f64::from_bits`).
///   Lox-produced NaNs are `0x7ff8…` (bit 50 clear), so they read back as numbers.
/// * **Nil / False / True** — `QNAN | {1, 2, 3}` (sign 0, bit 48 clear).
/// * **Symbol** — `QNAN | SYMBOL_TAG | spur`; `Spur`'s index fits the low 32 bits,
///   well clear of `SYMBOL_TAG` at bit 48.
/// * **Object** — `SIGN | QNAN | addr`; the heap pointer lives in the low 48 bits
///   (a userspace-pointer assumption, `debug_assert`ed), stored and recovered
///   through the *exposed-provenance* API so the int↔ptr round-trip is sound.
///
/// `Value` owns nothing — an `Object` payload is a non-owning handle into the GC
/// heap, exactly like the former `UnsafeRef`-in-enum representation.
#[derive(Clone, Copy)]
pub struct Value(u64);

/// Quiet-NaN tag: exponent all ones (bits 62–52) plus the top two mantissa bits
/// (bits 51, 50). A value is a number iff these bits are *not* all set.
const QNAN: u64 = 0x7ffc_0000_0000_0000;
const SIGN: u64 = 0x8000_0000_0000_0000;
/// Marks a `Symbol` — bit 48, free immediately below `QNAN`.
const SYMBOL_TAG: u64 = 1 << 48;
const TAG_NIL: u64 = QNAN | 1;
const TAG_FALSE: u64 = QNAN | 2;
const TAG_TRUE: u64 = QNAN | 3;
/// Low 48 bits: an object pointer, or (in its low 32) a `Spur` index.
const PAYLOAD_MASK: u64 = 0x0000_ffff_ffff_ffff;
const SPUR_MASK: u64 = 0x0000_0000_ffff_ffff;

/// A decoded view of a [`Value`] for exhaustive matching off the hot path. The
/// hot VM paths use the direct accessors ([`Value::as_number`] etc.) instead.
#[derive(Debug, Clone)]
pub enum ValueView {
    Nil,
    Boolean(bool),
    Number(f64),
    Symbol(Spur),
    Object(UnsafeRef<Object>),
}

impl Value {
    pub fn nil() -> Self {
        Value(TAG_NIL)
    }

    pub fn boolean(value: bool) -> Self {
        Value(if value { TAG_TRUE } else { TAG_FALSE })
    }

    pub fn number(value: f64) -> Self {
        Value(value.to_bits())
    }

    pub fn symbol(key: Spur) -> Self {
        Value(QNAN | SYMBOL_TAG | (key.into_usize() as u64 & SPUR_MASK))
    }

    pub fn object(value: UnsafeRef<Object>) -> Self {
        let ptr = UnsafeRef::into_raw(value);
        let addr = ptr.expose_provenance() as u64;
        // Unconditional (not `debug_assert`): a >48-bit pointer would be silently
        // truncated by `PAYLOAD_MASK` and read back as a wrong address. The check
        // is at object *construction* (already an allocation), not the hot read
        // path, so the cost is negligible — and it turns a future port to a
        // wider-VA target (LA57, AArch64 pointer tagging) into a clean abort
        // rather than memory corruption.
        assert!(
            addr & !PAYLOAD_MASK == 0,
            "object pointer exceeds 48 bits; NaN-boxing assumes a 48-bit address space"
        );
        Value(SIGN | QNAN | (addr & PAYLOAD_MASK))
    }

    // --- classifiers -------------------------------------------------------

    pub fn is_number(&self) -> bool {
        (self.0 & QNAN) != QNAN
    }

    pub fn is_nil(&self) -> bool {
        self.0 == TAG_NIL
    }

    pub fn is_boolean(&self) -> bool {
        self.0 == TAG_TRUE || self.0 == TAG_FALSE
    }

    pub fn is_symbol(&self) -> bool {
        (self.0 & (SIGN | QNAN | SYMBOL_TAG)) == (QNAN | SYMBOL_TAG)
    }

    pub fn is_object(&self) -> bool {
        (self.0 & (SIGN | QNAN)) == (SIGN | QNAN)
    }

    pub fn is_falsey(&self) -> bool {
        // `nil` and `false` are falsey; every number (incl. 0), symbol, object,
        // and `true` is truthy.
        self.0 == TAG_NIL || self.0 == TAG_FALSE
    }

    pub fn is_str(&self) -> bool {
        self.is_symbol() || matches!(self.as_object(), Some(o) if o.kind() == ObjKind::String)
    }

    pub fn is_instance(&self) -> bool {
        matches!(self.as_object(), Some(o) if o.kind() == ObjKind::Instance)
    }

    // --- extractors --------------------------------------------------------

    pub fn as_number(&self) -> Option<f64> {
        self.is_number().then(|| f64::from_bits(self.0))
    }

    pub fn as_boolean(&self) -> Option<bool> {
        match self.0 {
            TAG_TRUE => Some(true),
            TAG_FALSE => Some(false),
            _ => None,
        }
    }

    pub fn as_symbol(&self) -> Option<Spur> {
        self.is_symbol()
            .then(|| Spur::try_from_usize((self.0 & SPUR_MASK) as usize).expect("valid spur"))
    }

    pub fn as_object(&self) -> Option<UnsafeRef<Object>> {
        self.is_object().then(|| {
            let ptr = ptr::with_exposed_provenance::<Object>((self.0 & PAYLOAD_MASK) as usize);
            // SAFETY: the address was produced from a live object handle in
            // `object()` (its provenance exposed there); the value keeps the
            // object rooted, so the pointer is valid. `UnsafeRef` is non-owning,
            // so materializing one here does not affect ownership.
            unsafe { UnsafeRef::from_raw(ptr) }
        })
    }

    /// Decode into an owned [`ValueView`] for exhaustive matching.
    pub fn view(&self) -> ValueView {
        if let Some(n) = self.as_number() {
            ValueView::Number(n)
        } else if self.is_nil() {
            ValueView::Nil
        } else if let Some(b) = self.as_boolean() {
            ValueView::Boolean(b)
        } else if let Some(key) = self.as_symbol() {
            ValueView::Symbol(key)
        } else {
            ValueView::Object(self.as_object().expect("value is an object"))
        }
    }

    pub fn as_str<'s>(&'s self, storage: &'s Storage) -> &'s str {
        if let Some(key) = self.as_symbol() {
            storage.resolve(key)
        } else if let Some(o) = self.as_object() {
            if o.kind() == ObjKind::String {
                let s = o.as_str();
                // SAFETY: `s` points into the `LoxString` buffer, which lives at a
                // stable GC-heap address for the object's whole life. The object
                // outlives `'s` *structurally* (a `u64` `Value` roots nothing):
                // the GC runs only at the VM's dispatch safe point, `add_obj`
                // never collects, and no opcode handler reaches a safe point while
                // a borrow obtained here is live — so the buffer cannot be freed
                // before `'s` ends. We only widen the borrow from the temporary
                // handle `o` to `'s`; no data is moved.
                return unsafe { mem::transmute::<&str, &'s str>(s) };
            }
            panic!("Value::as_str called on non-string object")
        } else {
            panic!("Value::as_str called on non-string {self:?}")
        }
    }

    pub fn greater(self, other: Self) -> Result<Self, ValueError> {
        Self::partial_cmp(&self, &other)
            .map(|ord| Self::boolean(ord == Ordering::Greater))
            .ok_or(ValueError)
    }

    pub fn less(self, other: Self) -> Result<Self, ValueError> {
        Self::partial_cmp(&self, &other)
            .map(|ord| Self::boolean(ord == Ordering::Less))
            .ok_or(ValueError)
    }
}

impl Neg for Value {
    type Output = Result<Self, ValueError>;

    fn neg(self) -> Self::Output {
        self.as_number().map(|n| Self::number(-n)).ok_or(ValueError)
    }
}

impl Add for Value {
    type Output = Result<Self, ValueError>;

    fn add(self, rhs: Self) -> Self::Output {
        match (self.as_number(), rhs.as_number()) {
            (Some(x), Some(y)) => Ok(Self::number(x + y)),
            _ => Err(ValueError),
        }
    }
}

impl Sub for Value {
    type Output = Result<Self, ValueError>;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self.as_number(), rhs.as_number()) {
            (Some(x), Some(y)) => Ok(Self::number(x - y)),
            _ => Err(ValueError),
        }
    }
}

impl Mul for Value {
    type Output = Result<Self, ValueError>;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self.as_number(), rhs.as_number()) {
            (Some(x), Some(y)) => Ok(Self::number(x * y)),
            _ => Err(ValueError),
        }
    }
}

impl Div for Value {
    type Output = Result<Self, ValueError>;

    fn div(self, rhs: Self) -> Self::Output {
        match (self.as_number(), rhs.as_number()) {
            (Some(x), Some(y)) => Ok(Self::number(x / y)),
            _ => Err(ValueError),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        if self.is_number() && other.is_number() {
            // IEEE float equality (NaN != NaN, -0.0 == 0.0) — bitwise would get
            // both wrong.
            f64::from_bits(self.0) == f64::from_bits(other.0)
        } else {
            // Nil/Bool/Symbol equality is value-identity; Object equality is
            // reference-identity — all exactly bit equality of the payload+tag.
            self.0 == other.0
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self.as_number(), other.as_number()) {
            (Some(a), Some(b)) => a.partial_cmp(&b),
            _ => match (self.as_boolean(), other.as_boolean()) {
                (Some(a), Some(b)) => a.partial_cmp(&b),
                _ if self.is_nil() && other.is_nil() => Some(Ordering::Equal),
                _ => None,
            },
        }
    }
}

impl Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.view() {
            ValueView::Nil => write!(f, "Nil"),
            ValueView::Boolean(b) => write!(f, "Boolean({b})"),
            ValueView::Number(n) => write!(f, "Number({n})"),
            ValueView::Symbol(key) => write!(f, "Symbol({})", key.into_usize()),
            ValueView::Object(obj) => write!(f, "Object({:?})", obj.kind()),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.view() {
            ValueView::Nil => write!(f, "nil"),
            ValueView::Boolean(b) => write!(f, "{b}"),
            ValueView::Number(n) => write!(f, "{n}"),
            ValueView::Symbol(key) => write!(f, "Symbol({})", key.into_inner()),
            ValueView::Object(obj) => obj.display_fmt(f),
        }
    }
}

impl Display for WithStorage<'_, Value> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0.view() {
            ValueView::Symbol(key) => write!(f, "{}", self.1.resolve(key)),
            ValueView::Object(obj) => WithStorage(&obj, self.1).fmt(f),
            _ => Display::fmt(self.0, f),
        }
    }
}

pub type Addr = u8;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_is_nan_boxed_into_eight_bytes() {
        // The whole point of the NaN-box: a `Value` is one machine word.
        assert_eq!(mem::size_of::<Value>(), 8);
        assert_eq!(mem::align_of::<Value>(), 8);
    }

    #[test]
    fn primitives_round_trip() {
        assert!(Value::nil().is_nil());
        assert_eq!(Value::boolean(true).as_boolean(), Some(true));
        assert_eq!(Value::boolean(false).as_boolean(), Some(false));
        assert_eq!(Value::number(3.5).as_number(), Some(3.5));
        assert!(Value::nil().is_falsey());
        assert!(Value::boolean(false).is_falsey());
        assert!(!Value::boolean(true).is_falsey());
        assert!(!Value::number(0.0).is_falsey());
    }

    #[test]
    fn number_nan_is_not_misread_as_a_box() {
        // A hardware quiet NaN (0x7ff8…) and ±inf must classify as numbers, never
        // as an object/symbol — that is the safety-critical partition property.
        for n in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 0.0, -0.0, 1e308] {
            let v = Value::number(n);
            assert!(v.is_number(), "{n} should be a number");
            assert!(!v.is_object() && !v.is_symbol() && !v.is_nil() && !v.is_boolean());
        }
        // NaN != NaN, but -0.0 == 0.0.
        assert_ne!(Value::number(f64::NAN), Value::number(f64::NAN));
        assert_eq!(Value::number(0.0), Value::number(-0.0));
    }
}
