//! `Transparent` wrapper for transparent Value serialization.
//!
//! # Transparent Serialization
//!
//! `Transparent` wraps a `Value` and serializes it **transparently** -
//! producing identical bytes to the original typed value. This is useful for
//! interop when you need the serialized output to match what the original type
//! would produce.
//!
//! ```ignore
//! use serde_value::{Value, Transparent, to_value};
//!
//! let original = Point { x: 10, y: 20 };
//! let value = to_value(&original)?;
//!
//! // Transparent serialization produces IDENTICAL bytes:
//! assert_eq!(
//!     bincode::serialize(&original)?,
//!     bincode::serialize(&Transparent(value))?,
//! );
//! ```
//!
//! # Deserialization (requires `deserialize_any`)
//!
//! `Transparent` deserializes by calling [`Deserializer::deserialize_any`],
//! which only works with **self-describing formats** (JSON, MessagePack, RON).
//! Non-self-describing formats (bincode, postcard) will error.
//!
//! For bincode/postcard: deserialize to a typed value first, then use
//! [`to_value()`](crate::to_value).
//!
//! # Trade-off
//!
//! - `Value` (default): Serializes as tagged enum, works with ALL formats
//! - `Transparent(Value)`: Identical bytes to original type, but deserialize
//!   only works with self-describing formats

// Alias to avoid collision with Value::Some
use {
    crate::Value,
    serde::{
        de::{
            self,
            Deserialize,
            Deserializer,
            MapAccess,
            SeqAccess,
            Visitor,
        },
        ser::{
            Serialize,
            SerializeMap,
            SerializeSeq,
            SerializeStruct,
            SerializeStructVariant,
            SerializeTuple,
            SerializeTupleStruct,
            SerializeTupleVariant,
            Serializer,
        },
    },
    std::{
        fmt,
        option::Option::Some as StdSome,
    },
};

/// Wrapper that serializes `Value` transparently, producing identical bytes to
/// the original type.
///
/// Use this when you need the serialized output to match what the original
/// typed value would produce, rather than serializing `Value` as an enum.
#[derive(Debug, Clone, PartialEq)]
pub struct Transparent(pub Value);

impl Transparent {
    /// Wrap a Value for transparent serialization.
    pub fn new(value: Value) -> Self {
        Transparent(value)
    }

    /// Unwrap the inner Value.
    pub fn into_inner(self) -> Value {
        self.0
    }

    /// Get a reference to the inner Value.
    pub fn inner(&self) -> &Value {
        &self.0
    }
}

impl From<Value> for Transparent {
    fn from(value: Value) -> Self {
        Transparent(value)
    }
}

impl From<Transparent> for Value {
    fn from(t: Transparent) -> Self {
        t.0
    }
}

// === Transparent Serialize ===
// Produces identical bytes to the original typed value.

