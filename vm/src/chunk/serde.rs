//! Custom postcard/serde de/serialization for [`Chunk`].
//!
//! The constant pool's string values are `Storage`-relative (a symbol-tagged
//! `Value` holds an interner key, an object-tagged one a heap pointer), so the
//! wire form can't be `#[derive]`d. Serialization borrows `&Storage` to inline the text;
//! deserialization threads `&mut Storage` via [`DeserializeSeed`] to re-intern it.

use std::fmt;
use std::io::{Read, Write};

use serde::de::{DeserializeSeed, EnumAccess, Error as _, SeqAccess, VariantAccess, Visitor};
use serde::ser::{Error as _, SerializeSeq, SerializeStruct};
use serde::{Deserializer, Serialize, Serializer};
use thiserror::Error;

use crate::chunk::Chunk;
use crate::object::{ObjKind, string::LoxString};
use crate::storage::{Storage, WithStorage};
use crate::value::{Value, ValueView};

#[derive(Debug, Error)]
pub enum ChunkIoError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("(de)serialization error: {0}")]
    Postcard(#[from] postcard::Error),
}

impl Chunk {
    /// Serialize this chunk to `writer` in postcard format. `storage` resolves
    /// the chunk's interned/heap string constants, so pass the `Storage` it was
    /// compiled against.
    pub fn serialize<W: Write>(
        &self,
        storage: &Storage,
        writer: &mut W,
    ) -> Result<(), ChunkIoError> {
        postcard::to_io(&WithStorage(self, storage), writer)?;
        Ok(())
    }

    /// Load a chunk written by [`serialize`](Self::serialize). Its string
    /// constants are re-interned / re-allocated into `storage`, which must be
    /// the `Storage` the executing VM will use.
    pub fn load<R: Read>(storage: &mut Storage, reader: &mut R) -> Result<Self, ChunkIoError> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes)?;
        Self::from_bytes(storage, &bytes)
    }

    /// Load a chunk from an in-memory image. Prefer over [`load`](Self::load) when
    /// the bytes are already resident (e.g. an `mmap`ed file): strings are borrowed
    /// from `bytes`, so only the interned/heap copies allocate.
    pub fn from_bytes(storage: &mut Storage, bytes: &[u8]) -> Result<Self, ChunkIoError> {
        let mut de = postcard::Deserializer::from_bytes(bytes);
        Ok(ChunkSeed(storage).deserialize(&mut de)?)
    }
}

/// Serializes a byte slice as one opaque blob instead of a `u8` sequence.
struct Bytes<'a>(&'a [u8]);

impl Serialize for Bytes<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(self.0)
    }
}

impl Serialize for WithStorage<'_, Chunk> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let (chunk, storage) = (self.0, self.1);
        let mut s = serializer.serialize_struct("Chunk", 4)?;
        s.serialize_field("code", &Bytes(&chunk.code))?;
        s.serialize_field(
            "constants",
            &WithStorage(chunk.constants.as_slice(), storage),
        )?;
        s.serialize_field("lines", &chunk.lines)?;
        s.serialize_field("label", &chunk.label)?;
        s.end()
    }
}

impl Serialize for WithStorage<'_, [Value]> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let (values, storage) = (self.0, self.1);
        let mut seq = serializer.serialize_seq(Some(values.len()))?;
        for value in values {
            seq.serialize_element(&WithStorage(value, storage))?;
        }
        seq.end()
    }
}

impl Serialize for WithStorage<'_, Value> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let (value, storage) = (self.0, self.1);
        // Variant indices are part of the wire format — only ever append.
        const NAME: &str = "Value";
        match value.view() {
            ValueView::Nil => serializer.serialize_unit_variant(NAME, 0, "Nil"),
            ValueView::Boolean(b) => serializer.serialize_newtype_variant(NAME, 1, "Boolean", &b),
            ValueView::Number(n) => serializer.serialize_newtype_variant(NAME, 2, "Number", &n),
            ValueView::Symbol(key) => {
                serializer.serialize_newtype_variant(NAME, 3, "Symbol", storage.resolve(key))
            }
            ValueView::Object(obj) if obj.kind() == ObjKind::String => {
                serializer.serialize_newtype_variant(NAME, 4, "Str", obj.as_str())
            }
            ValueView::Object(_) => Err(S::Error::custom(
                "constant pool holds a non-string object that cannot be serialized",
            )),
        }
    }
}

