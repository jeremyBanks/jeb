//! `Deserialize` implementation for `Value`.
//!
//! This uses `deserialize_enum` which works with ALL serde formats,
//! including non-self-describing formats like bincode and postcard.

use crate::Value;
use serde::de::{self, Deserialize, Deserializer, EnumAccess, SeqAccess, VariantAccess, Visitor};
use std::fmt;

// All Value variant names
const VARIANTS: &[&str] = &[
    "Bool", "I8", "I16", "I32", "I64", "I128",
    "U8", "U16", "U32", "U64", "U128",
    "F32", "F64", "Char", "String", "Bytes",
    "None", "Some", "Unit", "UnitStruct",
    "NewtypeStruct", "NewtypeVariant",
    "Seq", "Tuple", "TupleStruct", "TupleVariant",
    "Map", "Struct", "StructVariant", "UnitVariant",
];

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_enum("Value", VARIANTS, ValueVisitor)
    }
}

struct ValueVisitor;

/// Simple variant identifier - either an index or a name.
/// This is NOT recursive (unlike using Value) to avoid infinite recursion.
enum VariantIdent {
    Index(u32),
    Name(String),
}

impl<'de> Deserialize<'de> for VariantIdent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct VariantIdentVisitor;

        impl<'de> Visitor<'de> for VariantIdentVisitor {
            type Value = VariantIdent;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "variant index (integer) or variant name (string)")
            }

            fn visit_u32<E: de::Error>(self, v: u32) -> Result<VariantIdent, E> {
                Ok(VariantIdent::Index(v))
            }

            fn visit_u64<E: de::Error>(self, v: u64) -> Result<VariantIdent, E> {
                if v > u32::MAX as u64 {
                    Err(de::Error::custom(format!("variant index {} out of range", v)))
                } else {
                    Ok(VariantIdent::Index(v as u32))
                }
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<VariantIdent, E> {
                Ok(VariantIdent::Name(v.to_owned()))
            }

            fn visit_string<E: de::Error>(self, v: String) -> Result<VariantIdent, E> {
                Ok(VariantIdent::Name(v))
            }
        }

        deserializer.deserialize_identifier(VariantIdentVisitor)
    }
}

fn variant_ident_to_index<E: de::Error>(v: &VariantIdent) -> Result<u32, E> {
    match v {
        VariantIdent::Index(i) => Ok(*i),
        VariantIdent::Name(s) => match s.as_str() {
            "Bool" => Ok(0), "I8" => Ok(1), "I16" => Ok(2), "I32" => Ok(3), "I64" => Ok(4), "I128" => Ok(5),
            "U8" => Ok(6), "U16" => Ok(7), "U32" => Ok(8), "U64" => Ok(9), "U128" => Ok(10),
            "F32" => Ok(11), "F64" => Ok(12), "Char" => Ok(13), "String" => Ok(14), "Bytes" => Ok(15),
            "None" => Ok(16), "Some" => Ok(17), "Unit" => Ok(18), "UnitStruct" => Ok(19),
            "NewtypeStruct" => Ok(20), "NewtypeVariant" => Ok(21),
            "Seq" => Ok(22), "Tuple" => Ok(23), "TupleStruct" => Ok(24), "TupleVariant" => Ok(25),
            "Map" => Ok(26), "Struct" => Ok(27), "StructVariant" => Ok(28), "UnitVariant" => Ok(29),
            _ => Err(de::Error::unknown_variant(s, VARIANTS)),
        },
    }
}

