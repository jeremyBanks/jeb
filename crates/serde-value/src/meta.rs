//! `Meta` wrapper for tagged Value serialization.
//!
//! # The Problem
//!
//! `Value`'s default serialization is **transparent**: it produces identical bytes to the
//! original typed value. This is great for interop, but means `Value` cannot roundtrip
//! through non-self-describing formats like bincode:
//!
//! ```ignore
//! let value = Value::I32(42);
//! let bytes = bincode::serialize(&value)?;      // Just serializes 42
//! let back: Value = bincode::deserialize(&bytes)?;  // ERROR: deserialize_any not supported
//! ```
//!
//! The issue is `deserialize_any`: we ask bincode "what type do you have?" but bincode
//! can't answer - it needs US to tell IT what type to expect.
//!
//! # The Solution: `Meta<Value>`
//!
//! `Meta` wraps a `Value` and serializes it as a **tagged enum** with explicit variant
//! discriminants. This enables full `Value` ↔ `Value` roundtrip through ANY format:
//!
//! ```ignore
//! use serde_value::{Value, Meta};
//!
//! let value = Value::I32(42);
//! let meta = Meta(value);
//!
//! // Roundtrips through bincode!
//! let bytes = bincode::serialize(&meta)?;
//! let Meta(restored) = bincode::deserialize(&bytes)?;
//! assert_eq!(restored, Value::I32(42));
//! ```
//!
//! # Trade-off
//!
//! - `Value` (transparent): Identical bytes to original type, but can't deserialize from bincode
//! - `Meta<Value>` (tagged): Includes enum tags, but roundtrips through ANY format

use crate::Value;
use serde::de::{self, Deserialize, Deserializer, EnumAccess, SeqAccess, VariantAccess, Visitor};
use serde::ser::{Serialize, SerializeTupleVariant, Serializer};
use std::fmt;

/// Wrapper that serializes `Value` as a tagged enum for full-fidelity roundtrip.
///
/// Use this when you need to serialize and deserialize `Value` itself (not just
/// the data it represents) through non-self-describing formats like bincode.
#[derive(Debug, Clone, PartialEq)]
pub struct Meta(pub Value);

