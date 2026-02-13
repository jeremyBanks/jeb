//! Extended Z85 encoding with raw ASCII passthrough support.
//!
//! This crate implements an extended version of the Z85 binary-to-text encoding
//! that allows printable ASCII bytes to be passed through raw, reducing overhead
//! for text-heavy data while maintaining the position invariant (complete Z85
//! blocks remain byte-identical to standard Z85 encoding).

pub mod z85;
pub mod extended;
pub mod error;

pub use z85::{encode_z85, decode_z85};
pub use extended::{encode, decode};
pub use error::{Error, Result};
