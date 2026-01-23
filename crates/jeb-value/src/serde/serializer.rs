//! Serializer serializing arbitrary `Serialize` values into our `Value` type.
use {
    crate::{
        Boolean,
        Bytes,
        Null,
        Number,
        String,
        Value,
        serde::SerdeError,
    },
    indexmap::IndexMap,
    serde::{
        Serialize,
        ser,
    },
};
pub struct Serializer;
impl ser::Serializer for Serializer {
    type Error = SerdeError;
    type Ok = Value;
    type SerializeMap = SerializeMap;
    type SerializeSeq = SerializeVec;
    type SerializeStruct = SerializeMap;
    type SerializeStructVariant = SerializeStructVariant;
    type SerializeTuple = SerializeVec;
    type SerializeTupleStruct = SerializeVec;
    type SerializeTupleVariant = SerializeTupleVariant;

    fn serialize_bool(self, v: bool) -> Result<Value, SerdeError> {
        Ok(Value::Boolean(Boolean::from(v)))
    }

    fn serialize_i8(self, v: i8) -> Result<Value, SerdeError> {
        Ok(Value::Number(Number::from(v)))
    }

    fn serialize_i16(self, v: i16) -> Result<Value, SerdeError> {
        Ok(Value::Number(Number::from(v)))
    }

    fn serialize_i32(self, v: i32) -> Result<Value, SerdeError> {
        Ok(Value::Number(Number::from(v)))
    }

    fn serialize_i64(self, v: i64) -> Result<Value, SerdeError> {
        Ok(Value::from(v))
    }

    fn serialize_i128(self, v: i128) -> Result<Value, SerdeError> {
        Ok(Value::from(v))
    }

    fn serialize_u8(self, v: u8) -> Result<Value, SerdeError> {
        Ok(Value::Number(Number::from(v)))
    }

    fn serialize_u16(self, v: u16) -> Result<Value, SerdeError> {
        Ok(Value::Number(Number::from(v)))
    }

    fn serialize_u32(self, v: u32) -> Result<Value, SerdeError> {
        Ok(Value::Number(Number::from(v)))
    }

    fn serialize_u64(self, v: u64) -> Result<Value, SerdeError> {
        Ok(Value::from(v))
    }

    fn serialize_u128(self, v: u128) -> Result<Value, SerdeError> {
        Ok(Value::from(v))
    }

    fn serialize_f32(self, v: f32) -> Result<Value, SerdeError> {
        Ok(Value::from(v))
    }

    fn serialize_f64(self, v: f64) -> Result<Value, SerdeError> {
        Ok(Value::from(v))
    }

    fn serialize_char(self, v: char) -> Result<Value, SerdeError> {
        Ok(Value::String(String::from(v.to_string())))
    }

