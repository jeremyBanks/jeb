//! Data model for items flowing through jeb pipelines.
//!
//! Data flows between nodes as streams of items. Each item has one of three
//! top-level types: Text, Bytes, or Structured.

use core::cmp::Ordering;

use indexmap::IndexMap;

/// An item flowing through a jeb pipeline.
///
/// Items are the fundamental unit of data in jeb. Each item has one of three
/// top-level types that represent different levels of structure.
#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    /// Raw binary data - the rawest form of data in the system.
    Bytes(Vec<u8>),
    /// Unicode text (UTF-8) - unparsed textual data.
    Text(String),
    /// A structured value in an extended JSON data model.
    Structured(Value),
}

impl Item {
    /// Returns true if this is a Bytes item.
    #[must_use]
    pub const fn is_bytes(&self) -> bool {
        matches!(self, Self::Bytes(_))
    }

    /// Returns true if this is a Text item.
    #[must_use]
    pub const fn is_text(&self) -> bool {
        matches!(self, Self::Text(_))
    }

    /// Returns true if this is a Structured item.
    #[must_use]
    pub const fn is_structured(&self) -> bool {
        matches!(self, Self::Structured(_))
    }

    /// Coerces this item to Bytes.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Self::Bytes(b) => b.clone(),
            Self::Text(s) => s.as_bytes().to_vec(),
            Self::Structured(v) => serde_json::to_vec(v).unwrap_or_default(),
        }
    }

    /// Coerces this item to Text, with potential loss of information.
    pub fn to_text(&self) -> Result<String, CoercionWarning> {
        match self {
            Self::Bytes(b) => String::from_utf8(b.clone())
                .map_err(|_| CoercionWarning::BytesToText),
            Self::Text(s) => Ok(s.clone()),
            Self::Structured(v) => Ok(serde_json::to_string(v).unwrap_or_default()),
        }
    }

    /// Coerces this item to a Structured value.
    pub fn to_structured(&self) -> Result<Value, CoercionWarning> {
        match self {
            Self::Bytes(b) => {
                // Try to interpret as UTF-8, then wrap as binary string
                match String::from_utf8(b.clone()) {
                    Ok(s) => Ok(Value::String(StringValue::Text(s))),
                    Err(_) => Ok(Value::String(StringValue::Binary(b.clone()))),
                }
            }
            Self::Text(s) => Ok(Value::String(StringValue::Text(s.clone()))),
            Self::Structured(v) => Ok(v.clone()),
        }
    }
}

impl From<Vec<u8>> for Item {
    fn from(bytes: Vec<u8>) -> Self {
        Self::Bytes(bytes)
    }
}

impl From<String> for Item {
    fn from(text: String) -> Self {
        Self::Text(text)
    }
}

impl From<&str> for Item {
    fn from(text: &str) -> Self {
        Self::Text(text.to_string())
    }
}

impl From<Value> for Item {
    fn from(value: Value) -> Self {
        Self::Structured(value)
    }
}

/// A warning emitted when implicit coercion occurs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoercionWarning {
    /// Bytes could not be converted to valid UTF-8 text.
    BytesToText,
    /// Text was wrapped as a structured string.
    TextToStructured,
    /// Bytes were wrapped as a structured binary string.
    BytesToStructured,
}

/// A structured value in jeb's extended JSON data model.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// JSON null
    Null,
    /// JSON boolean
    Bool(bool),
    /// A number value (integer or float)
    Number(NumberValue),
    /// A string value (text or binary)
    String(StringValue),
    /// An ordered array of values
    Array(Vec<Self>),
    /// An ordered map with homogeneous key types
    Map(MapValue),
}

impl Value {
    /// Returns true if this value is null.
    #[must_use]
    pub const fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    /// Returns true if this value is a boolean.
    #[must_use]
    pub const fn is_bool(&self) -> bool {
        matches!(self, Self::Bool(_))
    }

    /// Returns true if this value is a number.
    #[must_use]
    pub const fn is_number(&self) -> bool {
        matches!(self, Self::Number(_))
    }

    /// Returns true if this value is a string.
    #[must_use]
    pub const fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }

    /// Returns true if this value is an array.
    #[must_use]
    pub const fn is_array(&self) -> bool {
        matches!(self, Self::Array(_))
    }

    /// Returns true if this value is a map.
    #[must_use]
    pub const fn is_map(&self) -> bool {
        matches!(self, Self::Map(_))
    }
}

/// Number types supported in jeb's data model.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NumberValue {
    /// 64-bit signed integer
    Signed(i64),
    /// 64-bit unsigned integer
    Unsigned(u64),
    /// 64-bit finite float
    Float(f64),
}

