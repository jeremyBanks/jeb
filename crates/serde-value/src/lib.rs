//! Complete in-memory representation of the serde data model.
//!
//! This crate provides [`Value`], a type that can represent any value expressible in the
//! [serde data model](https://serde.rs/data-model.html) with complete fidelity. Unlike
//! [`serde_json::Value`](https://docs.rs/serde_json/latest/serde_json/value/enum.Value.html)
//! or similar types, this preserves all information including struct names, field names,
//! enum variant names and indices, and the distinctions between tuples and sequences,
//! structs and maps, etc.
//!
//! # Transparent Serialization (Primary Design Goal)
//!
//! **The core guarantee:** When you convert a value to [`Value`] and then serialize it,
//! you get **identical bytes** as if you had serialized the original value directly.
//!
//! This makes [`Value`] a fully transparent intermediate representation that interoperates
//! with ANY serialization format, including non-self-describing binary formats like bincode:
//!
//! ```ignore
//! let original = Point { x: 10, y: 20 };
//!
//! // These produce IDENTICAL bytes:
//! let bytes1 = bincode::serialize(&original)?;
//! let bytes2 = bincode::serialize(&to_value(&original)?)?;
//! assert_eq!(bytes1, bytes2);
//!
//! // So you can deserialize back to the original type:
//! let restored: Point = bincode::deserialize(&bytes2)?;
//! ```
//!
//! This enables workflows like: capture typed data → manipulate as Value → serialize
//! for transmission → receiver deserializes to their typed representation.
//!
//! # Deserialization and `deserialize_any`
//!
//! `Value`'s `Deserialize` impl uses serde's [`Deserializer::deserialize_any`] method,
//! which asks the format: "what type do you have?". This works with **self-describing
//! formats** that embed type information in the byte stream:
//!
//! - **JSON**: `{"x": 42}` → the format knows it's an object with string keys
//! - **MessagePack**: type tags precede each value
//! - **RON**: Rust-like syntax with explicit types
//!
//! **Non-self-describing formats** (bincode, postcard) cannot answer this question - they
//! return an error from `deserialize_any` because the byte stream doesn't contain type
//! information. For these formats, you must deserialize to a known type first, then use
//! [`to_value`] to convert.
//!
//! If you need to serialize and deserialize `Value` itself through non-self-describing
//! formats, use [`Meta`] which wraps `Value` with explicit type tags.
//!
//! # Core API
//!
//! The crate provides four key capabilities:
//!
//! ## 1. `Value` implements `Serialize` (Transparent)
//!
//! Serialize a [`Value`] to any serde format. The output is **identical** to serializing
//! the original value directly - no enum wrappers or type tags are added:
//!
//! ```ignore
//! let value = to_value(&my_struct)?;
//! let bytes = bincode::serialize(&value)?;  // Same bytes as bincode::serialize(&my_struct)
//! ```
//!
//! ## 2. `Value` implements `Deserialize` (requires `deserialize_any`)
//!
//! Deserialize a [`Value`] from self-describing serde formats (JSON, RON, MessagePack, etc.):
//!
//! ```ignore
//! let value: Value = serde_json::from_str(json)?;  // JSON → Value (works!)
//! let value: Value = bincode::deserialize(&bytes)?;  // ERROR: deserialize_any not supported
//! ```
//!
//! ## 3. `to_value<T: Serialize>(T) -> Value`
//!
//! Convert any serializable Rust type to [`Value`], preserving all metadata:
//!
//! ```
//! use serde::Serialize;
//! use serde_value::{to_value, Value};
//!
//! #[derive(Serialize)]
//! struct User { name: String, age: u32 }
//!
//! let user = User { name: "Alice".into(), age: 30 };
//! let value = to_value(&user).unwrap();
//!
//! // value is Value::Struct { name: "User", fields: [...] }
//! match &value {
//!     Value::Struct { name, fields } => {
//!         assert_eq!(*name, "User");
//!         assert_eq!(fields.len(), 2);
//!     }
//!     _ => panic!("expected struct"),
//! }
//! ```
//!
//! ## 4. `from_value<T: Deserialize>(Value) -> T`
//!
//! Convert a [`Value`] to any deserializable Rust type:
//!
//! ```
//! use serde::Deserialize;
//! use serde_value::{from_value, to_value};
//!
//! #[derive(Deserialize, PartialEq, Debug)]
//! struct User { name: String, age: u32 }
//!
//! # #[derive(serde::Serialize)]
//! # struct User2 { name: String, age: u32 }
//! # let user2 = User2 { name: "Alice".into(), age: 30 };
//! # let value = to_value(&user2).unwrap();
//! // Given a Value...
//! let user: User = from_value(value).unwrap();
//! assert_eq!(user.name, "Alice");
//! assert_eq!(user.age, 30);
//! ```
//!
//! ## 5. `Meta(Value)` - Tagged serialization for Value↔Value roundtrip
//!
//! When you need to serialize `Value` itself (not transparently), use [`Meta`]:
//!
//! ```ignore
//! use serde_value::{Value, Meta};
//!
//! let value = Value::I32(42);
//!
//! // Transparent: serializes as just 42
//! let bytes1 = bincode::serialize(&value)?;
//!
//! // Tagged: serializes as Value::I32(42) with enum discriminant
//! let bytes2 = bincode::serialize(&Meta(value.clone()))?;
//! let Meta(restored) = bincode::deserialize(&bytes2)?;  // Works!
//! assert_eq!(restored, value);
//! ```
//!
//! # Use Case: Universal Serde Intermediate
//!
//! [`Value`] can serve as an intermediate representation between any serde-compatible
//! interfaces:
//!
//! ```ignore
//! // Type → Value → Different Type (schema migration)
//! let v1_user: UserV1 = /* ... */;
//! let value = serde_value::to_value(&v1_user)?;
//! let v2_user: UserV2 = serde_value::from_value(value)?;
//!
//! // Type → Value → Format
//! let user = User { /* ... */ };
//! let value = serde_value::to_value(&user)?;
//! let json = serde_json::to_string(&value)?;
//! ```
//!
//! # The Serde Data Model
//!
//! [`Value`] covers all 29 types in the serde data model:
//!
//! - **Primitives (14):** `bool`, `i8`-`i128`, `u8`-`u128`, `f32`, `f64`, `char`
//! - **String and Bytes (2):** `String`, `Bytes`
//! - **Option (2):** `None`, `Some`
//! - **Unit Types (2):** `Unit`, `UnitStruct`
//! - **Newtype (2):** `NewtypeStruct`, `NewtypeVariant`
//! - **Sequences (4):** `Seq`, `Tuple`, `TupleStruct`, `TupleVariant`
//! - **Maps and Structs (3):** `Map`, `Struct`, `StructVariant`
//! - **Unit Variant (1):** `UnitVariant`

#[cfg(feature = "arbitrary")]
mod arbitrary;
mod cast;
mod de;
mod de_value;
mod error;
mod intern;
mod meta;
mod ser;
mod to_value;
mod value;

pub use cast::try_cast;
pub use de::from_value;
pub use error::Error;
pub use meta::Meta;
pub use to_value::to_value;
pub use value::Value;