impl Serialize for Transparent {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use Value::*;
        match &self.0 {
            // === Primitives ===
            Bool(v) => serializer.serialize_bool(*v),
            I8(v) => serializer.serialize_i8(*v),
            I16(v) => serializer.serialize_i16(*v),
            I32(v) => serializer.serialize_i32(*v),
            I64(v) => serializer.serialize_i64(*v),
            I128(v) => serializer.serialize_i128(*v),
            U8(v) => serializer.serialize_u8(*v),
            U16(v) => serializer.serialize_u16(*v),
            U32(v) => serializer.serialize_u32(*v),
            U64(v) => serializer.serialize_u64(*v),
            U128(v) => serializer.serialize_u128(*v),
            F32(v) => serializer.serialize_f32(*v),
            F64(v) => serializer.serialize_f64(*v),
            Char(v) => serializer.serialize_char(*v),

            // === String and Bytes ===
            String(v) => serializer.serialize_str(v),
            Bytes(v) => serializer.serialize_bytes(v),

            // === Option ===
            None => serializer.serialize_none(),
            Some(v) => serializer.serialize_some(&Transparent(v.as_ref().clone())),

            // === Unit Types ===
            Unit => serializer.serialize_unit(),
            UnitStruct { name } => serializer.serialize_unit_struct(name),

            // === Newtype ===
            NewtypeStruct { name, value } => {
                serializer.serialize_newtype_struct(name, &Transparent(value.as_ref().clone()))
            }
            NewtypeVariant {
                enum_name,
                variant_index,
                variant,
                value,
            } => serializer.serialize_newtype_variant(
                enum_name,
                *variant_index,
                variant,
                &Transparent(value.as_ref().clone()),
            ),

            // === Sequences ===
            Seq(values) => {
                let mut seq = serializer.serialize_seq(StdSome(values.len()))?;
                for value in values {
                    seq.serialize_element(&Transparent(value.clone()))?;
                }
                seq.end()
            }
            Tuple(values) => {
                let mut tuple = serializer.serialize_tuple(values.len())?;
                for value in values {
                    tuple.serialize_element(&Transparent(value.clone()))?;
                }
                tuple.end()
            }
            TupleStruct { name, fields } => {
                let mut ts = serializer.serialize_tuple_struct(name, fields.len())?;
                for field in fields {
                    ts.serialize_field(&Transparent(field.clone()))?;
                }
                ts.end()
            }
            TupleVariant {
                enum_name,
                variant_index,
                variant,
                fields,
            } => {
                let mut tv = serializer.serialize_tuple_variant(
                    enum_name,
                    *variant_index,
                    variant,
                    fields.len(),
                )?;
                for field in fields {
                    tv.serialize_field(&Transparent(field.clone()))?;
                }
                tv.end()
            }

            // === Maps and Structs ===
            Map(entries) => {
                let mut map = serializer.serialize_map(StdSome(entries.len()))?;
                for (key, value) in entries {
                    map.serialize_entry(&Transparent(key.clone()), &Transparent(value.clone()))?;
                }
                map.end()
            }
            Struct { name, fields } => {
                let mut s = serializer.serialize_struct(name, fields.len())?;
                for (field_name, value) in fields {
                    s.serialize_field(field_name, &Transparent(value.clone()))?;
                }
                s.end()
            }
            StructVariant {
                enum_name,
                variant_index,
                variant,
                fields,
            } => {
                let mut sv = serializer.serialize_struct_variant(
                    enum_name,
                    *variant_index,
                    variant,
                    fields.len(),
                )?;
                for (field_name, value) in fields {
                    sv.serialize_field(field_name, &Transparent(value.clone()))?;
                }
                sv.end()
            }

            // === Unit Variant ===
            UnitVariant {
                enum_name,
                variant_index,
                variant,
            } => serializer.serialize_unit_variant(enum_name, *variant_index, variant),
        }
    }
}

// === Transparent Deserialize ===
// Uses deserialize_any, only works with self-describing formats.

impl<'de> Deserialize<'de> for Transparent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(TransparentVisitor)
    }
}

struct TransparentVisitor;

