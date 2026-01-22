//! `Deserialize` implementation for `Value`.
//!
//! This allows deserializing a `Value` from any serde-compatible format.
//!
//! **Important limitation:** When deserializing from formats like JSON that don't
//! preserve type metadata, struct names and field names are lost. A JSON object
//! becomes `Value::Map`, not `Value::Struct`. This is fundamental to how serde works -
//! the type information flows from `deserialize_struct` to the format, not back.

use crate::Value;
use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use std::fmt;

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(ValueVisitor)
    }
}

/// Visitor that produces a `Value`.
struct ValueVisitor;

impl<'de> Visitor<'de> for ValueVisitor {
    type Value = Value;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "any valid serde value")
    }

    // === Primitives ===

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

    // === String and Bytes ===

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

    // === Option ===

    fn visit_none<E: de::Error>(self) -> Result<Value, E> {
        Ok(Value::None)
    }

    fn visit_some<D: Deserializer<'de>>(self, deserializer: D) -> Result<Value, D::Error> {
        Ok(Value::Some(Box::new(Value::deserialize(deserializer)?)))
    }

    // === Unit ===

    fn visit_unit<E: de::Error>(self) -> Result<Value, E> {
        Ok(Value::Unit)
    }

    // === Newtype Struct ===
    // Note: We lose the struct name here because the visitor doesn't receive it.
    // The name is passed to deserialize_newtype_struct, not to the visitor.

    fn visit_newtype_struct<D: Deserializer<'de>>(self, deserializer: D) -> Result<Value, D::Error> {
        // Without the name, we can only capture the inner value.
        // Return it directly rather than wrapping in NewtypeStruct.
        Value::deserialize(deserializer)
    }

    // === Sequences ===
    // Note: We cannot distinguish between seq, tuple, and tuple_struct here
    // because the visitor receives the same visit_seq call for all of them.

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Value, A::Error> {
        let mut values = Vec::new();
        while let Some(value) = seq.next_element()? {
            values.push(value);
        }
        Ok(Value::Seq(values))
    }

    // === Maps ===
    // Note: We cannot distinguish between map and struct here because
    // the visitor receives the same visit_map call for both.

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Value, A::Error> {
        let mut entries = Vec::new();
        while let Some((key, value)) = map.next_entry()? {
            entries.push((key, value));
        }
        Ok(Value::Map(entries))
    }

    // === Enum ===
    // Note: When deserializing from most formats (JSON, YAML, etc.), enums are
    // typically represented as strings (for unit variants) or objects (for other
    // variants). These get handled by visit_str and visit_map respectively.
    //
    // visit_enum is called when a format explicitly supports enum representation
    // (like our own Value deserializer). In that case, we preserve the information.

    fn visit_enum<A: de::EnumAccess<'de>>(self, access: A) -> Result<Value, A::Error> {
        let (variant, variant_access) = access.variant::<EnumVariantDeserializer>()?;
        variant_access.deserialize_variant(variant)
    }
}

/// Helper to capture the variant name from enum deserialization.
struct EnumVariantDeserializer {
    variant: String,
}

impl<'de> Deserialize<'de> for EnumVariantDeserializer {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct VariantVisitor;

        impl<'de> Visitor<'de> for VariantVisitor {
            type Value = EnumVariantDeserializer;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "variant identifier")
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<EnumVariantDeserializer, E> {
                Ok(EnumVariantDeserializer {
                    variant: v.to_owned(),
                })
            }

            fn visit_string<E: de::Error>(self, v: String) -> Result<EnumVariantDeserializer, E> {
                Ok(EnumVariantDeserializer { variant: v })
            }

            fn visit_u64<E: de::Error>(self, v: u64) -> Result<EnumVariantDeserializer, E> {
                Ok(EnumVariantDeserializer {
                    variant: v.to_string(),
                })
            }
        }

        deserializer.deserialize_identifier(VariantVisitor)
    }
}

impl EnumVariantDeserializer {
    fn deserialize_variant<'de, V: de::VariantAccess<'de>>(
        self,
        variant_access: V,
    ) -> Result<Value, V::Error> {
        // We have to pick one variant type to try. Since unit variants are most common
        // in simple enums, try that. If it fails, we'll get an error from the format.
        //
        // The fundamental issue is that serde's VariantAccess is consumed after one call,
        // so we can't probe for the type. In practice:
        // - Unit variants call unit_variant()
        // - Newtype variants call newtype_variant()
        // - etc.
        //
        // For a truly generic deserializer, we'd need format-specific handling.
        // For now, we support unit variants from visit_enum.

        variant_access.unit_variant()?;

        // We have to leak the string to get a 'static str.
        // This is the cost of dynamic variant names.
        let variant_static: &'static str = Box::leak(self.variant.into_boxed_str());

        Ok(Value::UnitVariant {
            enum_name: "",
            variant_index: 0,
            variant: variant_static,
        })
    }
}
