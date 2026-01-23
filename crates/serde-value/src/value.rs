//! The core `Value` type representing the complete serde data model.

use {
    serde::Serialize,
    std::{
        cmp::Ordering,
        hash::{
            Hash,
            Hasher,
        },
    },
};

/// A value that can represent any type in the serde data model.
///
/// This type preserves complete fidelity of the serde data model, including:
/// - All 14 primitive types (bool, i8-i128, u8-u128, f32, f64, char)
/// - The distinction between strings and byte arrays
/// - Option types (None vs Some)
/// - Unit types with names preserved
/// - Newtype wrappers with names preserved
/// - The distinction between sequences and tuples
/// - Struct names and field names
/// - Enum variant names and indices
///
/// # Serialization (Tagged Enum)
///
/// `Value` serializes as a **tagged enum** - like `#[derive(Serialize)]` would
/// produce. This works with ALL formats including bincode and postcard.
///
/// # Deserialization (Tagged Enum)
///
/// `Value` deserializes using `deserialize_enum`, which works with ALL formats.
///
/// # Transparent Serialization
///
/// To serialize `Value` **transparently** (producing identical bytes to the
/// original type), use [`Transparent(value)`](crate::Transparent). Note that
/// `Transparent` can only deserialize from self-describing formats (JSON,
/// MessagePack, RON).
#[derive(Debug, Clone, Serialize)]
#[must_use]
pub enum Value {
    // === Primitives (14) ===
    Bool(bool),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    F32(f32),
    F64(f64),
    Char(char),

    // === String and Bytes (2) ===
    String(String),
    Bytes(Vec<u8>),

    // === Option (2) ===
    None,
    Some(Box<Value>),

    // === Unit Types (2) ===
    Unit,
    UnitStruct {
        name: &'static str,
    },

    // === Newtype (2) ===
    NewtypeStruct {
        name: &'static str,
        value: Box<Value>,
    },
    NewtypeVariant {
        enum_name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: Box<Value>,
    },

    // === Sequences (4) ===
    Seq(Vec<Value>),
    Tuple(Vec<Value>),
    TupleStruct {
        name: &'static str,
        fields: Vec<Value>,
    },
    TupleVariant {
        enum_name: &'static str,
        variant_index: u32,
        variant: &'static str,
        fields: Vec<Value>,
    },

