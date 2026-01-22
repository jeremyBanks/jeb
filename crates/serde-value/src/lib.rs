//! Complete in-memory representation of the serde data model.
//!
//! This crate provides [`Value`], a type that can represent any value expressible in the
//! [serde data model](https://serde.rs/data-model.html) with complete fidelity. Unlike
//! [`serde_json::Value`](https://docs.rs/serde_json/latest/serde_json/value/enum.Value.html)
//! or similar types, this preserves all information including struct names, field names,
//! enum variant names and indices, and the distinctions between tuples and sequences,
//! structs and maps, etc.
//!
//! # Core API
//!
//! The crate provides four key capabilities:
//!
//! ## 1. `Value` implements `Serialize`
//!
//! Serialize a [`Value`] to any serde format:
//!
//! ```ignore
//! let value: Value = /* ... */;
//! let json = serde_json::to_string(&value)?;  // Value → JSON
//! ```
//!
//! ## 2. `Value` implements `Deserialize`
//!
//! Deserialize a [`Value`] from any serde format:
//!
//! ```ignore
//! let value: Value = serde_json::from_str(json)?;  // JSON → Value
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

mod de;
mod de_value;
mod error;
mod ser;
mod to_value;
mod value;

pub use de::from_value;
pub use error::Error;
pub use to_value::to_value;
pub use value::Value;