impl<'de> Visitor<'de> for ValueVisitor {
    type Value = Value;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "a Value enum variant")
    }

    fn visit_enum<A: EnumAccess<'de>>(self, access: A) -> Result<Value, A::Error> {
        let (variant_ident, variant_access) = access.variant::<VariantIdent>()?;
        let variant_index = variant_ident_to_index(&variant_ident)?;

        match variant_index {
            0 => Ok(Value::Bool(variant_access.newtype_variant()?)),
            1 => Ok(Value::I8(variant_access.newtype_variant()?)),
            2 => Ok(Value::I16(variant_access.newtype_variant()?)),
            3 => Ok(Value::I32(variant_access.newtype_variant()?)),
            4 => Ok(Value::I64(variant_access.newtype_variant()?)),
            5 => Ok(Value::I128(variant_access.newtype_variant()?)),
            6 => Ok(Value::U8(variant_access.newtype_variant()?)),
            7 => Ok(Value::U16(variant_access.newtype_variant()?)),
            8 => Ok(Value::U32(variant_access.newtype_variant()?)),
            9 => Ok(Value::U64(variant_access.newtype_variant()?)),
            10 => Ok(Value::U128(variant_access.newtype_variant()?)),
            11 => Ok(Value::F32(variant_access.newtype_variant()?)),
            12 => Ok(Value::F64(variant_access.newtype_variant()?)),
            13 => Ok(Value::Char(variant_access.newtype_variant()?)),
            14 => Ok(Value::String(variant_access.newtype_variant()?)),
            15 => Ok(Value::Bytes(variant_access.newtype_variant()?)),
            16 => {
                variant_access.unit_variant()?;
                Ok(Value::None)
            }
            17 => {
                let inner: Value = variant_access.newtype_variant()?;
                Ok(Value::Some(Box::new(inner)))
            }
            18 => {
                variant_access.unit_variant()?;
                Ok(Value::Unit)
            }
            19 => {
                // UnitStruct: name
                let name: String = variant_access.newtype_variant()?;
                Ok(Value::UnitStruct {
                    name: Box::leak(name.into_boxed_str()),
                })
            }
            20 => {
                // NewtypeStruct: (name, value)
                let (name, value): (String, Value) = variant_access.tuple_variant(2, TupleVisitor2)?;
                Ok(Value::NewtypeStruct {
                    name: Box::leak(name.into_boxed_str()),
                    value: Box::new(value),
                })
            }
            21 => {
                // NewtypeVariant: (enum_name, variant_index, variant, value)
                let (enum_name, variant_index, variant, value): (String, u32, String, Value) =
                    variant_access.tuple_variant(4, TupleVisitor4Newtype)?;
                Ok(Value::NewtypeVariant {
                    enum_name: Box::leak(enum_name.into_boxed_str()),
                    variant_index,
                    variant: Box::leak(variant.into_boxed_str()),
                    value: Box::new(value),
                })
            }
            22 => {
                // Seq
                let values: Vec<Value> = variant_access.newtype_variant()?;
                Ok(Value::Seq(values))
            }
            23 => {
                // Tuple
                let values: Vec<Value> = variant_access.newtype_variant()?;
                Ok(Value::Tuple(values))
            }
            24 => {
                // TupleStruct: (name, fields)
                let (name, fields): (String, Vec<Value>) = variant_access.tuple_variant(2, TupleVisitor2Vec)?;
                Ok(Value::TupleStruct {
                    name: Box::leak(name.into_boxed_str()),
                    fields,
                })
            }
            25 => {
                // TupleVariant: (enum_name, variant_index, variant, fields)
                let (enum_name, variant_index, variant, fields): (String, u32, String, Vec<Value>) =
                    variant_access.tuple_variant(4, TupleVisitor4Vec)?;
                Ok(Value::TupleVariant {
                    enum_name: Box::leak(enum_name.into_boxed_str()),
                    variant_index,
                    variant: Box::leak(variant.into_boxed_str()),
                    fields,
                })
            }
            26 => {
                // Map
                let entries: Vec<(Value, Value)> = variant_access.newtype_variant()?;
                Ok(Value::Map(entries))
            }
            27 => {
                // Struct: (name, fields)
                let (name, fields): (String, Vec<(String, Value)>) = variant_access.tuple_variant(2, TupleVisitor2StructFields)?;
                Ok(Value::Struct {
                    name: Box::leak(name.into_boxed_str()),
                    fields: fields.into_iter()
                        .map(|(k, v)| (Box::leak(k.into_boxed_str()) as &'static str, v))
                        .collect(),
                })
            }
            28 => {
                // StructVariant: (enum_name, variant_index, variant, fields)
                let (enum_name, variant_index, variant, fields): (String, u32, String, Vec<(String, Value)>) =
                    variant_access.tuple_variant(4, TupleVisitor4StructFields)?;
                Ok(Value::StructVariant {
                    enum_name: Box::leak(enum_name.into_boxed_str()),
                    variant_index,
                    variant: Box::leak(variant.into_boxed_str()),
                    fields: fields.into_iter()
                        .map(|(k, v)| (Box::leak(k.into_boxed_str()) as &'static str, v))
                        .collect(),
                })
            }
            29 => {
                // UnitVariant: (enum_name, variant_index, variant)
                let (enum_name, variant_index, variant): (String, u32, String) =
                    variant_access.tuple_variant(3, TupleVisitor3)?;
                Ok(Value::UnitVariant {
                    enum_name: Box::leak(enum_name.into_boxed_str()),
                    variant_index,
                    variant: Box::leak(variant.into_boxed_str()),
                })
            }
            _ => Err(de::Error::invalid_value(
                de::Unexpected::Unsigned(variant_index as u64),
                &"variant index 0-29",
            )),
        }
    }
}

// Helper visitors for tuple variants

struct TupleVisitor2;
impl<'de> Visitor<'de> for TupleVisitor2 {
    type Value = (String, Value);
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
    type Value = (String, Vec<Value>);
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
    type Value = (String, Vec<(String, Value)>);
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
    type Value = (String, u32, String, Value);
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
    type Value = (String, u32, String, Vec<Value>);
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
    type Value = (String, u32, String, Vec<(String, Value)>);
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
