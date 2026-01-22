# Implementation Plan: Complete Serde Data Model Value Type

## Overview

This crate implements `serde_value::Value`, a Rust type that can represent any value expressible in the serde data model with complete fidelity. Unlike `serde_json::Value` or `jeb_value::Value`, this type preserves all information including struct names, field names, enum variant names, variant indices, and the distinctions between tuples and sequences, structs and maps, etc.

## Design Decisions (Pre-Resolved)

- **No borrowed variant**: All strings are owned (`String`), no `Value<'a>` lifetime parameter
- **Integer coercion**: Use `unimplemented!()` for now, revisit later
- **Variant identification**: Preserve both indices (`u32`) and names (`&'static str`) when serde provides them
- **Map implementation**: Use `Vec<(Value, Value)>` - no hash lookup optimization
- **No streaming**: In-memory only, no streaming API

## The Serde Data Model (29 Types)

### Primitives (14)
- `bool`
- `i8`, `i16`, `i32`, `i64`, `i128`
- `u8`, `u16`, `u32`, `u64`, `u128`
- `f32`, `f64`
- `char`

### String and Bytes (2)
- `string` (UTF-8)
- `byte_array` (arbitrary bytes)

### Option (2)
- `none`
- `some(value)`

### Unit Types (2)
- `unit` (the `()` type)
- `unit_struct` (named unit struct, e.g., `struct Foo;`)

### Newtype (2)
- `newtype_struct` (e.g., `struct Meters(u32)`)
- `newtype_variant` (e.g., `enum E { V(u32) }`)

### Sequences (4)
- `seq` (variable-length, e.g., `Vec<T>`)
- `tuple` (fixed-length, e.g., `(A, B, C)`)
- `tuple_struct` (named tuple, e.g., `struct Point(i32, i32)`)
- `tuple_variant` (e.g., `enum E { V(i32, i32) }`)

### Maps and Structs (3)
- `map` (key-value pairs with any key type)
- `struct` (named struct with named fields)
- `struct_variant` (e.g., `enum E { V { x: i32 } }`)

### Unit Variant (1)
- `unit_variant` (e.g., `enum E { A, B }`)

## Type Definition

```rust
/// A value that can represent any type in the serde data model.
#[derive(Debug, Clone, PartialEq)]
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
```

## Implementation Phases

### Phase 1: Core Type Definition
**Files:** `src/lib.rs`, `src/value.rs`

1. Define the `Value` enum with all 29 variants
2. Implement standard traits:
   - `Debug` (derive)
   - `Clone` (derive)
   - `PartialEq` (derive, noting f32/f64 NaN behavior)
   - `Eq` (manual, must handle floats)
   - `Hash` (manual, must handle floats)
3. Add `#[must_use]` attribute

### Phase 2: Serialize Implementation
**Files:** `src/ser.rs`

Implement `serde::Serialize` for `Value`. This is straightforward - dispatch on each variant and call the corresponding serializer method:

```rust
impl Serialize for Value {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Value::Bool(v) => serializer.serialize_bool(*v),
            Value::I8(v) => serializer.serialize_i8(*v),
            // ... primitives

            Value::UnitStruct { name } => serializer.serialize_unit_struct(name),

            Value::NewtypeStruct { name, value } => {
                serializer.serialize_newtype_struct(name, value.as_ref())
            }

            Value::NewtypeVariant { enum_name, variant_index, variant, value } => {
                serializer.serialize_newtype_variant(enum_name, *variant_index, variant, value.as_ref())
            }

            Value::Struct { name, fields } => {
                let mut s = serializer.serialize_struct(name, fields.len())?;
                for (field_name, value) in fields {
                    s.serialize_field(field_name, value)?;
                }
                s.end()
            }
            // ... etc
        }
    }
}
```

### Phase 3: Deserializer Implementation (Value → T)
**Files:** `src/de.rs`

Implement `serde::de::Deserializer` for `Value` (and `&Value`). This allows deserializing any type `T: Deserialize` from a `Value`:

