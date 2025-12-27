use {
    crate::{serde::Error, Bytes, Float, Text, Value},
    indexmap::IndexMap,
    serde::{ser, Serialize},
};

pub struct Serializer;

impl ser::Serializer for Serializer {
    type Ok = Value;
    type Error = Error;

    type SerializeSeq = SerializeVec;
    type SerializeTuple = SerializeVec;
    type SerializeTupleStruct = SerializeVec;
    type SerializeTupleVariant = SerializeTupleVariant;
    type SerializeMap = SerializeMap;
    type SerializeStruct = SerializeMap;
    type SerializeStructVariant = SerializeStructVariant;

    fn serialize_bool(self, v: bool) -> Result<Value, Error> {
        Ok(Value::Bool(v))
    }

    fn serialize_i8(self, v: i8) -> Result<Value, Error> {
        Ok(Value::Signed(v as i64))
    }

    fn serialize_i16(self, v: i16) -> Result<Value, Error> {
        Ok(Value::Signed(v as i64))
    }

    fn serialize_i32(self, v: i32) -> Result<Value, Error> {
        Ok(Value::Signed(v as i64))
    }

    fn serialize_i64(self, v: i64) -> Result<Value, Error> {
        Ok(Value::Signed(v))
    }

    fn serialize_i128(self, v: i128) -> Result<Value, Error> {
        if let Ok(i) = i64::try_from(v) {
            Ok(Value::Signed(i))
        } else {
            Ok(Value::Bytes(Bytes::from(v.to_be_bytes().to_vec())))
        }
    }

    fn serialize_u8(self, v: u8) -> Result<Value, Error> {
        Ok(Value::Unsigned(v as u64))
    }

    fn serialize_u16(self, v: u16) -> Result<Value, Error> {
        Ok(Value::Unsigned(v as u64))
    }

    fn serialize_u32(self, v: u32) -> Result<Value, Error> {
        Ok(Value::Unsigned(v as u64))
    }

    fn serialize_u64(self, v: u64) -> Result<Value, Error> {
        Ok(Value::Unsigned(v))
    }

    fn serialize_u128(self, v: u128) -> Result<Value, Error> {
        if let Ok(u) = u64::try_from(v) {
            Ok(Value::Unsigned(u))
        } else {
            Ok(Value::Bytes(Bytes::from(v.to_be_bytes().to_vec())))
        }
    }

    fn serialize_f32(self, v: f32) -> Result<Value, Error> {
        if v.is_finite() {
            Ok(Value::Float(Float::new(v as f64).expect("f32 is_finite check guarantees Float::new success")))
        } else {
            Ok(Value::Bytes(Bytes::from(v.to_be_bytes().to_vec())))
        }
    }

    fn serialize_f64(self, v: f64) -> Result<Value, Error> {
        if v.is_finite() {
            Ok(Value::Float(Float::new(v).expect("f64 is_finite check guarantees Float::new success")))
        } else {
            Ok(Value::Bytes(Bytes::from(v.to_be_bytes().to_vec())))
        }
    }

    fn serialize_char(self, v: char) -> Result<Value, Error> {
        Ok(Value::Text(Text::from(v.to_string())))
    }