    fn serialize_str(self, v: &str) -> Result<Value, SerdeError> {
        Ok(Value::String(String::from(v.to_string())))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Value, SerdeError> {
        Ok(Value::Bytes(Bytes::from(v.to_vec())))
    }

    fn serialize_none(self) -> Result<Value, SerdeError> {
        Ok(Value::Null(Null::new()))
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Value, SerdeError> {
        let inner = to_value(value)?;
        let mut map = IndexMap::new();
        map.insert(String::from("Some".to_string()), inner);
        Ok(Value::StringMap(map))
    }

    fn serialize_unit(self) -> Result<Value, SerdeError> {
        Ok(Value::Null(Null::new()))
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Value, SerdeError> {
        Ok(Value::Null(Null::new()))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Value, SerdeError> {
        Ok(Value::String(String::from(variant.to_string())))
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Value, SerdeError> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Value, SerdeError> {
        let inner = to_value(value)?;
        let mut map = IndexMap::new();
        map.insert(String::from(variant.to_string()), inner);
        Ok(Value::StringMap(map))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, SerdeError> {
        Ok(SerializeVec {
            vec: Vec::with_capacity(len.unwrap_or(0)),
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, SerdeError> {
        Ok(SerializeVec {
            vec: Vec::with_capacity(len),
        })
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, SerdeError> {
        Ok(SerializeVec {
            vec: Vec::with_capacity(len),
        })
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, SerdeError> {
        Ok(SerializeTupleVariant {
            variant: variant.to_string(),
            vec: Vec::with_capacity(len),
        })
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, SerdeError> {
        Ok(SerializeMap {
            entries: Vec::with_capacity(len.unwrap_or(0)),
            next_key: None,
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, SerdeError> {
        Ok(SerializeMap {
            entries: Vec::with_capacity(len),
            next_key: None,
        })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, SerdeError> {
        Ok(SerializeStructVariant {
            variant: variant.to_string(),
            map: SerializeMap {
                entries: Vec::with_capacity(len),
                next_key: None,
            },
        })
    }

    fn is_human_readable(&self) -> bool {
        true
    }
}
pub struct SerializeVec {
    vec: Vec<Value>,
}
impl ser::SerializeSeq for SerializeVec {
    type Error = SerdeError;
    type Ok = Value;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), SerdeError> {
        self.vec.push(to_value(value)?);
        Ok(())
    }

    fn end(self) -> Result<Value, SerdeError> {
        Ok(Value::Array(self.vec))
    }
}
impl ser::SerializeTuple for SerializeVec {
    type Error = SerdeError;
    type Ok = Value;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), SerdeError> {
        self.vec.push(to_value(value)?);
        Ok(())
    }

    fn end(self) -> Result<Value, SerdeError> {
        Ok(Value::Array(self.vec))
    }
}
impl ser::SerializeTupleStruct for SerializeVec {
    type Error = SerdeError;
    type Ok = Value;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), SerdeError> {
        self.vec.push(to_value(value)?);
        Ok(())
    }

    fn end(self) -> Result<Value, SerdeError> {
        Ok(Value::Array(self.vec))
    }
}
pub struct SerializeTupleVariant {
    variant: std::string::String,
    vec: Vec<Value>,
}
impl ser::SerializeTupleVariant for SerializeTupleVariant {
    type Error = SerdeError;
    type Ok = Value;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), SerdeError> {
        self.vec.push(to_value(value)?);
        Ok(())
    }

    fn end(self) -> Result<Value, SerdeError> {
        let mut map = IndexMap::new();
        map.insert(String::from(self.variant.as_str()), Value::Array(self.vec));
        Ok(Value::StringMap(map))
    }
}
pub struct SerializeMap {
    entries: Vec<(MapKey, Value)>,
    next_key: Option<MapKey>,
}
#[derive(Debug)]
enum MapKey {
    String(String),
    Bytes(Bytes),
    Complex(Value),
}
impl ser::SerializeMap for SerializeMap {
    type Error = SerdeError;
    type Ok = Value;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), SerdeError> {
        self.next_key = Some(key.serialize(MapKeySerializer)?);
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), SerdeError> {
        let key = self.next_key.take().ok_or_else(|| {
            SerdeError::Message("serialize_value called before serialize_key".into())
        })?;
        self.entries.push((key, to_value(value)?));
        Ok(())
    }

    fn end(self) -> Result<Value, SerdeError> {
        if self.entries.is_empty() {
            return Ok(Value::Array(Vec::new()));
        }
        let mut all_text = true;
        let mut all_bytes = true;
        for (key, _) in &self.entries {
            match key {
                MapKey::String(_) => all_bytes = false,
                MapKey::Bytes(_) => all_text = false,
                MapKey::Complex(_) => {
                    all_text = false;
                    all_bytes = false;
                }
            }
        }
        if all_text {
            let mut map = IndexMap::new();
            for (key, value) in self.entries {
                match key {
                    MapKey::String(t) => {
                        map.insert(t, value);
                    }
                    _ => unreachable!(),
                }
            }
            Ok(Value::StringMap(map))
        } else if all_bytes {
            let mut map = IndexMap::new();
            for (key, value) in self.entries {
                match key {
                    MapKey::Bytes(b) => {
                        map.insert(b, value);
                    }
                    _ => unreachable!(),
                }
            }
            Ok(Value::BytesMap(map))
        } else {
            let pairs = self
                .entries
                .into_iter()
                .map(|(key, value)| {
                    let key_value = match key {
                        MapKey::String(t) => Value::String(t),
                        MapKey::Bytes(b) => Value::Bytes(b),
                        MapKey::Complex(v) => v,
                    };
                    Value::Array(vec![key_value, value])
                })
                .collect();
            Ok(Value::Array(pairs))
        }
    }
}
impl ser::SerializeStruct for SerializeMap {
    type Error = SerdeError;
    type Ok = Value;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), SerdeError> {
        self.entries.push((
            MapKey::String(String::from(key.to_string())),
            to_value(value)?,
        ));
        Ok(())
    }

    fn end(self) -> Result<Value, SerdeError> {
        ser::SerializeMap::end(self)
    }
}
pub struct SerializeStructVariant {
    variant: std::string::String,
    map: SerializeMap,
}
impl ser::SerializeStructVariant for SerializeStructVariant {
    type Error = SerdeError;
    type Ok = Value;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), SerdeError> {
        ser::SerializeStruct::serialize_field(&mut self.map, key, value)
    }

    fn end(self) -> Result<Value, SerdeError> {
        let fields = ser::SerializeMap::end(self.map)?;
        let mut map = IndexMap::new();
        map.insert(String::from(self.variant.as_str()), fields);
        Ok(Value::StringMap(map))
    }
}
struct MapKeySerializer;
struct MapKeySeq {
    elements: Vec<Value>,
}
impl ser::SerializeSeq for MapKeySeq {
    type Error = SerdeError;
    type Ok = MapKey;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), SerdeError> {
        self.elements.push(to_value(value)?);
        Ok(())
    }

    fn end(self) -> Result<MapKey, SerdeError> {
        Ok(MapKey::Complex(Value::Array(self.elements)))
    }
}
impl ser::SerializeTuple for MapKeySeq {
    type Error = SerdeError;
    type Ok = MapKey;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), SerdeError> {
        self.elements.push(to_value(value)?);
        Ok(())
    }

    fn end(self) -> Result<MapKey, SerdeError> {
        Ok(MapKey::Complex(Value::Array(self.elements)))
    }
}
impl ser::SerializeTupleStruct for MapKeySeq {
    type Error = SerdeError;
    type Ok = MapKey;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), SerdeError> {
        self.elements.push(to_value(value)?);
        Ok(())
    }

    fn end(self) -> Result<MapKey, SerdeError> {
        Ok(MapKey::Complex(Value::Array(self.elements)))
    }
}
impl ser::SerializeTupleVariant for MapKeySeq {
    type Error = SerdeError;
    type Ok = MapKey;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), SerdeError> {
        self.elements.push(to_value(value)?);
        Ok(())
    }

    fn end(self) -> Result<MapKey, SerdeError> {
        Ok(MapKey::Complex(Value::Array(self.elements)))
    }
}
struct MapKeyStruct {
    fields: IndexMap<String, Value>,
}
impl ser::SerializeStruct for MapKeyStruct {
    type Error = SerdeError;
    type Ok = MapKey;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), SerdeError> {
        self.fields
            .insert(String::from(key.to_string()), to_value(value)?);
        Ok(())
    }

    fn end(self) -> Result<MapKey, SerdeError> {
        Ok(MapKey::Complex(Value::StringMap(self.fields)))
    }
}
impl ser::SerializeStructVariant for MapKeyStruct {
    type Error = SerdeError;
    type Ok = MapKey;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), SerdeError> {
        self.fields
            .insert(String::from(key.to_string()), to_value(value)?);
        Ok(())
    }

    fn end(self) -> Result<MapKey, SerdeError> {
        Ok(MapKey::Complex(Value::StringMap(self.fields)))
    }
}
struct MapKeyMap {
    entries: Vec<(MapKey, Value)>,
    next_key: Option<MapKey>,
}
impl ser::SerializeMap for MapKeyMap {
    type Error = SerdeError;
    type Ok = MapKey;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), SerdeError> {
        self.next_key = Some(key.serialize(MapKeySerializer)?);
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), SerdeError> {
        let key = self.next_key.take().ok_or_else(|| {
            SerdeError::Message("serialize_value called before serialize_key".into())
        })?;
        self.entries.push((key, to_value(value)?));
        Ok(())
    }

    fn end(self) -> Result<MapKey, SerdeError> {
        if self.entries.is_empty() {
            return Ok(MapKey::Complex(Value::Array(Vec::new())));
        }
        let mut all_text = true;
        let mut all_bytes = true;
        for (key, _) in &self.entries {
            match key {
                MapKey::String(_) => all_bytes = false,
                MapKey::Bytes(_) => all_text = false,
                MapKey::Complex(_) => {
                    all_text = false;
                    all_bytes = false;
                }
            }
        }
        if all_text {
            let mut map = IndexMap::new();
            for (key, value) in self.entries {
                match key {
                    MapKey::String(t) => {
                        map.insert(t, value);
                    }
                    _ => unreachable!(),
                }
            }
            Ok(MapKey::Complex(Value::StringMap(map)))
        } else if all_bytes {
            let mut map = IndexMap::new();
            for (key, value) in self.entries {
                match key {
                    MapKey::Bytes(b) => {
                        map.insert(b, value);
                    }
                    _ => unreachable!(),
                }
            }
            Ok(MapKey::Complex(Value::BytesMap(map)))
        } else {
            let pairs = self
                .entries
                .into_iter()
                .map(|(key, value)| {
                    let key_value = match key {
                        MapKey::String(t) => Value::String(t),
                        MapKey::Bytes(b) => Value::Bytes(b),
                        MapKey::Complex(v) => v,
                    };
                    Value::Array(vec![key_value, value])
                })
                .collect();
            Ok(MapKey::Complex(Value::Array(pairs)))
        }
    }
}
impl ser::Serializer for MapKeySerializer {
    type Error = SerdeError;
    type Ok = MapKey;
    type SerializeMap = MapKeyMap;
    type SerializeSeq = MapKeySeq;
    type SerializeStruct = MapKeyStruct;
    type SerializeStructVariant = MapKeyStruct;
    type SerializeTuple = MapKeySeq;
    type SerializeTupleStruct = MapKeySeq;
    type SerializeTupleVariant = MapKeySeq;

    fn serialize_bool(self, v: bool) -> Result<MapKey, SerdeError> {
        Ok(MapKey::Complex(Value::Boolean(Boolean::from(v))))
    }

    fn serialize_i8(self, v: i8) -> Result<MapKey, SerdeError> {
        Ok(MapKey::Complex(Value::Number(Number::from(v))))
    }

    fn serialize_i16(self, v: i16) -> Result<MapKey, SerdeError> {
        Ok(MapKey::Complex(Value::Number(Number::from(v))))
    }

    fn serialize_i32(self, v: i32) -> Result<MapKey, SerdeError> {
        Ok(MapKey::Complex(Value::Number(Number::from(v))))
    }

    fn serialize_i64(self, v: i64) -> Result<MapKey, SerdeError> {
        Ok(MapKey::Complex(Value::from(v)))
    }

    fn serialize_i128(self, v: i128) -> Result<MapKey, SerdeError> {
        Ok(MapKey::Complex(Value::from(v)))
    }

    fn serialize_u8(self, v: u8) -> Result<MapKey, SerdeError> {
        Ok(MapKey::Complex(Value::Number(Number::from(v))))
    }

    fn serialize_u16(self, v: u16) -> Result<MapKey, SerdeError> {
        Ok(MapKey::Complex(Value::Number(Number::from(v))))
    }

    fn serialize_u32(self, v: u32) -> Result<MapKey, SerdeError> {
        Ok(MapKey::Complex(Value::Number(Number::from(v))))
    }

    fn serialize_u64(self, v: u64) -> Result<MapKey, SerdeError> {
        Ok(MapKey::Complex(Value::from(v)))
    }

    fn serialize_u128(self, v: u128) -> Result<MapKey, SerdeError> {
        Ok(MapKey::Complex(Serializer.serialize_u128(v)?))
    }

    fn serialize_f32(self, v: f32) -> Result<MapKey, SerdeError> {
        Ok(MapKey::Complex(Serializer.serialize_f32(v)?))
    }

    fn serialize_f64(self, v: f64) -> Result<MapKey, SerdeError> {
        Ok(MapKey::Complex(Serializer.serialize_f64(v)?))
    }

    fn serialize_char(self, v: char) -> Result<MapKey, SerdeError> {
        Ok(MapKey::String(String::from(v.to_string())))
    }

    fn serialize_str(self, v: &str) -> Result<MapKey, SerdeError> {
        Ok(MapKey::String(String::from(v.to_string())))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<MapKey, SerdeError> {
        Ok(MapKey::Bytes(Bytes::from(v.to_vec())))
    }

    fn serialize_none(self) -> Result<MapKey, SerdeError> {
        Ok(MapKey::Complex(Value::Null(Null::new())))
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<MapKey, SerdeError> {
        Ok(MapKey::Complex(Serializer.serialize_some(value)?))
    }

    fn serialize_unit(self) -> Result<MapKey, SerdeError> {
        Ok(MapKey::Complex(Value::Null(Null::new())))
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<MapKey, SerdeError> {
        Ok(MapKey::Complex(Value::Null(Null::new())))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<MapKey, SerdeError> {
        Ok(MapKey::String(String::from(variant.to_string())))
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<MapKey, SerdeError> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<MapKey, SerdeError> {
        let inner = to_value(value)?;
        let mut map = IndexMap::new();
        map.insert(String::from(variant.to_string()), inner);
        Ok(MapKey::Complex(Value::StringMap(map)))
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, SerdeError> {
        Ok(MapKeySeq {
            elements: Vec::new(),
        })
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, SerdeError> {
        Ok(MapKeySeq {
            elements: Vec::new(),
        })
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, SerdeError> {
        Ok(MapKeySeq {
            elements: Vec::new(),
        })
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, SerdeError> {
        Ok(MapKeySeq {
            elements: Vec::new(),
        })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, SerdeError> {
        Ok(MapKeyMap {
            entries: Vec::new(),
            next_key: None,
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, SerdeError> {
        Ok(MapKeyStruct {
            fields: IndexMap::new(),
        })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, SerdeError> {
        Ok(MapKeyStruct {
            fields: IndexMap::new(),
        })
    }
}
pub fn to_value<T: Serialize>(value: T) -> Result<Value, SerdeError> {
    value.serialize(Serializer)
}