```rust
impl<'de> Deserializer<'de> for Value {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self {
            Value::Bool(v) => visitor.visit_bool(v),
            Value::I8(v) => visitor.visit_i8(v),
            // ...
            Value::Struct { name, fields } => {
                visitor.visit_map(StructMapAccess::new(fields))
            }
            // ...
        }
    }

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self {
            Value::Bool(v) => visitor.visit_bool(v),
            _ => Err(Error::type_mismatch("bool", &self)),
        }
    }

    // ... type-specific deserialize methods
}
```

Key helper types needed:
- `SeqAccess` for sequences/tuples
- `MapAccess` for maps
- `StructMapAccess` for structs (provides field names)
- `EnumAccess` and `VariantAccess` for enum variants

### Phase 4: Serializer Implementation (T → Value)
**Files:** `src/to_value.rs`

Implement `serde::ser::Serializer` that produces `Value`. This is what `to_value()` uses:

```rust
pub struct ValueSerializer;

impl Serializer for ValueSerializer {
    type Ok = Value;
    type Error = Error;
    type SerializeSeq = SerializeSeq;
    type SerializeTuple = SerializeTuple;
    type SerializeTupleStruct = SerializeTupleStruct;
    type SerializeTupleVariant = SerializeTupleVariant;
    type SerializeMap = SerializeMap;
    type SerializeStruct = SerializeStruct;
    type SerializeStructVariant = SerializeStructVariant;

    fn serialize_bool(self, v: bool) -> Result<Value, Error> {
        Ok(Value::Bool(v))
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Error> {
        Ok(SerializeStruct {
            name,
            fields: Vec::with_capacity(len),
        })
    }

    // ... etc
}

pub fn to_value<T: Serialize>(value: T) -> Result<Value, Error> {
    value.serialize(ValueSerializer)
}
```

### Phase 5: Deserialize Implementation (Format → Value)
**Files:** `src/de_value.rs`

Implement `serde::Deserialize` for `Value`. This uses `deserialize_any`:

```rust
impl<'de> Deserialize<'de> for Value {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(ValueVisitor)
    }
}

struct ValueVisitor;

impl<'de> Visitor<'de> for ValueVisitor {
    type Value = Value;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "any valid serde value")
    }

    fn visit_bool<E>(self, v: bool) -> Result<Value, E> {
        Ok(Value::Bool(v))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Value, A::Error> {
        let mut entries = Vec::new();
        while let Some((k, v)) = map.next_entry()? {
            entries.push((k, v));
        }
        Ok(Value::Map(entries))
    }

    // ... etc
}
```

**Important limitation:** When deserializing from formats like JSON, struct/field names are lost. The `visit_map` method receives no struct name information. This is fundamental to serde's design - the struct information flows from `deserialize_struct` to the format, not from the format to the visitor.

### Phase 6: Error Type
**Files:** `src/error.rs`

```rust
#[derive(Debug, Clone)]
pub enum Error {
    Message(String),
    TypeMismatch {
        expected: &'static str,
        found: &'static str,
    },
    MissingField(&'static str),
    UnknownField(String),
    DuplicateField(&'static str),
    // Integer coercion (unimplemented for now)
    IntegerCoercion {
        from: &'static str,
        to: &'static str,
    },
}

impl std::error::Error for Error {}
impl serde::ser::Error for Error { ... }
impl serde::de::Error for Error { ... }
```

### Phase 7: Public API
**Files:** `src/lib.rs`

Re-export the complete public API:

```rust
pub use value::Value;
pub use error::Error;

/// Convert any `Serialize` type to `Value`.
/// Uses our custom `ValueSerializer` to capture all serde data model information.
pub fn to_value<T: Serialize>(value: T) -> Result<Value, Error>;

/// Convert `Value` to any `DeserializeOwned` type.
/// Uses our custom `ValueDeserializer` to provide all preserved information.
pub fn from_value<T: DeserializeOwned>(value: Value) -> Result<T, Error>;
```

### Phase 8: Tests
**Files:** `tests/`

1. **Round-trip tests**: Serialize → Value → Serialize for all serde types
2. **Primitive tests**: All 14 primitive types
3. **Struct tests**: Named structs preserve name and field names
4. **Enum tests**: All variant types preserve enum name, variant name, and index
5. **Nested tests**: Complex nested structures
6. **Edge cases**: Empty structs, zero-length tuples, empty maps

## File Structure

