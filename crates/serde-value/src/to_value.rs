//! Serializer that produces `Value` from any `Serialize` type.
//!
//! This captures the complete serde data model including struct names,
//! field names, enum variants with indices, etc.

use crate::{Error, Value};
use serde::ser::{
    self, Serialize, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant,
    SerializeTuple, SerializeTupleStruct, SerializeTupleVariant,
};

/// Convert any `Serialize` type to `Value`.
///
/// This uses a custom serializer to capture all serde data model information,
/// preserving struct names, field names, enum variant names and indices.
pub fn to_value<T: Serialize>(value: T) -> Result<Value, Error> {
    value.serialize(ValueSerializer)
}

/// A serializer that produces `Value`.
pub struct ValueSerializer;

impl ser::Serializer for ValueSerializer {
    type Ok = Value;
    type Error = Error;
    type SerializeSeq = SeqSerializer;
    type SerializeTuple = TupleSerializer;
    type SerializeTupleStruct = TupleStructSerializer;
    type SerializeTupleVariant = TupleVariantSerializer;
    type SerializeMap = MapSerializer;
    type SerializeStruct = StructSerializer;
    type SerializeStructVariant = StructVariantSerializer;

    // === Primitives ===

    fn serialize_bool(self, v: bool) -> Result<Value, Error> {
        Ok(Value::Bool(v))
    }

    fn serialize_i8(self, v: i8) -> Result<Value, Error> {
        Ok(Value::I8(v))
    }

    fn serialize_i16(self, v: i16) -> Result<Value, Error> {
        Ok(Value::I16(v))
    }

    fn serialize_i32(self, v: i32) -> Result<Value, Error> {
        Ok(Value::I32(v))
    }

    fn serialize_i64(self, v: i64) -> Result<Value, Error> {
        Ok(Value::I64(v))
    }

    fn serialize_i128(self, v: i128) -> Result<Value, Error> {
        Ok(Value::I128(v))
    }

    fn serialize_u8(self, v: u8) -> Result<Value, Error> {
        Ok(Value::U8(v))
    }

    fn serialize_u16(self, v: u16) -> Result<Value, Error> {
        Ok(Value::U16(v))
    }

    fn serialize_u32(self, v: u32) -> Result<Value, Error> {
        Ok(Value::U32(v))
    }

    fn serialize_u64(self, v: u64) -> Result<Value, Error> {
        Ok(Value::U64(v))
    }

    fn serialize_u128(self, v: u128) -> Result<Value, Error> {
        Ok(Value::U128(v))
    }

    fn serialize_f32(self, v: f32) -> Result<Value, Error> {
        Ok(Value::F32(v))
    }

    fn serialize_f64(self, v: f64) -> Result<Value, Error> {
        Ok(Value::F64(v))
    }

    fn serialize_char(self, v: char) -> Result<Value, Error> {
        Ok(Value::Char(v))
    }

    // === String and Bytes ===

