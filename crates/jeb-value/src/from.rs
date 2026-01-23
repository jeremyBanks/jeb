use crate::{
    Array,
    Bytes,
    BytesMap,
    Null,
    Number,
    String,
    StringMap,
    Value,
};

// [impl jeb-value.number.try-from-inner]
// (Note: TryFrom<f64> for Number is in number/mod.rs)

// [impl jeb-value.conv.numeric.f32]
impl From<f32> for Number {
    fn from(value: f32) -> Self {
        // f32 always fits in f64, and we assume finite inputs for From
        // Use try_from for fallible conversion
        Number::new(value.into()).expect("f32 should always be representable as finite f64")
    }
}

// Numbers that fit exactly in i32/u32 can always be exactly represented in f64
impl From<i32> for Number {
    fn from(value: i32) -> Self {
        Number(value.into())
    }
}

impl From<u32> for Number {
    fn from(value: u32) -> Self {
        Number(value.into())
    }
}

impl From<i16> for Number {
    fn from(value: i16) -> Self {
        Number(value.into())
    }
}

impl From<u16> for Number {
    fn from(value: u16) -> Self {
        Number(value.into())
    }
}

impl From<i8> for Number {
    fn from(value: i8) -> Self {
        Number(value.into())
    }
}

impl From<u8> for Number {
    fn from(value: u8) -> Self {
        Number(value.into())
    }
}

// [impl jeb-value.conv.fallback.numeric]
// [impl jeb-value.conv.fallback.infallible]
// For f32/f64, we try Number first, fall back to Bytes if not finite
impl From<f32> for Value {
    fn from(value: f32) -> Self {
        if value.is_finite() {
            Value::Number(Number::from(value))
        } else {
            // [impl jeb-value.conv.fallback.bytes-encoding]
            Value::Bytes(Bytes::from(value.to_be_bytes().to_vec()))
        }
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        if let Some(n) = Number::new(value) {
            Value::Number(n)
        } else {
            // [impl jeb-value.conv.fallback.bytes-encoding]
            Value::Bytes(Bytes::from(value.to_be_bytes().to_vec()))
        }
    }
}

// Small integers always fit exactly in f64
impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Value::Number(Number::from(value))
    }
}

impl From<u32> for Value {
    fn from(value: u32) -> Self {
        Value::Number(Number::from(value))
    }
}

impl From<i16> for Value {
    fn from(value: i16) -> Self {
        Value::Number(Number::from(value))
    }
}

impl From<u16> for Value {
    fn from(value: u16) -> Self {
        Value::Number(Number::from(value))
    }
}

impl From<i8> for Value {
    fn from(value: i8) -> Self {
        Value::Number(Number::from(value))
    }
}

impl From<u8> for Value {
    fn from(value: u8) -> Self {
        Value::Number(Number::from(value))
    }
}

// [impl jeb-value.conv.numeric.exactness]
// For i64/u64/i128/u128, check if exactly representable in f64
fn is_exactly_representable_i64(value: i64) -> bool {
    let as_f64 = value as f64;
    as_f64.is_finite() && (as_f64 as i64) == value
}

fn is_exactly_representable_u64(value: u64) -> bool {
    let as_f64 = value as f64;
    as_f64.is_finite() && (as_f64 as u64) == value
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        if is_exactly_representable_i64(value) {
            Value::Number(Number(value as f64))
        } else {
            // [impl jeb-value.conv.fallback.bytes-encoding]
            Value::Bytes(Bytes::from(value.to_be_bytes().to_vec()))
        }
    }
}

impl From<u64> for Value {
    fn from(value: u64) -> Self {
        if is_exactly_representable_u64(value) {
            Value::Number(Number(value as f64))
        } else {
            // [impl jeb-value.conv.fallback.bytes-encoding]
            Value::Bytes(Bytes::from(value.to_be_bytes().to_vec()))
        }
    }
}

impl From<i128> for Value {
    fn from(value: i128) -> Self {
        // First check if it fits in i64
        if let Ok(v) = i64::try_from(value) {
            Value::from(v)
        } else {
            // [impl jeb-value.conv.fallback.bytes-encoding]
            Value::Bytes(Bytes::from(value.to_be_bytes().to_vec()))
        }
    }
}