impl Meta {
    /// Wrap a Value in Meta for tagged serialization.
    pub fn new(value: Value) -> Self {
        Meta(value)
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

impl From<Value> for Meta {
    fn from(value: Value) -> Self {
        Meta(value)
    }
}

impl From<Meta> for Value {
    fn from(meta: Meta) -> Self {
        meta.0
    }
}

// All Value variant names for deserialize_enum
const VARIANTS: &[&str] = &[
    "Bool", "I8", "I16", "I32", "I64", "I128",
    "U8", "U16", "U32", "U64", "U128",
    "F32", "F64", "Char", "String", "Bytes",
    "None", "Some", "Unit", "UnitStruct",
    "NewtypeStruct", "NewtypeVariant",
    "Seq", "Tuple", "TupleStruct", "TupleVariant",
    "Map", "Struct", "StructVariant", "UnitVariant",
];

impl Serialize for Meta {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use Value::*;
        match &self.0 {
            // Primitives - newtype variants
            Bool(v) => serializer.serialize_newtype_variant("Value", 0, "Bool", v),
            I8(v) => serializer.serialize_newtype_variant("Value", 1, "I8", v),
            I16(v) => serializer.serialize_newtype_variant("Value", 2, "I16", v),
            I32(v) => serializer.serialize_newtype_variant("Value", 3, "I32", v),
            I64(v) => serializer.serialize_newtype_variant("Value", 4, "I64", v),
            I128(v) => serializer.serialize_newtype_variant("Value", 5, "I128", v),
            U8(v) => serializer.serialize_newtype_variant("Value", 6, "U8", v),
            U16(v) => serializer.serialize_newtype_variant("Value", 7, "U16", v),
            U32(v) => serializer.serialize_newtype_variant("Value", 8, "U32", v),
            U64(v) => serializer.serialize_newtype_variant("Value", 9, "U64", v),
            U128(v) => serializer.serialize_newtype_variant("Value", 10, "U128", v),
            F32(v) => serializer.serialize_newtype_variant("Value", 11, "F32", v),
            F64(v) => serializer.serialize_newtype_variant("Value", 12, "F64", v),
            Char(v) => serializer.serialize_newtype_variant("Value", 13, "Char", v),
            String(v) => serializer.serialize_newtype_variant("Value", 14, "String", v),
            Bytes(v) => serializer.serialize_newtype_variant("Value", 15, "Bytes", v),

            // Option
            None => serializer.serialize_unit_variant("Value", 16, "None"),
            Some(v) => serializer.serialize_newtype_variant("Value", 17, "Some", &Meta(v.as_ref().clone())),

            // Unit types
            Unit => serializer.serialize_unit_variant("Value", 18, "Unit"),
            UnitStruct { name } => {
                serializer.serialize_newtype_variant("Value", 19, "UnitStruct", name)
            }

            // Newtype
            NewtypeStruct { name, value } => {
                let mut tv = serializer.serialize_tuple_variant("Value", 20, "NewtypeStruct", 2)?;
                tv.serialize_field(name)?;
                tv.serialize_field(&Meta(value.as_ref().clone()))?;
                tv.end()
            }
            NewtypeVariant { enum_name, variant_index, variant, value } => {
                let mut tv = serializer.serialize_tuple_variant("Value", 21, "NewtypeVariant", 4)?;
                tv.serialize_field(enum_name)?;
                tv.serialize_field(variant_index)?;
                tv.serialize_field(variant)?;
                tv.serialize_field(&Meta(value.as_ref().clone()))?;
                tv.end()
            }

            // Sequences
            Seq(values) => {
                let wrapped: Vec<Meta> = values.iter().map(|v| Meta(v.clone())).collect();
                serializer.serialize_newtype_variant("Value", 22, "Seq", &wrapped)
            }
            Tuple(values) => {
                let wrapped: Vec<Meta> = values.iter().map(|v| Meta(v.clone())).collect();
                serializer.serialize_newtype_variant("Value", 23, "Tuple", &wrapped)
            }
            TupleStruct { name, fields } => {
                let wrapped: Vec<Meta> = fields.iter().map(|v| Meta(v.clone())).collect();
                let mut tv = serializer.serialize_tuple_variant("Value", 24, "TupleStruct", 2)?;
                tv.serialize_field(name)?;
                tv.serialize_field(&wrapped)?;
                tv.end()
            }
            TupleVariant { enum_name, variant_index, variant, fields } => {
                let wrapped: Vec<Meta> = fields.iter().map(|v| Meta(v.clone())).collect();
                let mut tv = serializer.serialize_tuple_variant("Value", 25, "TupleVariant", 4)?;
                tv.serialize_field(enum_name)?;
                tv.serialize_field(variant_index)?;
                tv.serialize_field(variant)?;
                tv.serialize_field(&wrapped)?;
                tv.end()
            }

            // Maps and Structs
            Map(entries) => {
                let wrapped: Vec<(Meta, Meta)> = entries.iter()
                    .map(|(k, v)| (Meta(k.clone()), Meta(v.clone())))
                    .collect();
                serializer.serialize_newtype_variant("Value", 26, "Map", &wrapped)
            }
            Struct { name, fields } => {
                let wrapped: Vec<(&str, Meta)> = fields.iter()
                    .map(|(k, v)| (*k, Meta(v.clone())))
                    .collect();
                let mut tv = serializer.serialize_tuple_variant("Value", 27, "Struct", 2)?;
                tv.serialize_field(name)?;
                tv.serialize_field(&wrapped)?;
                tv.end()
            }
            StructVariant { enum_name, variant_index, variant, fields } => {
                let wrapped: Vec<(&str, Meta)> = fields.iter()
                    .map(|(k, v)| (*k, Meta(v.clone())))
                    .collect();
                let mut tv = serializer.serialize_tuple_variant("Value", 28, "StructVariant", 4)?;
                tv.serialize_field(enum_name)?;
                tv.serialize_field(variant_index)?;
                tv.serialize_field(variant)?;
                tv.serialize_field(&wrapped)?;
                tv.end()
            }

            // Unit Variant
            UnitVariant { enum_name, variant_index, variant } => {
                let mut tv = serializer.serialize_tuple_variant("Value", 29, "UnitVariant", 3)?;
                tv.serialize_field(enum_name)?;
                tv.serialize_field(variant_index)?;
                tv.serialize_field(variant)?;
                tv.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for Meta {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_enum("Value", VARIANTS, MetaVisitor)
    }
}

struct MetaVisitor;

/// General-purpose variant identifier that stores either an index or a name.
///
/// This accepts:
/// - **Integers**: as variant index (with bounds checking)
/// - **Strings**: as variant name (caller does lookup)
/// - **Bytes**: as variant name if valid UTF-8
/// - **Floats**: as variant index if exact integer (no fractional part, in range)
///
/// Rejects (no sensible mapping exists):
/// - Bool, Char, Unit, None, Some, Sequences, Maps
enum VariantId {
    Index(u32),
    Name(String),
}

impl<'de> Deserialize<'de> for VariantId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct VariantIdVisitor;

        impl VariantIdVisitor {
            /// Convert an integer to u32, returning error if out of range.
            fn to_index<E: de::Error>(v: i128) -> Result<VariantId, E> {
                if v < 0 || v > u32::MAX as i128 {
                    Err(de::Error::invalid_value(
                        de::Unexpected::Other(&format!("integer {}", v)),
                        &"variant index in range 0..=4294967295",
                    ))
                } else {
                    Ok(VariantId::Index(v as u32))
                }
            }

            /// Convert an unsigned integer to u32, returning error if out of range.
            fn to_index_unsigned<E: de::Error>(v: u128) -> Result<VariantId, E> {
                if v > u32::MAX as u128 {
                    Err(de::Error::invalid_value(
                        de::Unexpected::Other(&format!("integer {}", v)),
                        &"variant index in range 0..=4294967295",
                    ))
                } else {
                    Ok(VariantId::Index(v as u32))
                }
            }
        }

        impl<'de> Visitor<'de> for VariantIdVisitor {
            type Value = VariantId;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "variant index (integer) or variant name (string)")
            }

            // === Signed integers: variant index with bounds checking ===
            fn visit_i8<E: de::Error>(self, v: i8) -> Result<VariantId, E> {
                Self::to_index(v as i128)
            }
            fn visit_i16<E: de::Error>(self, v: i16) -> Result<VariantId, E> {
                Self::to_index(v as i128)
            }
            fn visit_i32<E: de::Error>(self, v: i32) -> Result<VariantId, E> {
                Self::to_index(v as i128)
            }
            fn visit_i64<E: de::Error>(self, v: i64) -> Result<VariantId, E> {
                Self::to_index(v as i128)
            }
            fn visit_i128<E: de::Error>(self, v: i128) -> Result<VariantId, E> {
                Self::to_index(v)
            }

            // === Unsigned integers: variant index with bounds checking ===
            fn visit_u8<E: de::Error>(self, v: u8) -> Result<VariantId, E> {
                Ok(VariantId::Index(v as u32))
            }
            fn visit_u16<E: de::Error>(self, v: u16) -> Result<VariantId, E> {
                Ok(VariantId::Index(v as u32))
            }
            fn visit_u32<E: de::Error>(self, v: u32) -> Result<VariantId, E> {
                Ok(VariantId::Index(v))
            }
            fn visit_u64<E: de::Error>(self, v: u64) -> Result<VariantId, E> {
                Self::to_index_unsigned(v as u128)
            }
            fn visit_u128<E: de::Error>(self, v: u128) -> Result<VariantId, E> {
                Self::to_index_unsigned(v)
            }

            // === Floats: variant index if exact integer ===
            fn visit_f32<E: de::Error>(self, v: f32) -> Result<VariantId, E> {
                self.visit_f64(v as f64)
            }
            fn visit_f64<E: de::Error>(self, v: f64) -> Result<VariantId, E> {
                if v.is_nan() || v.is_infinite() || v != v.trunc() || v < 0.0 || v > u32::MAX as f64 {
                    Err(de::Error::invalid_value(
                        de::Unexpected::Float(v),
                        &"exact integer in range 0..=4294967295",
                    ))
                } else {
                    Ok(VariantId::Index(v as u32))
                }
            }

            // === Strings: variant name ===
            fn visit_str<E: de::Error>(self, v: &str) -> Result<VariantId, E> {
                Ok(VariantId::Name(v.to_owned()))
            }
            fn visit_string<E: de::Error>(self, v: String) -> Result<VariantId, E> {
                Ok(VariantId::Name(v))
            }

            // === Bytes: variant name if valid UTF-8 ===
            fn visit_bytes<E: de::Error>(self, v: &[u8]) -> Result<VariantId, E> {
                match std::str::from_utf8(v) {
                    Ok(s) => Ok(VariantId::Name(s.to_owned())),
                    Err(_) => Err(de::Error::invalid_value(
                        de::Unexpected::Bytes(v),
                        &"valid UTF-8 bytes for variant name",
                    )),
                }
            }
            fn visit_byte_buf<E: de::Error>(self, v: Vec<u8>) -> Result<VariantId, E> {
                match String::from_utf8(v) {
                    Ok(s) => Ok(VariantId::Name(s)),
                    Err(e) => Err(de::Error::invalid_value(
                        de::Unexpected::Bytes(e.as_bytes()),
                        &"valid UTF-8 bytes for variant name",
                    )),
                }
            }

            // All other types (bool, char, unit, none, some, seq, map) have no sensible mapping.
            // The default Visitor implementations return "invalid type" errors.
        }

        deserializer.deserialize_any(VariantIdVisitor)
    }
}