impl NumberValue {
    /// Convert to f64 for comparison purposes.
    #[must_use]
    #[expect(clippy::cast_precision_loss)]
    pub const fn to_f64(self) -> f64 {
        match self {
            Self::Signed(i) => i as f64,
            Self::Unsigned(u) => u as f64,
            Self::Float(f) => f,
        }
    }
}

/// String types in jeb's data model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StringValue {
    /// Unicode text string (UTF-8)
    Text(String),
    /// Binary string (byte array)
    Binary(Vec<u8>),
}

impl StringValue {
    /// Returns true if this is a text string.
    #[must_use]
    pub const fn is_text(&self) -> bool {
        matches!(self, Self::Text(_))
    }

    /// Returns true if this is a binary string.
    #[must_use]
    pub const fn is_binary(&self) -> bool {
        matches!(self, Self::Binary(_))
    }

    /// Get the string as bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Text(s) => s.as_bytes(),
            Self::Binary(b) => b,
        }
    }
}

/// Map types in jeb's data model.
///
/// Each map is either text-keyed or binary-keyed. Key types are homogeneous
/// within a map, but nested maps may use different key types.
#[derive(Debug, Clone, PartialEq)]
pub enum MapValue {
    /// Map with text (Unicode) keys
    TextKeyed(IndexMap<String, Value>),
    /// Map with binary keys
    BinaryKeyed(IndexMap<Vec<u8>, Value>),
}

impl MapValue {
    /// Create a new empty text-keyed map.
    #[must_use]
    pub fn new_text_keyed() -> Self {
        Self::TextKeyed(IndexMap::new())
    }

    /// Create a new empty binary-keyed map.
    #[must_use]
    pub fn new_binary_keyed() -> Self {
        Self::BinaryKeyed(IndexMap::new())
    }

    /// Returns true if this is a text-keyed map.
    #[must_use]
    pub const fn is_text_keyed(&self) -> bool {
        matches!(self, Self::TextKeyed(_))
    }

    /// Returns true if this is a binary-keyed map.
    #[must_use]
    pub const fn is_binary_keyed(&self) -> bool {
        matches!(self, Self::BinaryKeyed(_))
    }

    /// Get the number of entries in the map.
    #[must_use]
    pub fn len(&self) -> usize {
        match self {
            Self::TextKeyed(m) => m.len(),
            Self::BinaryKeyed(m) => m.len(),
        }
    }

    /// Returns true if the map is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for MapValue {
    fn default() -> Self {
        Self::new_text_keyed()
    }
}

// Serde serialization support for Value
impl serde::Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Null => serializer.serialize_none(),
            Self::Bool(b) => serializer.serialize_bool(*b),
            Self::Number(n) => match n {
                NumberValue::Signed(i) => serializer.serialize_i64(*i),
                NumberValue::Unsigned(u) => serializer.serialize_u64(*u),
                NumberValue::Float(f) => serializer.serialize_f64(*f),
            },
            Self::String(s) => match s {
                StringValue::Text(t) => serializer.serialize_str(t),
                StringValue::Binary(b) => {
                    // Serialize binary as base64 or escaped
                    use serde::ser::SerializeMap;
                    let mut map = serializer.serialize_map(Some(1))?;
                    map.serialize_entry("$binary", &base64_encode(b))?;
                    map.end()
                }
            },
            Self::Array(arr) => {
                use serde::ser::SerializeSeq;
                let mut seq = serializer.serialize_seq(Some(arr.len()))?;
                for item in arr {
                    seq.serialize_element(item)?;
                }
                seq.end()
            }
            Self::Map(map) => {
                use serde::ser::SerializeMap;
                match map {
                    MapValue::TextKeyed(m) => {
                        let mut ser_map = serializer.serialize_map(Some(m.len()))?;
                        for (k, v) in m {
                            ser_map.serialize_entry(k, v)?;
                        }
                        ser_map.end()
                    }
                    MapValue::BinaryKeyed(m) => {
                        let mut ser_map = serializer.serialize_map(Some(m.len()))?;
                        for (k, v) in m {
                            ser_map.serialize_entry(&base64_encode(k), v)?;
                        }
                        ser_map.end()
                    }
                }
            }
        }
    }
}

fn base64_encode(bytes: &[u8]) -> String {
    use std::io::Write;
    let mut buf = Vec::new();
    {
        let mut encoder = Base64Encoder::new(&mut buf);
        encoder.write_all(bytes).ok();
    }
    String::from_utf8(buf).unwrap_or_default()
}

// Simple base64 encoder
struct Base64Encoder<W> {
    writer: W,
}