    // === Maps and Structs (3) ===
    Map(Vec<(Value, Value)>),
    Struct {
        name: &'static str,
        fields: Vec<(&'static str, Value)>,
    },
    StructVariant {
        enum_name: &'static str,
        variant_index: u32,
        variant: &'static str,
        fields: Vec<(&'static str, Value)>,
    },

    // === Unit Variant (1) ===
    UnitVariant {
        enum_name: &'static str,
        variant_index: u32,
        variant: &'static str,
    },
}

impl Value {
    /// Wrap this Value in [`Transparent`](crate::Transparent) for transparent
    /// serialization.
    ///
    /// Use this when you need the serialized output to match the original type
    /// exactly, rather than serializing Value as a tagged enum.
    pub fn transparent(self) -> crate::Transparent {
        crate::Transparent(self)
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        use Value::*;
        match (self, other) {
            // Primitives
            (Bool(a), Bool(b)) => a == b,
            (I8(a), I8(b)) => a == b,
            (I16(a), I16(b)) => a == b,
            (I32(a), I32(b)) => a == b,
            (I64(a), I64(b)) => a == b,
            (I128(a), I128(b)) => a == b,
            (U8(a), U8(b)) => a == b,
            (U16(a), U16(b)) => a == b,
            (U32(a), U32(b)) => a == b,
            (U64(a), U64(b)) => a == b,
            (U128(a), U128(b)) => a == b,
            // Floats use total_cmp for Eq behavior (NaN == NaN, -0 != +0)
            (F32(a), F32(b)) => a.total_cmp(b) == Ordering::Equal,
            (F64(a), F64(b)) => a.total_cmp(b) == Ordering::Equal,
            (Char(a), Char(b)) => a == b,

            // String and Bytes
            (String(a), String(b)) => a == b,
            (Bytes(a), Bytes(b)) => a == b,

            // Option
            (None, None) => true,
            (Some(a), Some(b)) => a == b,

            // Unit types
            (Unit, Unit) => true,
            (UnitStruct { name: a }, UnitStruct { name: b }) => a == b,

            // Newtype
            (
                NewtypeStruct {
                    name: n1,
                    value: v1,
                },
                NewtypeStruct {
                    name: n2,
                    value: v2,
                },
            ) => n1 == n2 && v1 == v2,
            (
                NewtypeVariant {
                    enum_name: e1,
                    variant_index: i1,
                    variant: v1,
                    value: val1,
                },
                NewtypeVariant {
                    enum_name: e2,
                    variant_index: i2,
                    variant: v2,
                    value: val2,
                },
            ) => e1 == e2 && i1 == i2 && v1 == v2 && val1 == val2,

            // Sequences
            (Seq(a), Seq(b)) => a == b,
            (Tuple(a), Tuple(b)) => a == b,
            (
                TupleStruct {
                    name: n1,
                    fields: f1,
                },
                TupleStruct {
                    name: n2,
                    fields: f2,
                },
            ) => n1 == n2 && f1 == f2,
            (
                TupleVariant {
                    enum_name: e1,
                    variant_index: i1,
                    variant: v1,
                    fields: f1,
                },
                TupleVariant {
                    enum_name: e2,
                    variant_index: i2,
                    variant: v2,
                    fields: f2,
                },
            ) => e1 == e2 && i1 == i2 && v1 == v2 && f1 == f2,

            // Maps and Structs
            (Map(a), Map(b)) => a == b,
            (
                Struct {
                    name: n1,
                    fields: f1,
                },
                Struct {
                    name: n2,
                    fields: f2,
                },
            ) => n1 == n2 && f1 == f2,
            (
                StructVariant {
                    enum_name: e1,
                    variant_index: i1,
                    variant: v1,
                    fields: f1,
                },
                StructVariant {
                    enum_name: e2,
                    variant_index: i2,
                    variant: v2,
                    fields: f2,
                },
            ) => e1 == e2 && i1 == i2 && v1 == v2 && f1 == f2,

            // Unit Variant
            (
                UnitVariant {
                    enum_name: e1,
                    variant_index: i1,
                    variant: v1,
                },
                UnitVariant {
                    enum_name: e2,
                    variant_index: i2,
                    variant: v2,
                },
            ) => e1 == e2 && i1 == i2 && v1 == v2,

            // Different variants are never equal
            _ => false,
        }
    }
}

impl Eq for Value {}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash the discriminant first
        std::mem::discriminant(self).hash(state);

        use Value::*;
        match self {
            // Primitives
            Bool(v) => v.hash(state),
            I8(v) => v.hash(state),
            I16(v) => v.hash(state),
            I32(v) => v.hash(state),
            I64(v) => v.hash(state),
            I128(v) => v.hash(state),
            U8(v) => v.hash(state),
            U16(v) => v.hash(state),
            U32(v) => v.hash(state),
            U64(v) => v.hash(state),
            U128(v) => v.hash(state),
            // Floats use to_bits for consistent hashing
            F32(v) => v.to_bits().hash(state),
            F64(v) => v.to_bits().hash(state),
            Char(v) => v.hash(state),

            // String and Bytes
            String(v) => v.hash(state),
            Bytes(v) => v.hash(state),

            // Option
            None => {}
            Some(v) => v.hash(state),

            // Unit types
            Unit => {}
            UnitStruct { name } => name.hash(state),

            // Newtype
            NewtypeStruct { name, value } => {
                name.hash(state);
                value.hash(state);
            }
            NewtypeVariant {
                enum_name,
                variant_index,
                variant,
                value,
            } => {
                enum_name.hash(state);
                variant_index.hash(state);
                variant.hash(state);
                value.hash(state);
            }

            // Sequences
            Seq(v) => v.hash(state),
            Tuple(v) => v.hash(state),
            TupleStruct { name, fields } => {
                name.hash(state);
                fields.hash(state);
            }
            TupleVariant {
                enum_name,
                variant_index,
                variant,
                fields,
            } => {
                enum_name.hash(state);
                variant_index.hash(state);
                variant.hash(state);
                fields.hash(state);
            }

            // Maps and Structs
            Map(entries) => {
                entries.len().hash(state);
                for (k, v) in entries {
                    k.hash(state);
                    v.hash(state);
                }
            }
            Struct { name, fields } => {
                name.hash(state);
                fields.len().hash(state);
                for (k, v) in fields {
                    k.hash(state);
                    v.hash(state);
                }
            }
            StructVariant {
                enum_name,
                variant_index,
                variant,
                fields,
            } => {
                enum_name.hash(state);
                variant_index.hash(state);
                variant.hash(state);
                fields.len().hash(state);
                for (k, v) in fields {
                    k.hash(state);
                    v.hash(state);
                }
            }

            // Unit Variant
            UnitVariant {
                enum_name,
                variant_index,
                variant,
            } => {
                enum_name.hash(state);
                variant_index.hash(state);
                variant.hash(state);
            }
        }
    }
}

impl Value {
    /// Returns a human-readable description of the value's type.
    pub fn type_name(&self) -> &'static str {
        use Value::*;
        match self {
            Bool(_) => "bool",
            I8(_) => "i8",
            I16(_) => "i16",
            I32(_) => "i32",
            I64(_) => "i64",
            I128(_) => "i128",
            U8(_) => "u8",
            U16(_) => "u16",
            U32(_) => "u32",
            U64(_) => "u64",
            U128(_) => "u128",
            F32(_) => "f32",
            F64(_) => "f64",
            Char(_) => "char",
            String(_) => "string",
            Bytes(_) => "bytes",
            None => "none",
            Some(_) => "some",
            Unit => "unit",
            UnitStruct { .. } => "unit struct",
            NewtypeStruct { .. } => "newtype struct",
            NewtypeVariant { .. } => "newtype variant",
            Seq(_) => "sequence",
            Tuple(_) => "tuple",
            TupleStruct { .. } => "tuple struct",
            TupleVariant { .. } => "tuple variant",
            Map(_) => "map",
            Struct { .. } => "struct",
            StructVariant { .. } => "struct variant",
            UnitVariant { .. } => "unit variant",
        }
    }
}
