//! `Deserialize` implementation for `Value`.

use crate::Value;
use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use std::fmt;

/// # Requires `deserialize_any`
///
/// This implementation calls [`Deserializer::deserialize_any`], which asks the format:
/// "what type do you have?". This only works with **self-describing formats**:
///
/// | Format | Works? | Why |
/// |--------|--------|-----|
/// | JSON | Yes | `{"x": 42}` embeds type info (object, string keys, number values) |
/// | MessagePack | Yes | Type tags precede each value |
/// | RON | Yes | Rust-like syntax with explicit types |
/// | bincode | **No** | No type info in byte stream |
/// | postcard | **No** | No type info in byte stream |
///
/// For non-self-describing formats, deserialize to a typed value first, then use
/// [`to_value()`](crate::to_value) to convert. Or use [`Meta`](crate::Meta) for
/// Value↔Value roundtrip.
impl<'de> Deserialize<'de> for Value {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(ValueVisitor)
    }
}

struct ValueVisitor;

impl<'de> Visitor<'de> for ValueVisitor {
    type Value = Value;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "any valid serde value")
    }

    fn visit_bool<E: de::Error>(self, v: bool) -> Result<Value, E> {
        Ok(Value::Bool(v))
    }

    fn visit_i8<E: de::Error>(self, v: i8) -> Result<Value, E> {
        Ok(Value::I8(v))
    }

    fn visit_i16<E: de::Error>(self, v: i16) -> Result<Value, E> {
        Ok(Value::I16(v))
    }

    fn visit_i32<E: de::Error>(self, v: i32) -> Result<Value, E> {
        Ok(Value::I32(v))
    }

    fn visit_i64<E: de::Error>(self, v: i64) -> Result<Value, E> {
        Ok(Value::I64(v))
    }

    fn visit_i128<E: de::Error>(self, v: i128) -> Result<Value, E> {
        Ok(Value::I128(v))
    }

    fn visit_u8<E: de::Error>(self, v: u8) -> Result<Value, E> {
        Ok(Value::U8(v))
    }

    fn visit_u16<E: de::Error>(self, v: u16) -> Result<Value, E> {
        Ok(Value::U16(v))
    }

    fn visit_u32<E: de::Error>(self, v: u32) -> Result<Value, E> {
        Ok(Value::U32(v))
    }

    fn visit_u64<E: de::Error>(self, v: u64) -> Result<Value, E> {
        Ok(Value::U64(v))
    }

    fn visit_u128<E: de::Error>(self, v: u128) -> Result<Value, E> {
        Ok(Value::U128(v))
    }

    fn visit_f32<E: de::Error>(self, v: f32) -> Result<Value, E> {
        Ok(Value::F32(v))
    }

    fn visit_f64<E: de::Error>(self, v: f64) -> Result<Value, E> {
        Ok(Value::F64(v))
    }

    fn visit_char<E: de::Error>(self, v: char) -> Result<Value, E> {
        Ok(Value::Char(v))
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<Value, E> {
        Ok(Value::String(v.to_owned()))
    }

    fn visit_string<E: de::Error>(self, v: String) -> Result<Value, E> {
        Ok(Value::String(v))
    }

    fn visit_bytes<E: de::Error>(self, v: &[u8]) -> Result<Value, E> {
        Ok(Value::Bytes(v.to_vec()))
    }

    fn visit_byte_buf<E: de::Error>(self, v: Vec<u8>) -> Result<Value, E> {
        Ok(Value::Bytes(v))
    }

    fn visit_none<E: de::Error>(self) -> Result<Value, E> {
        Ok(Value::None)
    }

    fn visit_some<D: Deserializer<'de>>(self, deserializer: D) -> Result<Value, D::Error> {
        Ok(Value::Some(Box::new(Value::deserialize(deserializer)?)))
    }

    fn visit_unit<E: de::Error>(self) -> Result<Value, E> {
        Ok(Value::Unit)
    }

    fn visit_newtype_struct<D: Deserializer<'de>>(self, deserializer: D) -> Result<Value, D::Error> {
        Value::deserialize(deserializer)
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Value, A::Error> {
        let mut values = Vec::new();
        while let Some(value) = seq.next_element()? {
            values.push(value);
        }
        Ok(Value::Seq(values))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Value, A::Error> {
        let mut entries = Vec::new();
        while let Some((key, value)) = map.next_entry()? {
            entries.push((key, value));
        }
        Ok(Value::Map(entries))
    }

    fn visit_enum<A: de::EnumAccess<'de>>(self, access: A) -> Result<Value, A::Error> {
        use de::VariantAccess;

        let (variant, variant_access) = access.variant::<String>()?;
        variant_access.unit_variant()?;

        let variant_static: &'static str = Box::leak(variant.into_boxed_str());

        Ok(Value::UnitVariant {
            enum_name: "",
            variant_index: 0,
            variant: variant_static,
        })
    }
}