impl<W: std::io::Write> Base64Encoder<W> {
    fn new(writer: W) -> Self {
        Self { writer }
    }
}

impl<W: std::io::Write> std::io::Write for Base64Encoder<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        const ALPHABET: &[u8; 64] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

        for chunk in buf.chunks(3) {
            let b0 = chunk.first().copied().unwrap_or(0);
            let b1 = chunk.get(1).copied().unwrap_or(0);
            let b2 = chunk.get(2).copied().unwrap_or(0);

            let c0 = ALPHABET[(b0 >> 2) as usize];
            let c1 = ALPHABET[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize];
            let c2 = if chunk.len() > 1 {
                ALPHABET[(((b1 & 0x0f) << 2) | (b2 >> 6)) as usize]
            } else {
                b'='
            };
            let c3 = if chunk.len() > 2 {
                ALPHABET[(b2 & 0x3f) as usize]
            } else {
                b'='
            };

            self.writer.write_all(&[c0, c1, c2, c3])?;
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}

/// Convert from `serde_json::Value` to our Value type.
impl From<serde_json::Value> for Value {
    fn from(v: serde_json::Value) -> Self {
        match v {
            serde_json::Value::Null => Self::Null,
            serde_json::Value::Bool(b) => Self::Bool(b),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Self::Number(NumberValue::Signed(i))
                } else if let Some(u) = n.as_u64() {
                    Self::Number(NumberValue::Unsigned(u))
                } else if let Some(f) = n.as_f64() {
                    Self::Number(NumberValue::Float(f))
                } else {
                    Self::Number(NumberValue::Float(0.0))
                }
            }
            serde_json::Value::String(s) => Self::String(StringValue::Text(s)),
            serde_json::Value::Array(arr) => {
                Self::Array(arr.into_iter().map(Self::from).collect())
            }
            serde_json::Value::Object(obj) => {
                let map: IndexMap<String, Self> =
                    obj.into_iter().map(|(k, v)| (k, Self::from(v))).collect();
                Self::Map(MapValue::TextKeyed(map))
            }
        }
    }
}

/// Convert from our Value type to `serde_json::Value`.
impl From<Value> for serde_json::Value {
    fn from(v: Value) -> Self {
        match v {
            Value::Null => Self::Null,
            Value::Bool(b) => Self::Bool(b),
            Value::Number(n) => match n {
                NumberValue::Signed(i) => Self::Number(i.into()),
                NumberValue::Unsigned(u) => Self::Number(u.into()),
                NumberValue::Float(f) => {
                    serde_json::Number::from_f64(f).map_or(Self::Null, Self::Number)
                }
            },
            Value::String(s) => match s {
                StringValue::Text(t) => Self::String(t),
                StringValue::Binary(b) => Self::String(base64_encode(&b)),
            },
            Value::Array(arr) => Self::Array(arr.into_iter().map(Self::from).collect()),
            Value::Map(map) => match map {
                MapValue::TextKeyed(m) => {
                    Self::Object(m.into_iter().map(|(k, v)| (k, Self::from(v))).collect())
                }
                MapValue::BinaryKeyed(m) => Self::Object(
                    m.into_iter()
                        .map(|(k, v)| (base64_encode(&k), Self::from(v)))
                        .collect(),
                ),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_bytes() {
        let item = Item::Bytes(vec![1, 2, 3]);
        assert!(item.is_bytes());
        assert!(!item.is_text());
        assert!(!item.is_structured());
    }

    #[test]
    fn test_item_text() {
        let item = Item::Text("hello".to_string());
        assert!(!item.is_bytes());
        assert!(item.is_text());
        assert!(!item.is_structured());
    }

    #[test]
    fn test_item_structured() {
        let item = Item::Structured(Value::Null);
        assert!(!item.is_bytes());
        assert!(!item.is_text());
        assert!(item.is_structured());
    }

    #[test]
    fn test_value_types() {
        assert!(Value::Null.is_null());
        assert!(Value::Bool(true).is_bool());
        assert!(Value::Number(NumberValue::Signed(42)).is_number());
        assert!(Value::String(StringValue::Text("hi".into())).is_string());
        assert!(Value::Array(vec![]).is_array());
        assert!(Value::Map(MapValue::new_text_keyed()).is_map());
    }

    #[test]
    fn test_item_coercion_bytes_to_text() {
        let item = Item::Bytes(b"hello".to_vec());
        let text = item.to_text().unwrap();
        assert_eq!(text, "hello");
    }

    #[test]
    fn test_item_coercion_invalid_utf8() {
        let item = Item::Bytes(vec![0xFF, 0xFE]);
        assert!(item.to_text().is_err());
    }
}
