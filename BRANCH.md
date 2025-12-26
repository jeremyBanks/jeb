# Implementation Plan: Serde Serializer/Deserializer for jeb-values

## Overview

Implement `serde::Serializer` and `serde::Deserializer` traits for the `jeb-values::Value` type to enable `to_value<T>()` and `from_value<T>()` conversions. This will follow serde_json's architecture but with key improvements that leverage jeb-values's richer type system.

## Design Goals & Philosophy

### Primary Goal: 100% Round-Trip Fidelity
The most important goal is **perfect round-tripping when the target type is known**. Given any Rust type `T: Serialize + DeserializeOwned`, this must work:
```rust
let original: T = ...;
let value = to_value(&original)?;
let recovered: T = from_value(value)?;
assert_eq!(original, recovered);  // Always true
```

### Secondary Goal: Self-Description
When deserializing without knowing the target type (`deserialize_any`), we provide the most natural interpretation of the Value. However, **self-description is explicitly secondary to round-tripping**. Some information that enables round-tripping may not be self-describing:
- `Value::Bytes([16 bytes])` could be i128, u128, or actual bytes - but when we know the target type, we interpret it correctly
- `{"Some": v}` could theoretically collide with a user struct field named "Some" - but in practice this is extremely rare and round-tripping still works

### Compatibility Goal: Accept serde_json Output
Our deserializer should accept data serialized by serde_json where possible, enabling migration and interop. We serialize in our own unambiguous format, but accept multiple input formats.