impl From<u128> for Value {
    fn from(value: u128) -> Self {
        // First check if it fits in u64
        if let Ok(v) = u64::try_from(value) {
            Value::from(v)
        } else {
            // [impl jeb-value.conv.fallback.bytes-encoding]
            Value::Bytes(Bytes::from(value.to_be_bytes().to_vec()))
        }
    }
}

impl From<()> for Value {
    fn from((): ()) -> Self {
        Value::Null(Null::new())
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Boolean(value.into())
    }
}

// String conversions - use our String type, not std::string::String
impl From<std::string::String> for Value {
    fn from(value: std::string::String) -> Self {
        Value::String(String::from(value))
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Value::String(String::from(value))
    }
}

impl From<Vec<u8>> for Value {
    fn from(value: Vec<u8>) -> Self {
        Value::Bytes(Bytes::from(value))
    }
}

impl From<&[u8]> for Value {
    fn from(value: &[u8]) -> Self {
        Value::Bytes(Bytes::from(value))
    }
}

impl<const N: usize> From<[u8; N]> for Value {
    fn from(value: [u8; N]) -> Self {
        Value::Bytes(Bytes::from(value.to_vec()))
    }
}

impl<const N: usize> From<&[u8; N]> for Value {
    fn from(value: &[u8; N]) -> Self {
        Value::Bytes(Bytes::from(value.to_vec()))
    }
}

impl<const N: usize> From<[Value; N]> for Value {
    fn from(value: [Value; N]) -> Self {
        value.into_iter().collect()
    }
}

impl FromIterator<Value> for Value {
    fn from_iter<T: IntoIterator<Item = Value>>(iter: T) -> Self {
        Value::Array(Array::from_iter(iter))
    }
}

impl<const N: usize> From<[(String, Value); N]> for Value {
    fn from(value: [(String, Value); N]) -> Self {
        value.into_iter().collect()
    }
}

impl<const N: usize> From<[(std::string::String, Value); N]> for Value {
    fn from(value: [(std::string::String, Value); N]) -> Self {
        value.into_iter().collect()
    }
}

impl<const N: usize> From<[(&str, Value); N]> for Value {
    fn from(value: [(&str, Value); N]) -> Self {
        value.into_iter().collect()
    }
}

impl FromIterator<(String, Value)> for Value {
    fn from_iter<T: IntoIterator<Item = (String, Value)>>(iter: T) -> Self {
        Value::StringMap(StringMap::from_iter(iter))
    }
}

impl FromIterator<(std::string::String, Value)> for Value {
    fn from_iter<T: IntoIterator<Item = (std::string::String, Value)>>(iter: T) -> Self {
        Value::StringMap(StringMap::from_iter(
            iter.into_iter().map(|(k, v)| (String::from(k), v)),
        ))
    }
}

impl<'a> FromIterator<(&'a str, Value)> for Value {
    fn from_iter<T: IntoIterator<Item = (&'a str, Value)>>(iter: T) -> Self {
        Value::StringMap(StringMap::from_iter(
            iter.into_iter().map(|(k, v)| (String::from(k), v)),
        ))
    }
}

impl<const N: usize> From<[(Bytes, Value); N]> for Value {
    fn from(value: [(Bytes, Value); N]) -> Self {
        value.into_iter().collect()
    }
}

impl<const N: usize> From<[(Vec<u8>, Value); N]> for Value {
    fn from(value: [(Vec<u8>, Value); N]) -> Self {
        value.into_iter().collect()
    }
}

impl<const N: usize> From<[(&[u8], Value); N]> for Value {
    fn from(value: [(&[u8], Value); N]) -> Self {
        value.into_iter().collect()
    }
}

impl FromIterator<(Bytes, Value)> for Value {
    fn from_iter<T: IntoIterator<Item = (Bytes, Value)>>(iter: T) -> Self {
        Value::BytesMap(BytesMap::from_iter(iter))
    }
}

impl FromIterator<(Vec<u8>, Value)> for Value {
    fn from_iter<T: IntoIterator<Item = (Vec<u8>, Value)>>(iter: T) -> Self {
        Value::BytesMap(BytesMap::from_iter(
            iter.into_iter().map(|(k, v)| (Bytes::from(k), v)),
        ))
    }
}

impl<'a> FromIterator<(&'a [u8], Value)> for Value {
    fn from_iter<T: IntoIterator<Item = (&'a [u8], Value)>>(iter: T) -> Self {
        Value::BytesMap(BytesMap::from_iter(
            iter.into_iter().map(|(k, v)| (Bytes::from(k), v)),
        ))
    }
}