struct ChunkSeed<'a>(&'a mut Storage);

impl<'de> DeserializeSeed<'de> for ChunkSeed<'_> {
    type Value = Chunk;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Chunk, D::Error> {
        struct ChunkVisitor<'a>(&'a mut Storage);

        impl<'de> Visitor<'de> for ChunkVisitor<'_> {
            type Value = Chunk;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a serialized Chunk")
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Chunk, A::Error> {
                let missing = |field| A::Error::custom(format!("Chunk: missing `{field}`"));
                let code: &[u8] = seq.next_element()?.ok_or_else(|| missing("code"))?;
                let constants = seq
                    .next_element_seed(ConstantsSeed(self.0))?
                    .ok_or_else(|| missing("constants"))?;
                let lines = seq.next_element()?.ok_or_else(|| missing("lines"))?;
                let label = seq.next_element()?.ok_or_else(|| missing("label"))?;
                Ok(Chunk {
                    code: code.to_vec(),
                    constants,
                    lines,
                    label,
                })
            }
        }

        const FIELDS: &[&str] = &["code", "constants", "lines", "label"];
        deserializer.deserialize_struct("Chunk", FIELDS, ChunkVisitor(self.0))
    }
}

struct ConstantsSeed<'a>(&'a mut Storage);

impl<'de> DeserializeSeed<'de> for ConstantsSeed<'_> {
    type Value = Vec<Value>;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Vec<Value>, D::Error> {
        struct ConstantsVisitor<'a>(&'a mut Storage);

        impl<'de> Visitor<'de> for ConstantsVisitor<'_> {
            type Value = Vec<Value>;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a constant pool")
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Vec<Value>, A::Error> {
                let mut constants = Vec::with_capacity(seq.size_hint().unwrap_or(0));
                // Reborrow `self.0` each round so the seed can be handed out repeatedly.
                while let Some(value) = seq.next_element_seed(ValueSeed(&mut *self.0))? {
                    constants.push(value);
                }
                Ok(constants)
            }
        }

        deserializer.deserialize_seq(ConstantsVisitor(self.0))
    }
}

struct ValueSeed<'a>(&'a mut Storage);