    fn serialize_str(self, v: &str) -> Result<Value, Error> {
        Ok(Value::Text(Text::from(v.to_string())))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Value, Error> {
        Ok(Value::Bytes(Bytes::from(v.to_vec())))
    }

    fn serialize_none(self) -> Result<Value, Error> {
        Ok(Value::Null)
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Value, Error> {
        let inner = to_value(value)?;
        let mut map = IndexMap::new();
        map.insert(Text::from("Some".to_string()), inner);
        Ok(Value::TextMap(map))
    }

    fn serialize_unit(self) -> Result<Value, Error> {
        Ok(Value::Null)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Value, Error> {
        Ok(Value::Null)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Value, Error> {
        Ok(Value::Text(Text::from(variant.to_string())))
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Value, Error> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Value, Error> {
        let inner = to_value(value)?;
        let mut map = IndexMap::new();
        map.insert(Text::from(variant.to_string()), inner);
        Ok(Value::TextMap(map))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Error> {
        Ok(SerializeVec {
            vec: Vec::with_capacity(len.unwrap_or(0)),
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Error> {
        Ok(SerializeVec {
            vec: Vec::with_capacity(len),
        })
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Error> {
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
    ) -> Result<Self::SerializeTupleVariant, Error> {
        Ok(SerializeTupleVariant {
            variant: variant.to_string(),
            vec: Vec::with_capacity(len),
        })
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Error> {
        Ok(SerializeMap {
            entries: Vec::with_capacity(len.unwrap_or(0)),
            next_key: None,
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Error> {
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
    ) -> Result<Self::SerializeStructVariant, Error> {
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
    type Ok = Value;
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Error> {
        self.vec.push(to_value(value)?);
        Ok(())
    }

    fn end(self) -> Result<Value, Error> {
        Ok(Value::Array(self.vec))
    }
}

impl ser::SerializeTuple for SerializeVec {
    type Ok = Value;
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Error> {
        self.vec.push(to_value(value)?);
        Ok(())
    }

    fn end(self) -> Result<Value, Error> {
        Ok(Value::Array(self.vec))
    }
}

impl ser::SerializeTupleStruct for SerializeVec {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Error> {
        self.vec.push(to_value(value)?);
        Ok(())
    }

    fn end(self) -> Result<Value, Error> {
        Ok(Value::Array(self.vec))
    }
}

pub struct SerializeTupleVariant {
    variant: String,
    vec: Vec<Value>,
}

impl ser::SerializeTupleVariant for SerializeTupleVariant {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Error> {
        self.vec.push(to_value(value)?);
        Ok(())
    }

    fn end(self) -> Result<Value, Error> {
        let mut map = IndexMap::new();
        map.insert(Text::from(self.variant), Value::Array(self.vec));
        Ok(Value::TextMap(map))
    }
}

pub struct SerializeMap {
    entries: Vec<(MapKey, Value)>,
    next_key: Option<MapKey>,
}

#[derive(Debug)]
enum MapKey {
    Text(Text),
    Bytes(Bytes),
    Complex(Value),
}

impl ser::SerializeMap for SerializeMap {
    type Ok = Value;
    type Error = Error;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), Error> {
        self.next_key = Some(key.serialize(MapKeySerializer)?);
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
        if self.entries.is_empty() {
            return Ok(Value::Array(Vec::new()));
        }

        // Analyze keys to determine representation
        let mut all_text = true;
        let mut all_bytes = true;

        for (key, _) in &self.entries {
            match key {
                MapKey::Text(_) => all_bytes = false,
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
                    MapKey::Text(t) => {
                        map.insert(t, value);
                    }
                    _ => unreachable!(),
                }
            }
            Ok(Value::TextMap(map))
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
            // Mixed or complex keys: use array of pairs
            let pairs = self
                .entries
                .into_iter()
                .map(|(key, value)| {
                    let key_value = match key {
                        MapKey::Text(t) => Value::Text(t),
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
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        self.entries.push((
            MapKey::Text(Text::from(key.to_string())),
            to_value(value)?,
        ));
        Ok(())
    }

    fn end(self) -> Result<Value, Error> {
        ser::SerializeMap::end(self)
    }
}

pub struct SerializeStructVariant {
    variant: String,
    map: SerializeMap,
}

impl ser::SerializeStructVariant for SerializeStructVariant {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        ser::SerializeStruct::serialize_field(&mut self.map, key, value)
    }

    fn end(self) -> Result<Value, Error> {
        let fields = ser::SerializeMap::end(self.map)?;
        let mut map = IndexMap::new();
        map.insert(Text::from(self.variant), fields);
        Ok(Value::TextMap(map))
    }
}

struct MapKeySerializer;

// Helper for serializing compound types as map keys
struct MapKeySeq {
    elements: Vec<Value>,
}

impl ser::SerializeSeq for MapKeySeq {
    type Ok = MapKey;
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Error> {
        self.elements.push(to_value(value)?);
        Ok(())
    }

    fn end(self) -> Result<MapKey, Error> {
        Ok(MapKey::Complex(Value::Array(self.elements)))
    }
}

impl ser::SerializeTuple for MapKeySeq {
    type Ok = MapKey;
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Error> {
        self.elements.push(to_value(value)?);
        Ok(())
    }

    fn end(self) -> Result<MapKey, Error> {
        Ok(MapKey::Complex(Value::Array(self.elements)))
    }
}

impl ser::SerializeTupleStruct for MapKeySeq {
    type Ok = MapKey;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Error> {
        self.elements.push(to_value(value)?);
        Ok(())
    }

    fn end(self) -> Result<MapKey, Error> {
        Ok(MapKey::Complex(Value::Array(self.elements)))
    }
}

impl ser::SerializeTupleVariant for MapKeySeq {
    type Ok = MapKey;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Error> {
        self.elements.push(to_value(value)?);
        Ok(())
    }

    fn end(self) -> Result<MapKey, Error> {
        Ok(MapKey::Complex(Value::Array(self.elements)))
    }
}

struct MapKeyStruct {
    fields: IndexMap<Text, Value>,
}

impl ser::SerializeStruct for MapKeyStruct {
    type Ok = MapKey;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        self.fields.insert(Text::from(key.to_string()), to_value(value)?);
        Ok(())
    }

    fn end(self) -> Result<MapKey, Error> {
        Ok(MapKey::Complex(Value::TextMap(self.fields)))
    }
}

impl ser::SerializeStructVariant for MapKeyStruct {
    type Ok = MapKey;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        self.fields.insert(Text::from(key.to_string()), to_value(value)?);
        Ok(())
    }

    fn end(self) -> Result<MapKey, Error> {
        Ok(MapKey::Complex(Value::TextMap(self.fields)))
    }
}

struct MapKeyMap {
    entries: Vec<(MapKey, Value)>,
    next_key: Option<MapKey>,
}

impl ser::SerializeMap for MapKeyMap {
    type Ok = MapKey;
    type Error = Error;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), Error> {
        self.next_key = Some(key.serialize(MapKeySerializer)?);
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

    fn end(self) -> Result<MapKey, Error> {
        if self.entries.is_empty() {
            return Ok(MapKey::Complex(Value::Array(Vec::new())));
        }

        // Analyze keys to determine representation
        let mut all_text = true;
        let mut all_bytes = true;

        for (key, _) in &self.entries {
            match key {
                MapKey::Text(_) => all_bytes = false,
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
                    MapKey::Text(t) => {
                        map.insert(t, value);
                    }
                    _ => unreachable!(),
                }
            }
            Ok(MapKey::Complex(Value::TextMap(map)))
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
            // Mixed or complex keys: use array of pairs
            let pairs = self
                .entries
                .into_iter()
                .map(|(key, value)| {
                    let key_value = match key {
                        MapKey::Text(t) => Value::Text(t),
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
    type Ok = MapKey;
    type Error = Error;

    type SerializeSeq = MapKeySeq;
    type SerializeTuple = MapKeySeq;
    type SerializeTupleStruct = MapKeySeq;
    type SerializeTupleVariant = MapKeySeq;
    type SerializeMap = MapKeyMap;
    type SerializeStruct = MapKeyStruct;
    type SerializeStructVariant = MapKeyStruct;

    fn serialize_bool(self, v: bool) -> Result<MapKey, Error> {
        Ok(MapKey::Complex(Value::Bool(v)))
    }

    fn serialize_i8(self, v: i8) -> Result<MapKey, Error> {
        Ok(MapKey::Complex(Value::Signed(v as i64)))
    }

    fn serialize_i16(self, v: i16) -> Result<MapKey, Error> {
        Ok(MapKey::Complex(Value::Signed(v as i64)))
    }

    fn serialize_i32(self, v: i32) -> Result<MapKey, Error> {
        Ok(MapKey::Complex(Value::Signed(v as i64)))
    }

    fn serialize_i64(self, v: i64) -> Result<MapKey, Error> {
        Ok(MapKey::Complex(Value::Signed(v)))
    }

    fn serialize_i128(self, v: i128) -> Result<MapKey, Error> {
        Ok(MapKey::Complex(Serializer.serialize_i128(v)?))
    }

    fn serialize_u8(self, v: u8) -> Result<MapKey, Error> {
        Ok(MapKey::Complex(Value::Unsigned(v as u64)))
    }

    fn serialize_u16(self, v: u16) -> Result<MapKey, Error> {
        Ok(MapKey::Complex(Value::Unsigned(v as u64)))
    }

    fn serialize_u32(self, v: u32) -> Result<MapKey, Error> {
        Ok(MapKey::Complex(Value::Unsigned(v as u64)))
    }

    fn serialize_u64(self, v: u64) -> Result<MapKey, Error> {
        Ok(MapKey::Complex(Value::Unsigned(v)))
    }

    fn serialize_u128(self, v: u128) -> Result<MapKey, Error> {
        Ok(MapKey::Complex(Serializer.serialize_u128(v)?))
    }

    fn serialize_f32(self, v: f32) -> Result<MapKey, Error> {
        Ok(MapKey::Complex(Serializer.serialize_f32(v)?))
    }

    fn serialize_f64(self, v: f64) -> Result<MapKey, Error> {
        Ok(MapKey::Complex(Serializer.serialize_f64(v)?))
    }

    fn serialize_char(self, v: char) -> Result<MapKey, Error> {
        Ok(MapKey::Text(Text::from(v.to_string())))
    }

    fn serialize_str(self, v: &str) -> Result<MapKey, Error> {
        Ok(MapKey::Text(Text::from(v.to_string())))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<MapKey, Error> {
        Ok(MapKey::Bytes(Bytes::from(v.to_vec())))
    }

    fn serialize_none(self) -> Result<MapKey, Error> {
        Ok(MapKey::Complex(Value::Null))
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<MapKey, Error> {
        Ok(MapKey::Complex(Serializer.serialize_some(value)?))
    }

    fn serialize_unit(self) -> Result<MapKey, Error> {
        Ok(MapKey::Complex(Value::Null))
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<MapKey, Error> {
        Ok(MapKey::Complex(Value::Null))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<MapKey, Error> {
        Ok(MapKey::Text(Text::from(variant.to_string())))
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<MapKey, Error> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<MapKey, Error> {
        let inner = to_value(value)?;
        let mut map = IndexMap::new();
        map.insert(Text::from(variant.to_string()), inner);
        Ok(MapKey::Complex(Value::TextMap(map)))
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Error> {
        Ok(MapKeySeq {
            elements: Vec::new(),
        })
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Error> {
        Ok(MapKeySeq {
            elements: Vec::new(),
        })
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Error> {
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
    ) -> Result<Self::SerializeTupleVariant, Error> {
        Ok(MapKeySeq {
            elements: Vec::new(),
        })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Error> {
        Ok(MapKeyMap {
            entries: Vec::new(),
            next_key: None,
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Error> {
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
    ) -> Result<Self::SerializeStructVariant, Error> {
        Ok(MapKeyStruct {
            fields: IndexMap::new(),
        })
    }
}

pub fn to_value<T: Serialize>(value: T) -> Result<Value, Error> {
    value.serialize(Serializer)
}