impl<'de> Visitor<'de> for MetaVisitor {
    type Value = Meta;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "a Value enum variant")
    }

    fn visit_enum<A: EnumAccess<'de>>(self, access: A) -> Result<Meta, A::Error> {
        let (variant_id, variant_access) = access.variant::<VariantId>()?;

        // Resolve variant identifier to index (Meta-specific lookup)
        let variant_index = match variant_id {
            VariantId::Index(i) => i,
            VariantId::Name(ref name) => match name.as_str() {
                "Bool" => 0, "I8" => 1, "I16" => 2, "I32" => 3, "I64" => 4, "I128" => 5,
                "U8" => 6, "U16" => 7, "U32" => 8, "U64" => 9, "U128" => 10,
                "F32" => 11, "F64" => 12, "Char" => 13, "String" => 14, "Bytes" => 15,
                "None" => 16, "Some" => 17, "Unit" => 18, "UnitStruct" => 19,
                "NewtypeStruct" => 20, "NewtypeVariant" => 21,
                "Seq" => 22, "Tuple" => 23, "TupleStruct" => 24, "TupleVariant" => 25,
                "Map" => 26, "Struct" => 27, "StructVariant" => 28, "UnitVariant" => 29,
                _ => return Err(de::Error::unknown_variant(name, VARIANTS)),
            },
        };

        let value = match variant_index {
            0 => Value::Bool(variant_access.newtype_variant()?),
            1 => Value::I8(variant_access.newtype_variant()?),
            2 => Value::I16(variant_access.newtype_variant()?),
            3 => Value::I32(variant_access.newtype_variant()?),
            4 => Value::I64(variant_access.newtype_variant()?),
            5 => Value::I128(variant_access.newtype_variant()?),
            6 => Value::U8(variant_access.newtype_variant()?),
            7 => Value::U16(variant_access.newtype_variant()?),
            8 => Value::U32(variant_access.newtype_variant()?),
            9 => Value::U64(variant_access.newtype_variant()?),
            10 => Value::U128(variant_access.newtype_variant()?),
            11 => Value::F32(variant_access.newtype_variant()?),
            12 => Value::F64(variant_access.newtype_variant()?),
            13 => Value::Char(variant_access.newtype_variant()?),
            14 => Value::String(variant_access.newtype_variant()?),
            15 => Value::Bytes(variant_access.newtype_variant()?),
            16 => {
                variant_access.unit_variant()?;
                Value::None
            }
            17 => {
                let Meta(inner) = variant_access.newtype_variant()?;
                Value::Some(Box::new(inner))
            }
            18 => {
                variant_access.unit_variant()?;
                Value::Unit
            }
            19 => {
                let (name,): (String,) = variant_access.newtype_variant()?;
                Value::UnitStruct {
                    name: Box::leak(name.into_boxed_str()),
                }
            }
            20 => {
                // NewtypeStruct: (name, value)
                let (name, Meta(value)): (String, Meta) = variant_access.tuple_variant(2, TupleVisitor2)?;
                Value::NewtypeStruct {
                    name: Box::leak(name.into_boxed_str()),
                    value: Box::new(value),
                }
            }
            21 => {
                // NewtypeVariant: (enum_name, variant_index, variant, value)
                let (enum_name, variant_index, variant, Meta(value)): (String, u32, String, Meta) =
                    variant_access.tuple_variant(4, TupleVisitor4Newtype)?;
                Value::NewtypeVariant {
                    enum_name: Box::leak(enum_name.into_boxed_str()),
                    variant_index,
                    variant: Box::leak(variant.into_boxed_str()),
                    value: Box::new(value),
                }
            }
            22 => {
                // Seq
                let wrapped: Vec<Meta> = variant_access.newtype_variant()?;
                Value::Seq(wrapped.into_iter().map(|m| m.0).collect())
            }
            23 => {
                // Tuple
                let wrapped: Vec<Meta> = variant_access.newtype_variant()?;
                Value::Tuple(wrapped.into_iter().map(|m| m.0).collect())
            }
            24 => {
                // TupleStruct: (name, fields)
                let (name, wrapped): (String, Vec<Meta>) = variant_access.tuple_variant(2, TupleVisitor2Vec)?;
                Value::TupleStruct {
                    name: Box::leak(name.into_boxed_str()),
                    fields: wrapped.into_iter().map(|m| m.0).collect(),
                }
            }
            25 => {
                // TupleVariant: (enum_name, variant_index, variant, fields)
                let (enum_name, variant_index, variant, wrapped): (String, u32, String, Vec<Meta>) =
                    variant_access.tuple_variant(4, TupleVisitor4Vec)?;
                Value::TupleVariant {
                    enum_name: Box::leak(enum_name.into_boxed_str()),
                    variant_index,
                    variant: Box::leak(variant.into_boxed_str()),
                    fields: wrapped.into_iter().map(|m| m.0).collect(),
                }
            }
            26 => {
                // Map
                let wrapped: Vec<(Meta, Meta)> = variant_access.newtype_variant()?;
                Value::Map(wrapped.into_iter().map(|(k, v)| (k.0, v.0)).collect())
            }
            27 => {
                // Struct: (name, fields)
                let (name, wrapped): (String, Vec<(String, Meta)>) = variant_access.tuple_variant(2, TupleVisitor2StructFields)?;
                Value::Struct {
                    name: Box::leak(name.into_boxed_str()),
                    fields: wrapped.into_iter()
                        .map(|(k, v)| (Box::leak(k.into_boxed_str()) as &'static str, v.0))
                        .collect(),
                }
            }
            28 => {
                // StructVariant: (enum_name, variant_index, variant, fields)
                let (enum_name, variant_index, variant, wrapped): (String, u32, String, Vec<(String, Meta)>) =
                    variant_access.tuple_variant(4, TupleVisitor4StructFields)?;
                Value::StructVariant {
                    enum_name: Box::leak(enum_name.into_boxed_str()),
                    variant_index,
                    variant: Box::leak(variant.into_boxed_str()),
                    fields: wrapped.into_iter()
                        .map(|(k, v)| (Box::leak(k.into_boxed_str()) as &'static str, v.0))
                        .collect(),
                }
            }
            29 => {
                // UnitVariant: (enum_name, variant_index, variant)
                let (enum_name, variant_index, variant): (String, u32, String) =
                    variant_access.tuple_variant(3, TupleVisitor3)?;
                Value::UnitVariant {
                    enum_name: Box::leak(enum_name.into_boxed_str()),
                    variant_index,
                    variant: Box::leak(variant.into_boxed_str()),
                }
            }
            _ => return Err(de::Error::invalid_value(
                de::Unexpected::Unsigned(variant_index as u64),
                &"variant index 0-29",
            )),
        };

        Ok(Meta(value))
    }
}

// Helper visitors for tuple variants

struct TupleVisitor2;
impl<'de> Visitor<'de> for TupleVisitor2 {
    type Value = (String, Meta);
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "a 2-element tuple")
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let a = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(0, &self))?;
        let b = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(1, &self))?;
        Ok((a, b))
    }
}

