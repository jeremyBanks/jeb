//! `Deserializer` deserializing from our `Value` type into arbitrary
//! `Deserialize` types.
use {
    crate::{
        Boolean,
        Bytes,
        Null,
        Number,
        String,
        Value,
        serde::{
            SerdeError,
            error::Unexpected,
        },
    },
    indexmap::IndexMap,
    serde::de::{
        self,
        DeserializeSeed,
        Visitor,
    },
};
impl<'de> de::Deserializer<'de> for Value {
    type Error = SerdeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, SerdeError>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Null(_) => visitor.visit_unit(),
            Value::Boolean(b) => visitor.visit_bool(*b),
            Value::Number(n) => {
                let f = *n;
                // Try to represent as integer if it has no fractional part
                if f.fract() == 0.0 {
                    if f >= 0.0 && f <= u64::MAX as f64 {
                        visitor.visit_u64(f as u64)
                    } else if f >= i64::MIN as f64 && f <= i64::MAX as f64 {
                        visitor.visit_i64(f as i64)
                    } else {
                        visitor.visit_f64(f)
                    }
                } else {
                    visitor.visit_f64(f)
                }
            }
            Value::String(s) => visitor.visit_string(s.into_inner()),
            Value::Bytes(b) => visitor.visit_byte_buf(b.into_inner()),
            Value::Array(a) => visitor.visit_seq(SeqDeserializer::new(a)),
            Value::StringMap(m) => visitor.visit_map(StringMapDeserializer::new(m)),
            Value::BytesMap(m) => visitor.visit_map(BytesMapDeserializer::new(m)),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, SerdeError>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Boolean(b) => visitor.visit_bool(*b),
            _ => Err(SerdeError::invalid_type(self.unexpected(), "a boolean")),
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, SerdeError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, SerdeError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, SerdeError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, SerdeError>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Number(n) => {
                let f = *n;
                if f.fract() == 0.0 && f >= i64::MIN as f64 && f <= i64::MAX as f64 {
                    visitor.visit_i64(f as i64)
                } else {
                    Err(SerdeError::custom("number value out of range for i64"))
                }
            }
            Value::Bytes(b) if b.len() == 16 => {
                let slice: &[u8] = b.as_ref();
                let bytes: [u8; 16] = slice.try_into().unwrap();
                let i = i128::from_be_bytes(bytes);
                if let Ok(i64_val) = i64::try_from(i) {
                    visitor.visit_i64(i64_val)
                } else {
                    Err(SerdeError::custom("i128 value out of range for i64"))
                }
            }
            Value::Bytes(b) if b.len() == 8 => {
                let slice: &[u8] = b.as_ref();
                let bytes: [u8; 8] = slice.try_into().unwrap();
                visitor.visit_i64(i64::from_be_bytes(bytes))
            }
            _ => Err(SerdeError::invalid_type(self.unexpected(), "an integer")),
        }
    }

    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value, SerdeError>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Number(n) => {
                let f = *n;
                if f.fract() == 0.0 {
                    visitor.visit_i128(f as i128)
                } else {
                    Err(SerdeError::custom("number is not an integer"))
                }
            }
            Value::Bytes(b) if b.len() == 16 => {
                let slice: &[u8] = b.as_ref();
                let bytes: [u8; 16] = slice.try_into().unwrap();
                let i = i128::from_be_bytes(bytes);
                visitor.visit_i128(i)
            }
            _ => Err(SerdeError::invalid_type(self.unexpected(), "an i128")),
        }
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, SerdeError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_u64(visitor)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, SerdeError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_u64(visitor)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, SerdeError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_u64(visitor)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, SerdeError>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Number(n) => {
                let f = *n;
                if f.fract() == 0.0 && f >= 0.0 && f <= u64::MAX as f64 {
                    visitor.visit_u64(f as u64)
                } else {
                    Err(SerdeError::custom("number value out of range for u64"))
                }
            }
            Value::Bytes(b) if b.len() == 16 => {
                let slice: &[u8] = b.as_ref();
                let bytes: [u8; 16] = slice.try_into().unwrap();
                let u = u128::from_be_bytes(bytes);
                if let Ok(u64_val) = u64::try_from(u) {
                    visitor.visit_u64(u64_val)
                } else {
                    Err(SerdeError::custom("u128 value out of range for u64"))
                }
            }
            _ => Err(SerdeError::invalid_type(
                self.unexpected(),
                "an unsigned integer",
            )),
        }
    }

    fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value, SerdeError>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Number(n) => {
                let f = *n;
                if f.fract() == 0.0 && f >= 0.0 {
                    visitor.visit_u128(f as u128)
                } else {
                    Err(SerdeError::custom("number is negative or not an integer"))
                }
            }
            Value::Bytes(b) if b.len() == 16 => {
                let slice: &[u8] = b.as_ref();
                let bytes: [u8; 16] = slice.try_into().unwrap();
                let u = u128::from_be_bytes(bytes);
                visitor.visit_u128(u)
            }
            _ => Err(SerdeError::invalid_type(self.unexpected(), "a u128")),
        }
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, SerdeError>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Number(n) => visitor.visit_f32(*n as f32),
            Value::Bytes(b) if b.len() == 4 => {
                let slice: &[u8] = b.as_ref();
                let bytes: [u8; 4] = slice.try_into().unwrap();
                let f = f32::from_be_bytes(bytes);
                visitor.visit_f32(f)
            }
            _ => Err(SerdeError::invalid_type(self.unexpected(), "a float")),
        }
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, SerdeError>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Number(n) => visitor.visit_f64(*n),
            Value::Bytes(b) if b.len() == 8 => {
                let slice: &[u8] = b.as_ref();
                let bytes: [u8; 8] = slice.try_into().unwrap();
                let f = f64::from_be_bytes(bytes);
                visitor.visit_f64(f)
            }
            Value::Bytes(b) if b.len() == 4 => {
                let slice: &[u8] = b.as_ref();
                let bytes: [u8; 4] = slice.try_into().unwrap();
                let f = f32::from_be_bytes(bytes);
                visitor.visit_f64(f as f64)
            }
            _ => Err(SerdeError::invalid_type(self.unexpected(), "a float")),
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, SerdeError>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::String(s) => {
                let inner = s.into_inner();
                let mut chars = inner.chars();
                if let Some(c) = chars.next()
                    && chars.next().is_none()
                {
                    return visitor.visit_char(c);
                }
                Err(SerdeError::invalid_type(
                    Unexpected::Str(inner.into_boxed_str()),
                    "a single character",
                ))
            }
            _ => Err(SerdeError::invalid_type(self.unexpected(), "a character")),
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, SerdeError>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::String(s) => visitor.visit_string(s.into_inner()),
            _ => Err(SerdeError::invalid_type(self.unexpected(), "a string")),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, SerdeError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, SerdeError>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Bytes(b) => visitor.visit_byte_buf(b.into_inner()),
            Value::Array(arr) => {
                let mut bytes = Vec::with_capacity(arr.len());
                for v in arr {
                    match v {
                        Value::Number(n) => {
                            let f = *n;
                            if f.fract() == 0.0 && f >= 0.0 && f <= 255.0 {
                                bytes.push(f as u8);
                            } else {
                                return Err(SerdeError::custom(
                                    "array contains non-byte values for bytes deserialization",
                                ));
                            }
                        }
                        _ => {
                            return Err(SerdeError::custom(
                                "array contains non-byte values for bytes deserialization",
                            ));
                        }
                    }
                }
                visitor.visit_byte_buf(bytes)
            }
            Value::String(s) => visitor.visit_byte_buf(s.into_inner().into_bytes()),
            _ => Err(SerdeError::invalid_type(self.unexpected(), "bytes")),
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, SerdeError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, SerdeError>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Null(_) => visitor.visit_none(),
            Value::StringMap(map) if map.len() == 1 => {
                if let Some((key, value)) = map.iter().next()
                    && key.as_str() == "Some"
                {
                    let value = value.clone();
                    return visitor.visit_some(value);
                }
                visitor.visit_some(Value::StringMap(map))
            }
            value => visitor.visit_some(value),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, SerdeError>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Null(_) => visitor.visit_unit(),
            _ => Err(SerdeError::invalid_type(self.unexpected(), "null")),
        }
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, SerdeError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, SerdeError>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, SerdeError>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Array(arr) => visitor.visit_seq(SeqDeserializer::new(arr)),
            _ => Err(SerdeError::invalid_type(self.unexpected(), "a sequence")),
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, SerdeError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, SerdeError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, SerdeError>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::StringMap(m) => visitor.visit_map(StringMapDeserializer::new(m)),
            Value::BytesMap(m) => visitor.visit_map(BytesMapDeserializer::new(m)),
            Value::Array(arr) => {
                if arr.is_empty() {
                    visitor.visit_map(PairsDeserializer::new(Vec::new()))
                } else if arr
                    .iter()
                    .all(|v| matches!(v, Value::Array(inner) if inner.len() == 2))
                {
                    visitor.visit_map(PairsDeserializer::new(arr))
                } else {
                    Err(SerdeError::invalid_type(Unexpected::Seq, "a map"))
                }
            }
            _ => Err(SerdeError::invalid_type(self.unexpected(), "a map")),
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, SerdeError>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::StringMap(m) => visitor.visit_map(StringMapDeserializer::new(m)),
            Value::BytesMap(m) => visitor.visit_map(BytesMapDeserializer::new(m)),
            Value::Array(arr) => visitor.visit_seq(SeqDeserializer::new(arr)),
            _ => Err(SerdeError::invalid_type(self.unexpected(), "a struct")),
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, SerdeError>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::String(s) => {
                let inner = s.into_inner();
                visitor.visit_enum(inner.into_deserializer())
            }
            Value::StringMap(m) if m.len() == 1 => {
                let (key, value) = m.into_iter().next().unwrap();
                let variant = key.into_inner();
                visitor.visit_enum(EnumDeserializer {
                    variant,
                    value: Some(value),
                })
            }
            _ => Err(SerdeError::invalid_type(self.unexpected(), "an enum")),
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, SerdeError>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::String(s) => visitor.visit_string(s.into_inner()),
            Value::Number(n) => {
                let f = *n;
                if f.fract() == 0.0 && f >= 0.0 && f <= u64::MAX as f64 {
                    visitor.visit_u64(f as u64)
                } else {
                    Err(SerdeError::invalid_type(self.unexpected(), "an identifier"))
                }
            }
            _ => Err(SerdeError::invalid_type(self.unexpected(), "an identifier")),
        }
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, SerdeError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}
impl Value {
    fn unexpected(&self) -> Unexpected {
        match self {
            Value::Null(_) => Unexpected::Unit,
            Value::Boolean(b) => Unexpected::Bool(**b),
            Value::Number(n) => Unexpected::Float(**n),
            Value::String(s) => Unexpected::Str(s.clone().into_inner().into_boxed_str()),
            Value::Bytes(b) => {
                let slice: &[u8] = b.as_ref();
                Unexpected::Bytes(slice.to_vec().into_boxed_slice())
            }
            Value::Array(_) => Unexpected::Seq,
            Value::StringMap(_) | Value::BytesMap(_) => Unexpected::Map,
        }
    }
}
struct SeqDeserializer {
    iter: std::vec::IntoIter<Value>,
}
impl SeqDeserializer {
    fn new(vec: Vec<Value>) -> Self {
        SeqDeserializer {
            iter: vec.into_iter(),
        }
    }
}
impl<'de> de::SeqAccess<'de> for SeqDeserializer {
    type Error = SerdeError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, SerdeError>
    where
        T: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(value) => seed.deserialize(value).map(Some),
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.iter.len())
    }
}
struct StringMapDeserializer {
    iter: <IndexMap<String, Value> as IntoIterator>::IntoIter,
    value: Option<Value>,
}
impl StringMapDeserializer {
    fn new(map: IndexMap<String, Value>) -> Self {
        StringMapDeserializer {
            iter: map.into_iter(),
            value: None,
        }
    }
}
impl<'de> de::MapAccess<'de> for StringMapDeserializer {
    type Error = SerdeError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, SerdeError>
    where
        K: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some((key, value)) => {
                self.value = Some(value);
                seed.deserialize(Value::String(key)).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, SerdeError>
    where
        V: DeserializeSeed<'de>,
    {
        let value = self.value.take().ok_or_else(|| {
            SerdeError::Message("next_value_seed called before next_key_seed".into())
        })?;
        seed.deserialize(value)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.iter.len())
    }
}
struct BytesMapDeserializer {
    iter: <IndexMap<Bytes, Value> as IntoIterator>::IntoIter,
    value: Option<Value>,
}
impl BytesMapDeserializer {
    fn new(map: IndexMap<Bytes, Value>) -> Self {
        BytesMapDeserializer {
            iter: map.into_iter(),
            value: None,
        }
    }
}
impl<'de> de::MapAccess<'de> for BytesMapDeserializer {
    type Error = SerdeError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, SerdeError>
    where
        K: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some((key, value)) => {
                self.value = Some(value);
                seed.deserialize(Value::Bytes(key)).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, SerdeError>
    where
        V: DeserializeSeed<'de>,
    {
        let value = self.value.take().ok_or_else(|| {
            SerdeError::Message("next_value_seed called before next_key_seed".into())
        })?;
        seed.deserialize(value)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.iter.len())
    }
}
struct PairsDeserializer {
    pairs: std::vec::IntoIter<Value>,
    value: Option<Value>,
}
impl PairsDeserializer {
    fn new(arr: Vec<Value>) -> Self {
        PairsDeserializer {
            pairs: arr.into_iter(),
            value: None,
        }
    }
}
impl<'de> de::MapAccess<'de> for PairsDeserializer {
    type Error = SerdeError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, SerdeError>
    where
        K: DeserializeSeed<'de>,
    {
        match self.pairs.next() {
            Some(Value::Array(mut pair)) if pair.len() == 2 => {
                let value = pair.pop().unwrap();
                let key = pair.pop().unwrap();
                self.value = Some(value);
                seed.deserialize(key).map(Some)
            }
            Some(_) => Err(SerdeError::custom("expected [key, value] pair")),
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, SerdeError>
    where
        V: DeserializeSeed<'de>,
    {
        let value = self.value.take().ok_or_else(|| {
            SerdeError::Message("next_value_seed called before next_key_seed".into())
        })?;
        seed.deserialize(value)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.pairs.len())
    }
}
struct EnumDeserializer {
    variant: std::string::String,
    value: Option<Value>,
}
impl<'de> de::EnumAccess<'de> for EnumDeserializer {
    type Error = SerdeError;
    type Variant = VariantDeserializer;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), SerdeError>
    where
        V: DeserializeSeed<'de>,
    {
        let variant = self.variant.clone();
        let visitor = seed.deserialize(variant.into_deserializer())?;
        Ok((visitor, VariantDeserializer { value: self.value }))
    }
}
struct VariantDeserializer {
    value: Option<Value>,
}
impl<'de> de::VariantAccess<'de> for VariantDeserializer {
    type Error = SerdeError;

