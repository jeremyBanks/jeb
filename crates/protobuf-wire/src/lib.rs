//! A concrete data model for the Protocol Buffers wire format.
//!
//! This crate provides types to represent any valid protobuf wire-format message
//! with complete fidelity. The model captures exactly what exists on the wire—
//! field numbers, wire types, and raw payloads—without any schema interpretation.
//!
//! # Example
//!
//! ```
//! use protobuf_wire::{Message, Record, Value};
//!
//! // Create a message
//! let message = Message::from_records(vec![
//!     Record::new(1, Value::Varint(42)),
//!     Record::new(2, Value::LenDelimited(b"hello".to_vec())),
//! ]);
//!
//! // Serialize to bytes
//! let bytes = message.serialize().unwrap();
//!
//! // Parse back
//! let parsed = Message::parse(&bytes).unwrap();
//! assert_eq!(message, parsed);
//! ```

#[cfg(feature = "arbitrary")]
mod arbitrary;
mod error;
mod parse;
mod serialize;
mod types;
mod varint;
mod wire_type;

pub use error::{ParseError, SerializeError};
pub use types::{Message, Record, Value};
pub use wire_type::WireType;
