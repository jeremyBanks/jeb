//! `Deserializer` implementation for `Value`.
//!
//! This allows deserializing any `Deserialize` type from a `Value`.

use crate::{Error, Value};
use serde::de::{
    self, DeserializeSeed, Deserializer, EnumAccess, IntoDeserializer, MapAccess, SeqAccess,
    VariantAccess, Visitor,
};

/// Convert a `Value` to any `DeserializeOwned` type.
pub fn from_value<T: de::DeserializeOwned>(value: Value) -> Result<T, Error> {
    T::deserialize(value)
}

impl<'de> Deserializer<'de> for Value {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        use Value::*;
        match self {
            Bool(v) => visitor.visit_bool(v),
            I8(v) => visitor.visit_i8(v),
            I16(v) => visitor.visit_i16(v),
            I32(v) => visitor.visit_i32(v),
            I64(v) => visitor.visit_i64(v),
            I128(v) => visitor.visit_i128(v),
            U8(v) => visitor.visit_u8(v),
            U16(v) => visitor.visit_u16(v),
            U32(v) => visitor.visit_u32(v),
            U64(v) => visitor.visit_u64(v),
            U128(v) => visitor.visit_u128(v),
            F32(v) => visitor.visit_f32(v),
            F64(v) => visitor.visit_f64(v),
            Char(v) => visitor.visit_char(v),
            String(v) => visitor.visit_string(v),
            Bytes(v) => visitor.visit_byte_buf(v),
            None => visitor.visit_none(),
            Some(v) => visitor.visit_some(*v),
            Unit => visitor.visit_unit(),
            UnitStruct { .. } => visitor.visit_unit(),
            NewtypeStruct { value, .. } => visitor.visit_newtype_struct(*value),
            NewtypeVariant {
                enum_name,
                variant_index,
                variant,
                value,
            } => visitor.visit_enum(EnumDeserializer::NewtypeVariant {
                enum_name,
                variant_index,
                variant,
                value: *value,
            }),
            Seq(v) => visitor.visit_seq(SeqDeserializer::new(v)),
            Tuple(v) => visitor.visit_seq(SeqDeserializer::new(v)),
            TupleStruct { fields, .. } => visitor.visit_seq(SeqDeserializer::new(fields)),
            TupleVariant {
                enum_name,
                variant_index,
                variant,
                fields,
            } => visitor.visit_enum(EnumDeserializer::TupleVariant {
                enum_name,
                variant_index,
                variant,
                fields,
            }),
            Map(entries) => visitor.visit_map(MapDeserializer::new(entries)),
            Struct { fields, .. } => visitor.visit_map(StructDeserializer::new(fields)),
            StructVariant {
                enum_name,
                variant_index,
                variant,
                fields,
            } => visitor.visit_enum(EnumDeserializer::StructVariant {
                enum_name,
                variant_index,
                variant,
                fields,
            }),
            UnitVariant {
                enum_name,
                variant_index,
                variant,
            } => visitor.visit_enum(EnumDeserializer::UnitVariant {
                enum_name,
                variant_index,
                variant,
            }),
        }
    }

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self {
            Value::Bool(v) => visitor.visit_bool(v),
            other => Err(Error::type_mismatch("bool", other.type_name())),
        }
    }

    fn deserialize_i8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self {
            Value::I8(v) => visitor.visit_i8(v),
            other => unimplemented!(
                "integer coercion from {} to i8 not implemented",
                other.type_name()
            ),
        }
    }

    fn deserialize_i16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self {
            Value::I16(v) => visitor.visit_i16(v),
            other => unimplemented!(
                "integer coercion from {} to i16 not implemented",
                other.type_name()
            ),
        }
    }

    fn deserialize_i32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self {
            Value::I32(v) => visitor.visit_i32(v),
            other => unimplemented!(
                "integer coercion from {} to i32 not implemented",
                other.type_name()
            ),
        }
    }

    fn deserialize_i64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self {
            Value::I64(v) => visitor.visit_i64(v),
            other => unimplemented!(
                "integer coercion from {} to i64 not implemented",
                other.type_name()
            ),
        }
    }

    fn deserialize_i128<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self {
            Value::I128(v) => visitor.visit_i128(v),
            other => unimplemented!(
                "integer coercion from {} to i128 not implemented",
                other.type_name()
            ),
        }
    }

    fn deserialize_u8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self {
            Value::U8(v) => visitor.visit_u8(v),
            other => unimplemented!(
                "integer coercion from {} to u8 not implemented",
                other.type_name()
            ),
        }
    }

    fn deserialize_u16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self {
            Value::U16(v) => visitor.visit_u16(v),
            other => unimplemented!(
                "integer coercion from {} to u16 not implemented",
                other.type_name()
            ),
        }
    }

    fn deserialize_u32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self {
            Value::U32(v) => visitor.visit_u32(v),
            other => unimplemented!(
                "integer coercion from {} to u32 not implemented",
                other.type_name()
            ),
        }
    }

    fn deserialize_u64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self {
            Value::U64(v) => visitor.visit_u64(v),
            other => unimplemented!(
                "integer coercion from {} to u64 not implemented",
                other.type_name()
            ),
        }
    }

    fn deserialize_u128<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self {
            Value::U128(v) => visitor.visit_u128(v),
            other => unimplemented!(
                "integer coercion from {} to u128 not implemented",
                other.type_name()
            ),
        }
    }

    fn deserialize_f32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self {
            Value::F32(v) => visitor.visit_f32(v),
            other => unimplemented!(
                "float coercion from {} to f32 not implemented",
                other.type_name()
            ),
        }
    }

    fn deserialize_f64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self {
            Value::F64(v) => visitor.visit_f64(v),
            other => unimplemented!(
                "float coercion from {} to f64 not implemented",
                other.type_name()
            ),
        }
    }

    fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self {
            Value::Char(v) => visitor.visit_char(v),
            other => Err(Error::type_mismatch("char", other.type_name())),
        }
    }

    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self {
            Value::String(v) => visitor.visit_string(v),
            other => Err(Error::type_mismatch("string", other.type_name())),
        }
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self {
            Value::Bytes(v) => visitor.visit_byte_buf(v),
            other => Err(Error::type_mismatch("bytes", other.type_name())),
        }
    }

    fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self {
            Value::None | Value::Unit => visitor.visit_none(),
            Value::Some(v) => visitor.visit_some(*v),
            other => visitor.visit_some(other),
        }
    }

    fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self {
            Value::Unit | Value::UnitStruct { .. } | Value::None => visitor.visit_unit(),
            other => Err(Error::type_mismatch("unit", other.type_name())),
        }
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Error> {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Error> {
        match self {
            Value::NewtypeStruct { value, .. } => visitor.visit_newtype_struct(*value),
            other => visitor.visit_newtype_struct(other),
        }
    }

    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self {
            Value::Seq(v) => visitor.visit_seq(SeqDeserializer::new(v)),
            Value::Tuple(v) => visitor.visit_seq(SeqDeserializer::new(v)),
            Value::TupleStruct { fields, .. } => visitor.visit_seq(SeqDeserializer::new(fields)),
            other => Err(Error::type_mismatch("sequence", other.type_name())),
        }
    }

    fn deserialize_tuple<V: Visitor<'de>>(self, _len: usize, visitor: V) -> Result<V::Value, Error> {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Error> {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self {
            Value::Map(entries) => visitor.visit_map(MapDeserializer::new(entries)),
            Value::Struct { fields, .. } => visitor.visit_map(StructDeserializer::new(fields)),
            other => Err(Error::type_mismatch("map", other.type_name())),
        }
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error> {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error> {
        match self {
            Value::UnitVariant {
                enum_name,
                variant_index,
                variant,
            } => visitor.visit_enum(EnumDeserializer::UnitVariant {
                enum_name,
                variant_index,
                variant,
            }),
            Value::NewtypeVariant {
                enum_name,
                variant_index,
                variant,
                value,
            } => visitor.visit_enum(EnumDeserializer::NewtypeVariant {
                enum_name,
                variant_index,
                variant,
                value: *value,
            }),
            Value::TupleVariant {
                enum_name,
                variant_index,
                variant,
                fields,
            } => visitor.visit_enum(EnumDeserializer::TupleVariant {
                enum_name,
                variant_index,
                variant,
                fields,
            }),
            Value::StructVariant {
                enum_name,
                variant_index,
                variant,
                fields,
            } => visitor.visit_enum(EnumDeserializer::StructVariant {
                enum_name,
                variant_index,
                variant,
                fields,
            }),
            // String as unit variant
            Value::String(s) => visitor.visit_enum(StringEnumDeserializer(s)),
            other => Err(Error::type_mismatch("enum", other.type_name())),
        }
    }

    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_unit()
    }
}

