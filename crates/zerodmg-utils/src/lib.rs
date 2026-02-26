#![warn(missing_docs, missing_debug_implementations)]

//! # zerodmg-utils
//!
//! Utility functions for the zerodmg Game Boy emulator.
//!
//! ## Modules
//!
//! - [`little_endian`]: Functions for working with little-endian binary data,
//!   including byte splitting/combining and bit manipulation.
//!
//! ## Testing
//!
//! This crate uses both doc-tests and property-based testing (via proptest)
//! to verify invariants like roundtrip conversions and bit isolation.

/// Functions for working with little-endian binary data.
///
/// The Game Boy uses little-endian byte ordering for 16-bit values.
/// These utilities help convert between byte sequences and integers.
///
/// Argument lists and return tuples are least-significant-first.
/// Don't forget that Rust's hex integer literals are big-endian!
///
/// # Examples
///
/// ```rust
/// use zerodmg_utils::little_endian::{u8s_to_u16, u16_to_u8s};
///
/// // Combine two bytes into a u16
/// let value = u8s_to_u16(0x34, 0x12);
/// assert_eq!(value, 0x1234);
///
/// // Split a u16 back into bytes
/// let (lo, hi) = u16_to_u8s(0x1234);
/// assert_eq!(lo, 0x34);
/// assert_eq!(hi, 0x12);
/// ```
pub mod little_endian;