impl<'de> Visitor<'de> for TransparentVisitor {
    type Value = Transparent;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "any valid serde value")
    }

    fn visit_bool<E: de::Error>(self, v: bool) -> Result<Transparent, E> {
        Ok(Transparent(Value::Bool(v)))
    }

    fn visit_i8<E: de::Error>(self, v: i8) -> Result<Transparent, E> {
        Ok(Transparent(Value::I8(v)))
    }

    fn visit_i16<E: de::Error>(self, v: i16) -> Result<Transparent, E> {
        Ok(Transparent(Value::I16(v)))
    }

    fn visit_i32<E: de::Error>(self, v: i32) -> Result<Transparent, E> {
        Ok(Transparent(Value::I32(v)))
    }

    fn visit_i64<E: de::Error>(self, v: i64) -> Result<Transparent, E> {
        Ok(Transparent(Value::I64(v)))
    }

    fn visit_i128<E: de::Error>(self, v: i128) -> Result<Transparent, E> {
        Ok(Transparent(Value::I128(v)))
    }

    fn visit_u8<E: de::Error>(self, v: u8) -> Result<Transparent, E> {
        Ok(Transparent(Value::U8(v)))
    }

    fn visit_u16<E: de::Error>(self, v: u16) -> Result<Transparent, E> {
        Ok(Transparent(Value::U16(v)))
    }

    fn visit_u32<E: de::Error>(self, v: u32) -> Result<Transparent, E> {
        Ok(Transparent(Value::U32(v)))
    }

    fn visit_u64<E: de::Error>(self, v: u64) -> Result<Transparent, E> {
        Ok(Transparent(Value::U64(v)))
    }

    fn visit_u128<E: de::Error>(self, v: u128) -> Result<Transparent, E> {
        Ok(Transparent(Value::U128(v)))
    }

    fn visit_f32<E: de::Error>(self, v: f32) -> Result<Transparent, E> {
        Ok(Transparent(Value::F32(v)))
    }

    fn visit_f64<E: de::Error>(self, v: f64) -> Result<Transparent, E> {
        Ok(Transparent(Value::F64(v)))
    }

    fn visit_char<E: de::Error>(self, v: char) -> Result<Transparent, E> {
        Ok(Transparent(Value::Char(v)))
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<Transparent, E> {
        Ok(Transparent(Value::String(v.to_owned())))
    }

    fn visit_string<E: de::Error>(self, v: String) -> Result<Transparent, E> {
        Ok(Transparent(Value::String(v)))
    }

    fn visit_bytes<E: de::Error>(self, v: &[u8]) -> Result<Transparent, E> {
        Ok(Transparent(Value::Bytes(v.to_vec())))
    }

    fn visit_byte_buf<E: de::Error>(self, v: Vec<u8>) -> Result<Transparent, E> {
        Ok(Transparent(Value::Bytes(v)))
    }

    fn visit_none<E: de::Error>(self) -> Result<Transparent, E> {
        Ok(Transparent(Value::None))
    }

    fn visit_some<D: Deserializer<'de>>(self, deserializer: D) -> Result<Transparent, D::Error> {
        let inner = Transparent::deserialize(deserializer)?;
        Ok(Transparent(Value::Some(Box::new(inner.0))))
    }

    fn visit_unit<E: de::Error>(self) -> Result<Transparent, E> {
        Ok(Transparent(Value::Unit))
    }

    fn visit_newtype_struct<D: Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<Transparent, D::Error> {
        Transparent::deserialize(deserializer)
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Transparent, A::Error> {
        let mut values = Vec::new();
        while let Some(Transparent(value)) = seq.next_element()? {
            values.push(value);
        }
        Ok(Transparent(Value::Seq(values)))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Transparent, A::Error> {
        let mut entries = Vec::new();
        while let Some((Transparent(key), Transparent(value))) = map.next_entry()? {
            entries.push((key, value));
        }
        Ok(Transparent(Value::Map(entries)))
    }

    fn visit_enum<A: de::EnumAccess<'de>>(self, access: A) -> Result<Transparent, A::Error> {
        use de::VariantAccess;

        let (variant, variant_access) = access.variant::<String>()?;
        variant_access.unit_variant()?;

        let variant_static: &'static str = Box::leak(variant.into_boxed_str());

        Ok(Transparent(Value::UnitVariant {
            enum_name: "",
            variant_index: 0,
            variant: variant_static,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transparent_roundtrip_json_string() {
        // Strings roundtrip exactly through JSON
        let original = Value::String("hello".to_string());
        let t = Transparent(original.clone());

        let json = serde_json::to_string(&t).unwrap();
        let back: Transparent = serde_json::from_str(&json).unwrap();

        assert_eq!(back.0, original);
    }

    #[test]
    fn test_transparent_json_integer_type_not_preserved() {
        // JSON doesn't preserve integer types - this is expected behavior
        let original = Value::I32(42);
        let t = Transparent(original);

        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, "42"); // Just the number

        let back: Transparent = serde_json::from_str(&json).unwrap();
        // JSON integers come back as U64 (serde_json's default)
        assert_eq!(back.0, Value::U64(42));
    }

    #[test]
    fn test_transparent_produces_same_bytes() {
        // Transparent serialization should produce identical bytes to direct
        // serialization
        let value = Value::String("hello".to_string());
        let t = Transparent(value);

        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, "\"hello\""); // Just the string, no enum wrapper
    }
}