// === Helper Deserializers ===

/// Deserializer for sequences.
struct SeqDeserializer {
    iter: std::vec::IntoIter<Value>,
}

impl SeqDeserializer {
    fn new(values: Vec<Value>) -> Self {
        SeqDeserializer {
            iter: values.into_iter(),
        }
    }
}

impl<'de> SeqAccess<'de> for SeqDeserializer {
    type Error = Error;

    fn next_element_seed<T: DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>, Error> {
        match self.iter.next() {
            Some(value) => seed.deserialize(value).map(Some),
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        let (lower, upper) = self.iter.size_hint();
        if Some(lower) == upper {
            Some(lower)
        } else {
            None
        }
    }
}

/// Deserializer for maps.
struct MapDeserializer {
    iter: std::vec::IntoIter<(Value, Value)>,
    next_value: Option<Value>,
}

impl MapDeserializer {
    fn new(entries: Vec<(Value, Value)>) -> Self {
        MapDeserializer {
            iter: entries.into_iter(),
            next_value: None,
        }
    }
}

impl<'de> MapAccess<'de> for MapDeserializer {
    type Error = Error;

    fn next_key_seed<K: DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, Error> {
        match self.iter.next() {
            Some((key, value)) => {
                self.next_value = Some(value);
                seed.deserialize(key).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(&mut self, seed: V) -> Result<V::Value, Error> {
        let value = self
            .next_value
            .take()
            .ok_or_else(|| Error::Message("next_value_seed called before next_key_seed".into()))?;
        seed.deserialize(value)
    }

    fn size_hint(&self) -> Option<usize> {
        let (lower, upper) = self.iter.size_hint();
        if Some(lower) == upper {
            Some(lower)
        } else {
            None
        }
    }
}

/// Deserializer for structs (preserves field names).
struct StructDeserializer {
    iter: std::vec::IntoIter<(&'static str, Value)>,
    next_value: Option<Value>,
}

impl StructDeserializer {
    fn new(fields: Vec<(&'static str, Value)>) -> Self {
        StructDeserializer {
            iter: fields.into_iter(),
            next_value: None,
        }
    }
}

impl<'de> MapAccess<'de> for StructDeserializer {
    type Error = Error;

    fn next_key_seed<K: DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, Error> {
        match self.iter.next() {
            Some((key, value)) => {
                self.next_value = Some(value);
                seed.deserialize(key.into_deserializer()).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(&mut self, seed: V) -> Result<V::Value, Error> {
        let value = self
            .next_value
            .take()
            .ok_or_else(|| Error::Message("next_value_seed called before next_key_seed".into()))?;
        seed.deserialize(value)
    }

    fn size_hint(&self) -> Option<usize> {
        let (lower, upper) = self.iter.size_hint();
        if Some(lower) == upper {
            Some(lower)
        } else {
            None
        }
    }
}

/// Deserializer for enums.
enum EnumDeserializer {
    UnitVariant {
        #[allow(dead_code)]
        enum_name: &'static str,
        #[allow(dead_code)]
        variant_index: u32,
        variant: &'static str,
    },
    NewtypeVariant {
        #[allow(dead_code)]
        enum_name: &'static str,
        #[allow(dead_code)]
        variant_index: u32,
        variant: &'static str,
        value: Value,
    },
    TupleVariant {
        #[allow(dead_code)]
        enum_name: &'static str,
        #[allow(dead_code)]
        variant_index: u32,
        variant: &'static str,
        fields: Vec<Value>,
    },
    StructVariant {
        #[allow(dead_code)]
        enum_name: &'static str,
        #[allow(dead_code)]
        variant_index: u32,
        variant: &'static str,
        fields: Vec<(&'static str, Value)>,
    },
}

impl<'de> EnumAccess<'de> for EnumDeserializer {
    type Error = Error;
    type Variant = VariantDeserializer;

    fn variant_seed<V: DeserializeSeed<'de>>(
        self,
        seed: V,
    ) -> Result<(V::Value, Self::Variant), Error> {
        match self {
            EnumDeserializer::UnitVariant { variant, .. } => {
                let val = seed.deserialize(variant.into_deserializer())?;
                Ok((val, VariantDeserializer::Unit))
            }
            EnumDeserializer::NewtypeVariant { variant, value, .. } => {
                let val = seed.deserialize(variant.into_deserializer())?;
                Ok((val, VariantDeserializer::Newtype(value)))
            }
            EnumDeserializer::TupleVariant {
                variant, fields, ..
            } => {
                let val = seed.deserialize(variant.into_deserializer())?;
                Ok((val, VariantDeserializer::Tuple(fields)))
            }
            EnumDeserializer::StructVariant {
                variant, fields, ..
            } => {
                let val = seed.deserialize(variant.into_deserializer())?;
                Ok((val, VariantDeserializer::Struct(fields)))
            }
        }
    }
}

/// Variant deserializer for different enum variant types.
enum VariantDeserializer {
    Unit,
    Newtype(Value),
    Tuple(Vec<Value>),
    Struct(Vec<(&'static str, Value)>),
}

impl<'de> VariantAccess<'de> for VariantDeserializer {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Error> {
        match self {
            VariantDeserializer::Unit => Ok(()),
            _ => Err(Error::type_mismatch("unit variant", "other variant")),
        }
    }

    fn newtype_variant_seed<T: DeserializeSeed<'de>>(self, seed: T) -> Result<T::Value, Error> {
        match self {
            VariantDeserializer::Newtype(value) => seed.deserialize(value),
            _ => Err(Error::type_mismatch("newtype variant", "other variant")),
        }
    }

    fn tuple_variant<V: Visitor<'de>>(self, _len: usize, visitor: V) -> Result<V::Value, Error> {
        match self {
            VariantDeserializer::Tuple(fields) => {
                visitor.visit_seq(SeqDeserializer::new(fields))
            }
            _ => Err(Error::type_mismatch("tuple variant", "other variant")),
        }
    }

    fn struct_variant<V: Visitor<'de>>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error> {
        match self {
            VariantDeserializer::Struct(fields) => {
                visitor.visit_map(StructDeserializer::new(fields))
            }
            _ => Err(Error::type_mismatch("struct variant", "other variant")),
        }
    }
}

/// Deserializer for string-as-enum (unit variant from string).
struct StringEnumDeserializer(String);

impl<'de> EnumAccess<'de> for StringEnumDeserializer {
    type Error = Error;
    type Variant = UnitVariantDeserializer;

    fn variant_seed<V: DeserializeSeed<'de>>(
        self,
        seed: V,
    ) -> Result<(V::Value, Self::Variant), Error> {
        let val = seed.deserialize(self.0.into_deserializer())?;
        Ok((val, UnitVariantDeserializer))
    }
}

/// Unit variant deserializer for string enums.
struct UnitVariantDeserializer;

impl<'de> VariantAccess<'de> for UnitVariantDeserializer {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Error> {
        Ok(())
    }

    fn newtype_variant_seed<T: DeserializeSeed<'de>>(self, _seed: T) -> Result<T::Value, Error> {
        Err(Error::type_mismatch("newtype variant", "unit variant"))
    }

    fn tuple_variant<V: Visitor<'de>>(self, _len: usize, _visitor: V) -> Result<V::Value, Error> {
        Err(Error::type_mismatch("tuple variant", "unit variant"))
    }

    fn struct_variant<V: Visitor<'de>>(
        self,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Error> {
        Err(Error::type_mismatch("struct variant", "unit variant"))
    }
}

// Implement IntoDeserializer for Value
impl<'de> IntoDeserializer<'de, Error> for Value {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}
