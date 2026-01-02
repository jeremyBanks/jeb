//! Non-derived `Deserialize` implementations for our `Value` types, to allow
//! them to be deserialized by arbitrary serde `Deserializer`s.
use {
    crate::{
        Bytes,
        Float,
        Text,
        Value,
    },
    indexmap::IndexMap,
    serde::de::{
        self,
        Visitor,
    },
};
impl<'de> serde::Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(ValueVisitor)
    }
}
struct ValueVisitor;
impl<'de> Visitor<'de> for ValueVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("any valid value")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Bool(value))
    }

    fn visit_i8<E>(self, value: i8) -> Result<Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Signed(value as i64))
    }

    fn visit_i16<E>(self, value: i16) -> Result<Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Signed(value as i64))
    }

    fn visit_i32<E>(self, value: i32) -> Result<Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Signed(value as i64))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Signed(value))
    }

    fn visit_i128<E>(self, value: i128) -> Result<Value, E>
    where
        E: de::Error,
    {
        if let Ok(i64_val) = i64::try_from(value) {
            Ok(Value::Signed(i64_val))
        } else {
            Ok(Value::Bytes(Bytes::from(value.to_be_bytes().to_vec())))
        }
    }

    fn visit_u8<E>(self, value: u8) -> Result<Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Unsigned(value as u64))
    }

    fn visit_u16<E>(self, value: u16) -> Result<Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Unsigned(value as u64))
    }

    fn visit_u32<E>(self, value: u32) -> Result<Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Unsigned(value as u64))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Unsigned(value))
    }

    fn visit_u128<E>(self, value: u128) -> Result<Value, E>
    where
        E: de::Error,
    {
        if let Ok(u64_val) = u64::try_from(value) {
            Ok(Value::Unsigned(u64_val))
        } else {
            Ok(Value::Bytes(Bytes::from(value.to_be_bytes().to_vec())))
        }
    }

    fn visit_f32<E>(self, value: f32) -> Result<Value, E>
    where
        E: de::Error,
    {
        Float::try_from(value)
            .map(Value::Float)
            .map_err(|_| de::Error::custom("invalid float value (NaN or infinity)"))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Value, E>
    where
        E: de::Error,
    {
        Float::try_from(value)
            .map(Value::Float)
            .map_err(|_| de::Error::custom("invalid float value (NaN or infinity)"))
    }

    fn visit_char<E>(self, value: char) -> Result<Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Text(Text::from(value.to_string())))
    }

    fn visit_str<E>(self, value: &str) -> Result<Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Text(Text::from(value)))
    }

    fn visit_borrowed_str<E>(self, value: &'de str) -> Result<Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Text(Text::from(value)))
    }

    fn visit_string<E>(self, value: String) -> Result<Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Text(Text::from(value)))
    }

    fn visit_bytes<E>(self, value: &[u8]) -> Result<Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Bytes(Bytes::from(value)))
    }

    fn visit_borrowed_bytes<E>(self, value: &'de [u8]) -> Result<Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Bytes(Bytes::from(value)))
    }

    fn visit_byte_buf<E>(self, value: Vec<u8>) -> Result<Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Bytes(Bytes::from(value)))
    }

    fn visit_none<E>(self) -> Result<Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Null)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(ValueVisitor)
    }

    fn visit_unit<E>(self) -> Result<Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Null)
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(ValueVisitor)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(value) = seq.next_element()? {
            values.push(value);
        }
        Ok(Value::Array(values))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Value, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        let mut entries: Vec<(Value, Value)> = Vec::new();
        while let Some((key, value)) = map.next_entry()? {
            entries.push((key, value));
        }
        if entries.is_empty() {
            return Ok(Value::TextMap(IndexMap::new()));
        }
        let is_text_map = matches!(entries[0].0, Value::Text(_));
        if is_text_map {
            let mut text_map = IndexMap::new();
            for (key, value) in entries {
                match key {
                    Value::Text(text) => {
                        text_map.insert(text, value);
                    }
                    _ => {
                        return Err(de::Error::custom(
                            "inconsistent map key types: expected all Text keys",
                        ));
                    }
                }
            }
            Ok(Value::TextMap(text_map))
        } else {
            let mut bytes_map = IndexMap::new();
            for (key, value) in entries {
                match key {
                    Value::Bytes(bytes) => {
                        bytes_map.insert(bytes, value);
                    }
                    _ => {
                        return Err(de::Error::custom(
                            "inconsistent map key types: expected all Bytes keys",
                        ));
                    }
                }
            }
            Ok(Value::BytesMap(bytes_map))
        }
    }
}