```
crates/serde-value/
├── Cargo.toml
├── PLAN.md (this file)
├── src/
│   ├── lib.rs          # Public API, re-exports
│   ├── value.rs        # Value enum definition
│   ├── ser.rs          # Serialize impl for Value
│   ├── de.rs           # Deserializer impl for Value
│   ├── to_value.rs     # Serializer that produces Value
│   ├── de_value.rs     # Deserialize impl for Value (ValueVisitor)
│   └── error.rs        # Error type
└── tests/
    ├── primitives.rs
    ├── structs.rs
    ├── enums.rs
    ├── sequences.rs
    └── round_trip.rs
```

## Cargo.toml

```toml
[package]
name = "serde-value"
description = "Complete in-memory representation of the serde data model"
publish = false
edition.workspace = true
version.workspace = true
license.workspace = true

[dependencies]
serde = { workspace = true }

[dev-dependencies]
serde = { workspace = true, features = ["derive"] }
serde_json = { workspace = true }
```

## Open Implementation Questions

### 1. Float Equality/Hashing
`f32` and `f64` don't implement `Eq` or `Hash` due to NaN. Options:
- Use `total_cmp` for ordering and bit representation for hashing (like `jeb_value::Float`)
- Implement `PartialEq` but not `Eq` (limiting usability)
- **Decision**: Use `total_cmp` approach for `Eq`/`Ord`/`Hash`

### 2. Integer Coercion in Deserializer
When deserializing `Value::I64(42)` into a `u32` field, should this succeed?
- **Decision**: `unimplemented!()` for now, revisit later

### 3. The "Struct Name Problem"
When deserializing from JSON into `Value`, we get `visit_map` not `visit_struct`. The struct name is lost.
- **Decision**: Document this limitation. `Value` preserves information *when it exists*, but can't create information that the source format doesn't provide.

### 4. Static vs Owned Strings for Names
Serde provides `&'static str` for struct/field/variant names. Should we:
- Keep `&'static str` (matches serde, but limits dynamic creation)
- Use `String` (allows dynamic creation, but loses the static guarantee)
- Use `Cow<'static, str>` (flexible but more complex)
- **Decision**: Use `&'static str` as serde provides. Dynamic creation requires `Box::leak` or similar.

## Dependencies

This crate should be **standalone** with only `serde` as a required dependency:

```toml
[dependencies]
serde = { workspace = true }

[dev-dependencies]
serde = { workspace = true, features = ["derive"] }
serde_json = { workspace = true }
```

## Core API Summary

The crate provides four key capabilities:

### 1. `Value` implements `Serialize`
Allows serializing a `Value` to any serde format:
```rust
let value: Value = ...;
let json = serde_json::to_string(&value)?;  // Value → JSON
let yaml = serde_yaml::to_string(&value)?;  // Value → YAML
```

### 2. `Value` implements `Deserialize`
Allows deserializing a `Value` from any serde format:
```rust
let value: Value = serde_json::from_str(json)?;  // JSON → Value
let value: Value = serde_yaml::from_str(yaml)?;  // YAML → Value
```

### 3. `to_value<T: Serialize>(T) -> Value`
Converts any serializable Rust type to `Value`:
```rust
#[derive(Serialize)]
struct User { name: String, age: u32 }

let user = User { name: "Alice".into(), age: 30 };
let value = serde_value::to_value(&user)?;
// value is Value::Struct { name: "User", fields: [...] }
```

### 4. `from_value<T: Deserialize>(Value) -> T`
Converts a `Value` to any deserializable Rust type:
```rust
let value: Value = ...;
let user: User = serde_value::from_value(value)?;
```

### Use Case: Universal Serde Intermediate

This enables `Value` to serve as an intermediate representation between any serde-compatible interfaces:

```rust
// Type → Value → Different Type
let v1_user: UserV1 = ...;
let value = serde_value::to_value(&v1_user)?;
let v2_user: UserV2 = serde_value::from_value(value)?;

// Format → Value → Different Format
let json_str = r#"{"name": "Alice"}"#;
let value: Value = serde_json::from_str(json_str)?;
let yaml_str = serde_yaml::to_string(&value)?;

// Type → Value → Format
let user = User { ... };
let value = serde_value::to_value(&user)?;
let msgpack_bytes = rmp_serde::to_vec(&value)?;
```
