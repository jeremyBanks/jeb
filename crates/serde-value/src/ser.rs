//! `Serialize` implementation for `Value`.
//!
//! # Transparent Serialization
//!
//! This implementation is **transparent**: serializing a `Value` produces **identical bytes**
//! to serializing the original typed value. This is achieved by directly invoking the
//! appropriate serde serializer methods (e.g., `serialize_struct`, `serialize_i32`) rather
//! than serializing `Value` as an enum with discriminants.
//!
//! This enables interop with ANY serialization format, including non-self-describing binary
//! formats like bincode:
//!
//! ```ignore
//! let original = Point { x: 10, y: 20 };
//! let value = to_value(&original)?;
//!
//! // These produce IDENTICAL bytes:
//! assert_eq!(
//!     bincode::serialize(&original)?,
//!     bincode::serialize(&value)?
//! );
//! ```

use crate::Value;
use serde::ser::{
    Serialize, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant,
    SerializeTuple, SerializeTupleStruct, SerializeTupleVariant, Serializer,
};

// Alias to avoid collision with Value::Some
use std::option::Option::Some as StdSome;

impl Serialize for Value {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use Value::*;
        match self {
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
            Some(v) => serializer.serialize_some(v.as_ref()),

            // === Unit Types ===
            Unit => serializer.serialize_unit(),
            UnitStruct { name } => serializer.serialize_unit_struct(name),

            // === Newtype ===
            NewtypeStruct { name, value } => {
                serializer.serialize_newtype_struct(name, value.as_ref())
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
                value.as_ref(),
            ),

            // === Sequences ===
            Seq(values) => {
                let mut seq = serializer.serialize_seq(StdSome(values.len()))?;
                for value in values {
                    seq.serialize_element(value)?;
                }
                seq.end()
            }
            Tuple(values) => {
                let mut tuple = serializer.serialize_tuple(values.len())?;
                for value in values {
                    tuple.serialize_element(value)?;
                }
                tuple.end()
            }
            TupleStruct { name, fields } => {
                let mut ts = serializer.serialize_tuple_struct(name, fields.len())?;
                for field in fields {
                    ts.serialize_field(field)?;
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
                    tv.serialize_field(field)?;
                }
                tv.end()
            }

            // === Maps and Structs ===
            Map(entries) => {
                let mut map = serializer.serialize_map(StdSome(entries.len()))?;
                for (key, value) in entries {
                    map.serialize_entry(key, value)?;
                }
                map.end()
            }
            Struct { name, fields } => {
                let mut s = serializer.serialize_struct(name, fields.len())?;
                for (field_name, value) in fields {
                    s.serialize_field(field_name, value)?;
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
                    sv.serialize_field(field_name, value)?;
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