struct TupleVisitor2Vec;
impl<'de> Visitor<'de> for TupleVisitor2Vec {
    type Value = (String, Vec<Meta>);
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "a 2-element tuple")
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let a = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(0, &self))?;
        let b = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(1, &self))?;
        Ok((a, b))
    }
}

struct TupleVisitor2StructFields;
impl<'de> Visitor<'de> for TupleVisitor2StructFields {
    type Value = (String, Vec<(String, Meta)>);
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "a 2-element tuple")
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let a = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(0, &self))?;
        let b = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(1, &self))?;
        Ok((a, b))
    }
}

struct TupleVisitor3;
impl<'de> Visitor<'de> for TupleVisitor3 {
    type Value = (String, u32, String);
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "a 3-element tuple")
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let a = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(0, &self))?;
        let b = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(1, &self))?;
        let c = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(2, &self))?;
        Ok((a, b, c))
    }
}

struct TupleVisitor4Newtype;
impl<'de> Visitor<'de> for TupleVisitor4Newtype {
    type Value = (String, u32, String, Meta);
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "a 4-element tuple")
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let a = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(0, &self))?;
        let b = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(1, &self))?;
        let c = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(2, &self))?;
        let d = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(3, &self))?;
        Ok((a, b, c, d))
    }
}

struct TupleVisitor4Vec;
impl<'de> Visitor<'de> for TupleVisitor4Vec {
    type Value = (String, u32, String, Vec<Meta>);
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "a 4-element tuple")
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let a = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(0, &self))?;
        let b = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(1, &self))?;
        let c = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(2, &self))?;
        let d = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(3, &self))?;
        Ok((a, b, c, d))
    }
}