### Non-Goals
- We do NOT prioritize JSON compatibility for output (we have a richer type system)
- We do NOT try to make `deserialize_any` perfect (it's inherently limited)
- We do NOT error on things we can represent (e.g., NaN/Infinity → bytes, not error)

## Key Improvements Over serde_json

| Feature | serde_json | jeb-values (our approach) |
|---------|-----------|---------------------------|
| **Integer signedness** | Lost (all → Number) | **Preserved** (Unsigned/Signed variants) |
| **Bytes** | Converted to array `[1,2,3,...]` | **Native Bytes variant** |
| **Map keys** | All stringified | **Text/Bytes native, others as array-of-pairs** |
| **NaN/Infinity** | Silently → null | **Preserved as raw bytes** (4 or 8 bytes) |
| **i128/u128 overflow** | Error or arbitrary_precision | **Preserved as raw bytes** (16 bytes) |
| **Round-trip coverage** | ~90% of serde data model (loses nested Options) | **100% of serde data model** (including nested Options) |

## Files to Modify/Create

### 1. Create Error Infrastructure
**File**: `crates/jeb-values/src/serde/error.rs` (NEW)

Error type implementing:
- `serde::ser::Error` trait
- `serde::de::Error` trait
- `std::error::Error` trait

Key error variants:
- `Message(Box<str>)` - custom error messages
- `InvalidType { unexpected, expected }` - type mismatches

### 2. Implement Serializer
**File**: `crates/jeb-values/src/serde/serialize.rs` (NEW)

**Main components**:

```rust
pub struct Serializer;

impl serde::Serializer for Serializer {
    type Ok = Value;
    type Error = Error;

    type SerializeSeq = SerializeVec;
    type SerializeTuple = SerializeVec;
    type SerializeTupleStruct = SerializeVec;
    type SerializeTupleVariant = SerializeTupleVariant;
    type SerializeMap = SerializeMap;
    type SerializeStruct = SerializeMap;
    type SerializeStructVariant = SerializeStructVariant;
}

pub fn to_value<T: Serialize>(value: T) -> Result<Value, Error> {
    value.serialize(Serializer)
}
```

**Key implementation details**:

1. **Integer serialization** (preserve signedness + overflow handling):
   - `serialize_i8/i16/i32/i64` → `Value::Signed(i64)`
   - `serialize_u8/u16/u32/u64` → `Value::Unsigned(u64)`
   - `serialize_i128` → `Value::Signed(i64)` if fits, else `Value::Bytes([16 bytes BE])`
   - `serialize_u128` → `Value::Unsigned(u64)` if fits, else `Value::Bytes([16 bytes BE])`

2. **Float serialization** (preserve NaN/Infinity as bytes):
   - `serialize_f32` → `Value::Float(Float)` if finite, else `Value::Bytes([4 bytes BE])`
   - `serialize_f64` → `Value::Float(Float)` if finite, else `Value::Bytes([8 bytes BE])`

3. **Bytes serialization** (native support):
   - `serialize_bytes` → `Value::Bytes(bytes.into())`

4. **Char serialization**:
   - `serialize_char` → `Value::Text(single-char string)`

5. **Option serialization** (tagged Some for losslessness):
   - `serialize_none` → `Value::Null`
   - `serialize_some(v)` → `Value::TextMap({"Some": to_value(v)})`
   - Fixes nested Option round-tripping: None→null, Some(None)→{"Some":null}, Some(Some(x))→{"Some":{"Some":x}}

6. **Map serialization** (buffered three-tier strategy for universal key support):
   - Buffer all (key, value) pairs during serialization
   - At `end()`, analyze all keys and pick optimal representation:
     - Empty map → `Array([])` (avoids implying key type)
     - All Text keys → `TextMap`
     - All Bytes keys → `BytesMap`
     - Mixed or other types → `Array` of `[key, value]` pairs
   - Implement `MapKeySerializer` that returns `MapKey::Text(Text)`, `MapKey::Bytes(Bytes)`, or `MapKey::Complex(Value)`

**Helper types needed**:
- `SerializeVec` - for sequences, tuples
- `SerializeMap` - buffers pairs, decides representation at end()
- `SerializeTupleVariant` - for `{variant: [values]}`
- `SerializeStructVariant` - for `{variant: {fields}}`
- `MapKeySerializer` - converts keys to Text, Bytes, or Complex (serialized Value)
- `MapKey` enum - `Text(Text) | Bytes(Bytes) | Complex(Value)`

### 3. Implement Deserializer
**File**: `crates/jeb-values/src/serde/deserialize.rs` (NEW)

**Main components**:

```rust
impl<'de> serde::Deserializer<'de> for Value {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value> {
        match self {
            Value::Null => visitor.visit_unit(),
            Value::Bool(b) => visitor.visit_bool(b),
            Value::Unsigned(u) => visitor.visit_u64(u),
            Value::Signed(i) => visitor.visit_i64(i),
            Value::Float(f) => visitor.visit_f64(*f),
            Value::Text(t) => visitor.visit_string(t.into()),
            Value::Bytes(b) => visitor.visit_byte_buf(b.into()),
            Value::Array(a) => visit_array(a, visitor),
            Value::TextMap(m) => visit_text_map(m, visitor),
            Value::BytesMap(m) => visit_bytes_map(m, visitor),
        }
    }

    // Specialized deserialize_* methods with cross-conversion...
}

// Also implement for &'de Value for borrowing support
impl<'de> serde::Deserializer<'de> for &'de Value { ... }

pub fn from_value<T: DeserializeOwned>(value: Value) -> Result<T, Error> {
    T::deserialize(value)
}
```

**Key implementation details**:

1. **Integer deserialization** (accept both native and bytes representations):
   - `deserialize_i64` accepts `Signed`, `Unsigned` (if fits), or `Bytes(len=16)` (for i128 fallback)
   - `deserialize_u64` accepts `Unsigned`, `Signed` (if non-negative), or `Bytes(len=16)` (for u128 fallback)
   - `deserialize_i128` accepts `Signed`, `Unsigned` (if fits), or `Bytes(len=16)`
   - `deserialize_u128` accepts `Unsigned`, `Signed` (if ≥0), or `Bytes(len=16)`

2. **Float deserialization** (accept both Float and raw bytes):
   - `deserialize_f32` accepts `Float` or `Bytes(len=4)`
   - `deserialize_f64` accepts `Float`, `Bytes(len=8)`, or `Bytes(len=4)` (promoted f32)

3. **Bytes deserialization** (native + compatibility):
   - Primary: `Value::Bytes` → `visit_byte_buf`
   - Fallback: `Value::Array` of numbers → convert to bytes
   - Fallback: `Value::Text` → UTF-8 bytes

4. **Map deserialization** (three-tier support):
   - `deserialize_map` accepts `TextMap`, `BytesMap`, OR `Array` of `[key, value]` pairs
   - When target type is known as map, interpret array-of-pairs as map
   - When using `deserialize_any`, array-of-pairs remains just an array (no ambiguity!)

5. **Struct deserialization** (flexible):
   - `deserialize_struct` accepts `TextMap`, `BytesMap`, or `Array` (positional/tuple-like)

6. **Identifier deserialization**:
   - `deserialize_identifier` accepts `Text` (variant names) or `Unsigned` (variant indices)
   - Supports both string-based and index-based enum formats

7. **Option deserialization** (dual-format for compatibility):
   - Primary: Accept our format: null → None, {"Some": v} → Some(v)
   - Fallback: Accept serde_json format: null → None, bare value → Some(v)
   - When deserializing Option<Option<T>>, our format is unambiguous

**Helper types needed**:
- `SeqDeserializer` - for array iteration
- `TextMapDeserializer` - for TextMap iteration
- `BytesMapDeserializer` - for BytesMap iteration
- `PairsDeserializer` - for array-of-pairs iteration (when target is map)
- `MapKeyDeserializer` - deserializes Text/Bytes/Value as map keys
- `EnumDeserializer` - for enum variant handling
- `VariantDeserializer` - for enum content

### 4. Create serde module
**File**: `crates/jeb-values/src/serde/mod.rs` (NEW)

```rust
mod error;
mod serialize;
mod deserialize;

pub use error::Error;
pub use serialize::to_value;
pub use deserialize::from_value;
```

### 5. Update lib.rs
**File**: `crates/jeb-values/src/lib.rs` (MINOR)

Replace current serialize/deserialize with serde module:
```rust
#[cfg(feature = "serde")]
pub mod serde;
#[cfg(feature = "serde")]
pub use self::serde::{to_value, from_value, Error as SerdeError};
```

Remove current serialize.rs and deserialize.rs stubs.

## Design Decisions Summary

1. **Map key strategy**: Buffered three-tier approach
   - Buffer all pairs, then at end() analyze keys:
   - Empty → Array([]) (avoids implying key type)
   - All Text keys → TextMap
   - All Bytes keys → BytesMap
   - Mixed or other types → Array of [key, value] pairs

2. **Integer overflow handling**: Bytes encoding for i128/u128
   - i128: Use Signed(i64) if fits, else Bytes([16 bytes BE])
   - u128: Use Unsigned(u64) if fits, else Bytes([16 bytes BE])
   - Deserialization accepts both representations

3. **NaN/Infinity handling**: Bytes encoding (NOT error)
   - f32: Use Float if finite, else Bytes([4 bytes BE])
   - f64: Use Float if finite, else Bytes([8 bytes BE])
   - Deserialization accepts both representations
   - Preserves exact bit patterns for round-tripping

4. **Bytes compatibility**: Accept Bytes (primary), Array (fallback), or Text (UTF-8)

5. **Struct from array**: Support positional deserialization (like serde_json)

6. **Enum representation**: Follow serde_json patterns (unit → string, others → object)

7. **Option handling**: Tagged Some for perfect round-tripping
   - Serialize: None → null, Some(v) → {"Some": v}
   - Deserialize: Accept our format (null/{"Some":v}) OR serde_json format (null/bare value) for compatibility
   - Fixes nested Option ambiguity: None→null, Some(None)→{"Some":null}, Some(Some(x))→{"Some":{"Some":x}}

8. **Identifier**: Text or Unsigned (supports both string variant names and numeric indices)

9. **Round-trip priority**: When target type is known, accept multiple Value representations
   - Our serialization is unambiguous and lossless
   - Our deserialization accepts both our format AND serde_json format for compatibility
   - Enables 100% round-trip coverage of serde data model
   - Self-describing mode (deserialize_any) uses natural interpretation

## Success Criteria

- [ ] `to_value()` and `from_value()` functions work for **all** Rust types (100% serde data model coverage)
- [ ] Integer signedness preserved (i64 ≠ u64 in Value)
- [ ] i128/u128 overflow handled via bytes encoding (round-trips correctly)
- [ ] NaN/Infinity preserved via bytes encoding (round-trips exact bit patterns)
- [ ] Bytes serialize to native Bytes variant (not array)
- [ ] TextMap, BytesMap, and array-of-pairs all supported for maps
- [ ] All map key types supported (string, bytes, integers, tuples, structs, etc.)
- [ ] All tests pass (unit, integration, doc tests)
- [ ] Documentation with examples complete
- [ ] Round-trip tests verify 100% data model coverage
- [ ] Nested Options round-trip correctly (Some(None) ≠ None)
- [ ] Compatibility: can deserialize serde_json output (Null, bare values)
