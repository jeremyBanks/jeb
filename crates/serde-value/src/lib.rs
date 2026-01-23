//! Complete in-memory representation of the serde data model.
//!
//! This crate provides [`Value`], a type that can represent any value expressible in the
//! [serde data model](https://serde.rs/data-model.html) with complete fidelity. Unlike
//! [`serde_json::Value`](https://docs.rs/serde_json/latest/serde_json/value/enum.Value.html)
//! or similar types, this preserves all information including struct names, field names,
//! enum variant names and indices, and the distinctions between tuples and sequences,
//! structs and maps, etc.
//!
//! # Serialization and Deserialization
//!
//! `Value` serializes and deserializes as a **tagged enum** (like `#[derive(Serialize, Deserialize)]`
//! would produce). This works with **ALL formats** including non-self-describing binary formats
//! like bincode and postcard.
//!
//! ```ignore
//! let value = Value::I32(42);
//!
//! // Roundtrips through ANY format:
//! let bytes = bincode::serialize(&value)?;
//! let restored: Value = bincode::deserialize(&bytes)?;
//! assert_eq!(restored, value);
//! ```
//!
//! # Transparent Serialization
//!
//! To serialize `Value` **transparently** (producing identical bytes to the original type),
//! use [`Transparent`]:
//!
//! ```ignore
//! let original = Point { x: 10, y: 20 };
//! let value = to_value(&original)?;
//!
//! // Transparent produces IDENTICAL bytes:
//! let bytes1 = bincode::serialize(&original)?;
//! let bytes2 = bincode::serialize(&Transparent(value))?;
//! assert_eq!(bytes1, bytes2);
//! ```
//!
//! Note: `Transparent` can only **deserialize** from self-describing formats (JSON, MessagePack, RON)
//! because it uses `deserialize_any`.
//!
//! # Core API
//!
//! ## 1. `Value` implements `Serialize` and `Deserialize`
//!
//! Works with ALL serde formats:
//!
//! ```ignore
//! let value = Value::I32(42);
//! let bytes = bincode::serialize(&value)?;      // Works!
//! let back: Value = bincode::deserialize(&bytes)?;  // Works!
//! ```
//!
//! ## 2. `to_value<T: Serialize>(T) -> Value`
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
//! ## 3. `from_value<T: Deserialize>(Value) -> T`
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
//! ## 4. `Transparent(Value)` - Transparent serialization
//!
//! When you need the serialized output to match the original type exactly:
//!
//! ```ignore
//! use serde_value::{Value, Transparent, to_value};
//!
//! let original = Point { x: 10, y: 20 };
//! let value = to_value(&original)?;
//!
//! // Transparent: identical bytes to original
//! let bytes = bincode::serialize(&Transparent(value))?;
//! let restored: Point = bincode::deserialize(&bytes)?;  // Works!
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
mod transparent;
mod to_value;
mod value;

pub use cast::try_cast;
pub use de::from_value;
pub use error::Error;
pub use transparent::Transparent;
pub use to_value::to_value;
pub use value::Value;

/// Backwards-compatible alias for [`Transparent`].
#[deprecated(since = "0.1.0", note = "Use `Transparent` instead")]
pub type Meta = Transparent;