impl<'de> DeserializeSeed<'de> for ValueSeed<'_> {
    type Value = Value;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Value, D::Error> {
        struct ValueVisitor<'a>(&'a mut Storage);

        impl<'de> Visitor<'de> for ValueVisitor<'_> {
            type Value = Value;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a Lox constant value")
            }

            fn visit_enum<A: EnumAccess<'de>>(self, data: A) -> Result<Value, A::Error> {
                let (index, payload) = data.variant::<u32>()?;
                Ok(match index {
                    0 => {
                        payload.unit_variant()?;
                        Value::nil()
                    }
                    1 => Value::boolean(payload.newtype_variant()?),
                    2 => {
                        // Validate at the trust boundary: a corrupt/hostile chunk
                        // could encode a non-number NaN whose bits collide with the
                        // NaN-box Object/Symbol tag, and `Value::number` keeps raw
                        // bits. Rejecting it here means deserialization can never
                        // mint a tag-aliasing `Value` that the rest of the VM would
                        // misread as a (wild) pointer.
                        let value = Value::number(payload.newtype_variant()?);
                        if !value.is_number() {
                            return Err(A::Error::custom("constant pool holds a non-number NaN"));
                        }
                        value
                    }
                    3 => {
                        let name: &str = payload.newtype_variant()?;
                        Value::symbol(self.0.intern(name))
                    }
                    4 => {
                        let text: &str = payload.newtype_variant()?;
                        Value::object(self.0.add_obj(LoxString::boxed(text)))
                    }
                    other => {
                        return Err(A::Error::custom(format!("unknown Value variant {other}")));
                    }
                })
            }
        }

        const VARIANTS: &[&str] = &["Nil", "Boolean", "Number", "Symbol", "Str"];
        deserializer.deserialize_enum("Value", VARIANTS, ValueVisitor(self.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enconding::OpCode;
    use crate::vm::VirtualMachine;

    /// Build a small valid program: `Constant 1.5; DefGlobal foo; Ret`.
    fn sample(storage: &mut Storage) -> Chunk {
        let mut chunk = Chunk::with_label("sample".into());
        let num = chunk.add_constant(Value::number(1.5));
        let foo = chunk.add_constant(Value::symbol(storage.intern("foo")));
        chunk.write_with_line(1, OpCode::Constant(num));
        chunk.write_with_line(1, OpCode::DefGlobal(foo));
        // Top-level chunks return implicitly: push the (discarded) return value
        // the `Ret` semantics expect.
        chunk.write_with_line(2, OpCode::Nil);
        chunk.write_with_line(2, OpCode::Ret);
        chunk
    }

    #[test]
    fn round_trips_code_lines_and_constants() {
        let mut src_storage = Storage::new();
        let original = sample(&mut src_storage);

        let mut bytes = Vec::new();
        original.serialize(&src_storage, &mut bytes).unwrap();

        let mut dst_storage = Storage::new();
        let loaded = Chunk::load(&mut dst_storage, &mut bytes.as_slice()).unwrap();

        assert_eq!(loaded.code, original.code);
        assert_eq!(loaded.lines, original.lines);
        assert_eq!(loaded.label, original.label);
        assert_eq!(loaded.constant(0), &Value::number(1.5));
        // The symbol is re-interned into the destination storage, so its Spur
        // may differ; the resolved text must not.
        let Some(key) = loaded.constant(1).as_symbol() else {
            panic!("constant 1 should be a symbol")
        };
        assert_eq!(dst_storage.resolve(key), "foo");
    }

    #[test]
    fn round_trips_all_constant_kinds() {
        let mut src = Storage::new();
        let mut chunk = Chunk::default();
        chunk.add_constant(Value::nil());
        chunk.add_constant(Value::boolean(true));
        chunk.add_constant(Value::number(2.5));
        chunk.add_constant(Value::symbol(src.intern("name")));
        chunk.add_constant(Value::object(src.add_obj(LoxString::boxed("hello"))));

        let mut bytes = Vec::new();
        chunk.serialize(&src, &mut bytes).unwrap();

        let mut dst = Storage::new();
        let loaded = Chunk::from_bytes(&mut dst, &bytes).unwrap();

        assert_eq!(loaded.constant(0), &Value::nil());
        assert_eq!(loaded.constant(1), &Value::boolean(true));
        assert_eq!(loaded.constant(2), &Value::number(2.5));
        let Some(key) = loaded.constant(3).as_symbol() else {
            panic!("constant 3 should be a symbol")
        };
        assert_eq!(dst.resolve(key), "name");
        let Some(obj) = loaded.constant(4).as_object() else {
            panic!("constant 4 should be a heap string")
        };
        assert_eq!(obj.kind(), ObjKind::String);
        assert_eq!(obj.as_str(), "hello");
    }

    #[test]
    fn loaded_chunk_executes() {
        let mut src_storage = Storage::new();
        let original = sample(&mut src_storage);
        let mut bytes = Vec::new();
        original.serialize(&src_storage, &mut bytes).unwrap();

        let mut vm = VirtualMachine::default();
        let loaded = Chunk::load(vm.storage(), &mut bytes.as_slice()).unwrap();
        vm.run(loaded).unwrap();
    }
}