    fn serialize_str(self, v: &str) -> Result<Value, Error> {
        Ok(Value::String(v.to_owned()))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Value, Error> {
        Ok(Value::Bytes(v.to_vec()))
    }

    // === Option ===

    fn serialize_none(self) -> Result<Value, Error> {
        Ok(Value::None)
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Value, Error> {
        Ok(Value::Some(Box::new(to_value(value)?)))
    }

    // === Unit Types ===

    fn serialize_unit(self) -> Result<Value, Error> {
        Ok(Value::Unit)
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Value, Error> {
        Ok(Value::UnitStruct { name })
    }

    // === Newtype ===

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Value, Error> {
        Ok(Value::NewtypeStruct {
            name,
            value: Box::new(to_value(value)?),
        })
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        enum_name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Value, Error> {
        Ok(Value::NewtypeVariant {
            enum_name,
            variant_index,
            variant,
            value: Box::new(to_value(value)?),
        })
    }

    // === Sequences ===

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Error> {
        Ok(SeqSerializer {
            values: Vec::with_capacity(len.unwrap_or(0)),
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Error> {
        Ok(TupleSerializer {
            values: Vec::with_capacity(len),
        })
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Error> {
        Ok(TupleStructSerializer {
            name,
            fields: Vec::with_capacity(len),
        })
    }

    fn serialize_tuple_variant(
        self,
        enum_name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Error> {
        Ok(TupleVariantSerializer {
            enum_name,
            variant_index,
            variant,
            fields: Vec::with_capacity(len),
        })
    }

    // === Maps and Structs ===

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Error> {
        Ok(MapSerializer {
            entries: Vec::with_capacity(len.unwrap_or(0)),
            next_key: None,
        })
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Error> {
        Ok(StructSerializer {
            name,
            fields: Vec::with_capacity(len),
        })
    }

    fn serialize_struct_variant(
        self,
        enum_name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Error> {
        Ok(StructVariantSerializer {
            enum_name,
            variant_index,
            variant,
            fields: Vec::with_capacity(len),
        })
    }

    // === Unit Variant ===

    fn serialize_unit_variant(
        self,
        enum_name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Value, Error> {
        Ok(Value::UnitVariant {
            enum_name,
            variant_index,
            variant,
        })
    }
}

// === Helper Serializers ===

/// Serializer for sequences.
pub struct SeqSerializer {
    values: Vec<Value>,
}

impl SerializeSeq for SeqSerializer {
    type Ok = Value;
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Error> {
        self.values.push(to_value(value)?);
        Ok(())
    }

    fn end(self) -> Result<Value, Error> {
        Ok(Value::Seq(self.values))
    }
}

/// Serializer for tuples.
pub struct TupleSerializer {
    values: Vec<Value>,
}

impl SerializeTuple for TupleSerializer {
    type Ok = Value;
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Error> {
        self.values.push(to_value(value)?);
        Ok(())
    }

    fn end(self) -> Result<Value, Error> {
        Ok(Value::Tuple(self.values))
    }
}

/// Serializer for tuple structs.
pub struct TupleStructSerializer {
    name: &'static str,
    fields: Vec<Value>,
}

impl SerializeTupleStruct for TupleStructSerializer {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Error> {
        self.fields.push(to_value(value)?);
        Ok(())
    }

    fn end(self) -> Result<Value, Error> {
        Ok(Value::TupleStruct {
            name: self.name,
            fields: self.fields,
        })
    }
}

/// Serializer for tuple variants.
pub struct TupleVariantSerializer {
    enum_name: &'static str,
    variant_index: u32,
    variant: &'static str,
    fields: Vec<Value>,
}

impl SerializeTupleVariant for TupleVariantSerializer {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Error> {
        self.fields.push(to_value(value)?);
        Ok(())
    }

    fn end(self) -> Result<Value, Error> {
        Ok(Value::TupleVariant {
            enum_name: self.enum_name,
            variant_index: self.variant_index,
            variant: self.variant,
            fields: self.fields,
        })
    }
}

/// Serializer for maps.
pub struct MapSerializer {
    entries: Vec<(Value, Value)>,
    next_key: Option<Value>,
}

impl SerializeMap for MapSerializer {
    type Ok = Value;
    type Error = Error;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), Error> {
        self.next_key = Some(to_value(key)?);
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Error> {
        let key = self
            .next_key
            .take()
            .ok_or_else(|| Error::Message("serialize_value called before serialize_key".into()))?;
        self.entries.push((key, to_value(value)?));
        Ok(())
    }

    fn end(self) -> Result<Value, Error> {
        Ok(Value::Map(self.entries))
    }
}

/// Serializer for structs.
pub struct StructSerializer {
    name: &'static str,
    fields: Vec<(&'static str, Value)>,
}

impl SerializeStruct for StructSerializer {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        self.fields.push((key, to_value(value)?));
        Ok(())
    }

    fn end(self) -> Result<Value, Error> {
        Ok(Value::Struct {
            name: self.name,
            fields: self.fields,
        })
    }
}

/// Serializer for struct variants.
pub struct StructVariantSerializer {
    enum_name: &'static str,
    variant_index: u32,
    variant: &'static str,
    fields: Vec<(&'static str, Value)>,
}

impl SerializeStructVariant for StructVariantSerializer {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        self.fields.push((key, to_value(value)?));
        Ok(())
    }

    fn end(self) -> Result<Value, Error> {
        Ok(Value::StructVariant {
            enum_name: self.enum_name,
            variant_index: self.variant_index,
            variant: self.variant,
            fields: self.fields,
        })
    }
}