    fn unit_variant(self) -> Result<(), SerdeError> {
        match self.value {
            None => Ok(()),
            Some(_) => Err(SerdeError::custom("expected unit variant")),
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, SerdeError>
    where
        T: DeserializeSeed<'de>,
    {
        match self.value {
            Some(value) => seed.deserialize(value),
            None => Err(SerdeError::custom("expected newtype variant")),
        }
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, SerdeError>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Some(Value::Array(arr)) => visitor.visit_seq(SeqDeserializer::new(arr)),
            Some(_) => Err(SerdeError::custom("expected tuple variant")),
            None => Err(SerdeError::custom("expected tuple variant")),
        }
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, SerdeError>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Some(Value::StringMap(m)) => visitor.visit_map(StringMapDeserializer::new(m)),
            Some(Value::BytesMap(m)) => visitor.visit_map(BytesMapDeserializer::new(m)),
            Some(Value::Array(arr)) => visitor.visit_map(PairsDeserializer::new(arr)),
            Some(_) => Err(SerdeError::custom("expected struct variant")),
            None => Err(SerdeError::custom("expected struct variant")),
        }
    }
}
pub fn from_value<T: de::DeserializeOwned>(value: Value) -> Result<T, SerdeError> {
    T::deserialize(value)
}
trait IntoDeserializer {
    fn into_deserializer(self) -> StringDeserializer;
}
impl IntoDeserializer for std::string::String {
    fn into_deserializer(self) -> StringDeserializer {
        StringDeserializer(self)
    }
}
struct StringDeserializer(std::string::String);
impl<'de> de::Deserializer<'de> for StringDeserializer {
    type Error = SerdeError;

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string bytes
        byte_buf option unit unit_struct newtype_struct seq tuple tuple_struct map struct
        identifier ignored_any
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, SerdeError>
    where
        V: Visitor<'de>,
    {
        visitor.visit_string(self.0)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, SerdeError>
    where
        V: Visitor<'de>,
    {
        visitor.visit_enum(self.0.into_deserializer())
    }
}
impl<'de> de::EnumAccess<'de> for StringDeserializer {
    type Error = SerdeError;
    type Variant = UnitVariant;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), SerdeError>
    where
        V: DeserializeSeed<'de>,
    {
        let visitor = seed.deserialize(self)?;
        Ok((visitor, UnitVariant))
    }
}
struct UnitVariant;
impl<'de> de::VariantAccess<'de> for UnitVariant {
    type Error = SerdeError;

    fn unit_variant(self) -> Result<(), SerdeError> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, _seed: T) -> Result<T::Value, SerdeError>
    where
        T: DeserializeSeed<'de>,
    {
        Err(SerdeError::custom("expected unit variant"))
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value, SerdeError>
    where
        V: Visitor<'de>,
    {
        Err(SerdeError::custom("expected unit variant"))
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, SerdeError>
    where
        V: Visitor<'de>,
    {
        Err(SerdeError::custom("expected unit variant"))
    }
}