struct TupleVisitor4StructFields;
impl<'de> Visitor<'de> for TupleVisitor4StructFields {
    type Value = (String, u32, String, Vec<(String, Meta)>);
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "a 4-element tuple")
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let a = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(0, &self))?;
        let b = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(1, &self))?;
        let c = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(2, &self))?;
        let d = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(3, &self))?;
        Ok((a, b, c, d))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_meta_bincode_roundtrip_primitives() {
        let test_values = vec![
            Value::Bool(true),
            Value::Bool(false),
            Value::I8(-42),
            Value::I16(-1000),
            Value::I32(-100000),
            Value::I64(-1000000000),
            Value::U8(255),
            Value::U16(65535),
            Value::U32(4294967295),
            Value::U64(18446744073709551615),
            Value::F32(3.14),
            Value::F64(2.718281828),
            Value::Char('🦀'),
            Value::String("hello world".to_string()),
            Value::Bytes(vec![1, 2, 3, 4, 5]),
        ];

        for original in test_values {
            let meta = Meta(original.clone());
            let bytes = bincode::serialize(&meta).unwrap();
            let Meta(restored) = bincode::deserialize(&bytes).unwrap();
            assert_eq!(restored, original, "failed for {:?}", original);
        }
    }

    #[test]
    fn test_meta_bincode_roundtrip_option() {
        // None
        let original = Value::None;
        let meta = Meta(original.clone());
        let bytes = bincode::serialize(&meta).unwrap();
        let Meta(restored) = bincode::deserialize(&bytes).unwrap();
        assert_eq!(restored, original);

        // Some
        let original = Value::Some(Box::new(Value::I32(42)));
        let meta = Meta(original.clone());
        let bytes = bincode::serialize(&meta).unwrap();
        let Meta(restored) = bincode::deserialize(&bytes).unwrap();
        assert_eq!(restored, original);
    }

    #[test]
    fn test_meta_bincode_roundtrip_unit() {
        let original = Value::Unit;
        let meta = Meta(original.clone());
        let bytes = bincode::serialize(&meta).unwrap();
        let Meta(restored) = bincode::deserialize(&bytes).unwrap();
        assert_eq!(restored, original);
    }

    #[test]
    fn test_meta_bincode_roundtrip_seq() {
        let original = Value::Seq(vec![
            Value::I32(1),
            Value::I32(2),
            Value::I32(3),
        ]);
        let meta = Meta(original.clone());
        let bytes = bincode::serialize(&meta).unwrap();
        let Meta(restored) = bincode::deserialize(&bytes).unwrap();
        assert_eq!(restored, original);
    }

    #[test]
    fn test_meta_bincode_roundtrip_map() {
        let original = Value::Map(vec![
            (Value::String("key1".to_string()), Value::I32(1)),
            (Value::String("key2".to_string()), Value::I32(2)),
        ]);
        let meta = Meta(original.clone());
        let bytes = bincode::serialize(&meta).unwrap();
        let Meta(restored) = bincode::deserialize(&bytes).unwrap();
        assert_eq!(restored, original);
    }

    #[test]
    fn test_meta_bincode_roundtrip_struct() {
        let original = Value::Struct {
            name: "Point",
            fields: vec![
                ("x", Value::I32(10)),
                ("y", Value::I32(20)),
            ],
        };
        let meta = Meta(original.clone());
        let bytes = bincode::serialize(&meta).unwrap();
        let Meta(restored) = bincode::deserialize(&bytes).unwrap();
        assert_eq!(restored, original);
    }

    #[test]
    fn test_meta_bincode_roundtrip_nested() {
        let original = Value::Struct {
            name: "Outer",
            fields: vec![
                ("inner", Value::Struct {
                    name: "Inner",
                    fields: vec![
                        ("value", Value::I32(42)),
                    ],
                }),
                ("items", Value::Seq(vec![
                    Value::String("a".to_string()),
                    Value::String("b".to_string()),
                ])),
            ],
        };
        let meta = Meta(original.clone());
        let bytes = bincode::serialize(&meta).unwrap();
        let Meta(restored) = bincode::deserialize(&bytes).unwrap();
        assert_eq!(restored, original);
    }

    // === Meta-ception tests: Meta as Value, layers of abstraction ===

    #[test]
    fn test_meta_to_value_roundtrip() {
        use crate::{to_value, from_value};

        // Start with a Value
        let original = Value::I32(42);

        // Wrap in Meta
        let meta = Meta(original.clone());

        // Serialize Meta to Value (captures the Meta enum structure)
        let meta_as_value = to_value(&meta).unwrap();

        // The result should be a NewtypeVariant representing Meta's structure
        // (Meta serializes Value as a tagged enum)
        match &meta_as_value {
            Value::NewtypeVariant { enum_name, variant, .. } => {
                assert_eq!(*enum_name, "Value");
                assert_eq!(*variant, "I32");
            }
            other => panic!("expected NewtypeVariant, got {:?}", other),
        }

        // Deserialize back to Meta
        let meta_back: Meta = from_value(meta_as_value).unwrap();
        assert_eq!(meta_back.into_inner(), original);
    }

    #[test]
    fn test_meta_meta_double_wrap() {
        use crate::{to_value, from_value};

        // Start with a Value
        let original = Value::String("hello".to_string());

        // Wrap in Meta
        let meta1 = Meta(original.clone());

        // Capture Meta as a Value
        let meta1_as_value = to_value(&meta1).unwrap();

        // Wrap THAT in another Meta (meta-ception!)
        let meta2 = Meta(meta1_as_value.clone());

        // Roundtrip through bincode
        let bytes = bincode::serialize(&meta2).unwrap();
        let meta2_back: Meta = bincode::deserialize(&bytes).unwrap();

        // Unwrap outer Meta
        let meta1_as_value_back = meta2_back.into_inner();
        assert_eq!(meta1_as_value_back, meta1_as_value);

        // Deserialize inner to Meta
        let meta1_back: Meta = from_value(meta1_as_value_back).unwrap();
        assert_eq!(meta1_back.into_inner(), original);
    }

    #[test]
    fn test_meta_three_levels_deep() {
        use crate::{to_value, from_value};

        // Level 0: a simple value
        let v0 = Value::I32(42);

        // Level 1: Meta(v0) as Value
        let v1 = to_value(&Meta(v0.clone())).unwrap();

        // Level 2: Meta(v1) as Value
        let v2 = to_value(&Meta(v1.clone())).unwrap();

        // Level 3: Meta(v2) as Value
        let v3 = to_value(&Meta(v2.clone())).unwrap();

        // Wrap in Meta and roundtrip through bincode
        let bytes = bincode::serialize(&Meta(v3.clone())).unwrap();
        let restored: Meta = bincode::deserialize(&bytes).unwrap();

        // Unwrap all the layers
        let r3 = restored.into_inner();
        assert_eq!(r3, v3);

        let r2: Meta = from_value(r3).unwrap();
        assert_eq!(r2.inner(), &v2);

        let r1: Meta = from_value(r2.into_inner()).unwrap();
        assert_eq!(r1.inner(), &v1);

        let r0: Meta = from_value(r1.into_inner()).unwrap();
        assert_eq!(r0.into_inner(), v0);
    }

    #[test]
    fn test_value_containing_meta_representation() {
        use crate::to_value;

        // Create a struct Value
        let point = Value::Struct {
            name: "Point",
            fields: vec![
                ("x", Value::I32(10)),
                ("y", Value::I32(20)),
            ],
        };

        // Capture how Meta serializes it
        let meta_repr = to_value(&Meta(point.clone())).unwrap();

        // It should be a TupleVariant (how Meta serializes Struct)
        match &meta_repr {
            Value::TupleVariant { enum_name, variant, fields, .. } => {
                assert_eq!(*enum_name, "Value");
                assert_eq!(*variant, "Struct");
                assert_eq!(fields.len(), 2); // name and fields
            }
            other => panic!("expected TupleVariant for Struct, got {:?}", other),
        }
    }
}
