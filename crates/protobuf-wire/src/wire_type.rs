//! Wire type definitions for the protobuf wire format.

use crate::ParseError;

/// The six wire types defined by the protobuf wire format.
///
/// Wire types 3 (SGROUP) and 4 (EGROUP) are deprecated but remain valid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum WireType {
    /// Variable-length integer (1-10 bytes).
    Varint = 0,
    /// Fixed 64-bit value (8 bytes, little-endian).
    I64 = 1,
    /// Length-prefixed bytes.
    Len = 2,
    /// Group start marker (deprecated).
    SGroup = 3,
    /// Group end marker (deprecated).
    EGroup = 4,
    /// Fixed 32-bit value (4 bytes, little-endian).
    I32 = 5,
}

impl WireType {
    /// Try to convert a u8 to a WireType.
    pub fn from_u8(value: u8) -> Result<Self, ParseError> {
        match value {
            0 => Ok(WireType::Varint),
            1 => Ok(WireType::I64),
            2 => Ok(WireType::Len),
            3 => Ok(WireType::SGroup),
            4 => Ok(WireType::EGroup),
            5 => Ok(WireType::I32),
            _ => Err(ParseError::InvalidWireType(value)),
        }
    }
}
