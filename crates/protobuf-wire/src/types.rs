//! Core types for the protobuf wire format data model.

/// A wire-format message: a sequence of records.
#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    pub records: Vec<Record>,
}

impl Message {
    /// Create a new empty message.
    pub fn new() -> Self {
        Message {
            records: Vec::new(),
        }
    }

    /// Create a message from a vector of records.
    pub fn from_records(records: Vec<Record>) -> Self {
        Message { records }
    }
}

impl Default for Message {
    fn default() -> Self {
        Self::new()
    }
}

/// A single record: a field number and its value.
#[derive(Debug, Clone, PartialEq)]
pub struct Record {
    pub field_number: u32,
    pub value: Value,
}

impl Record {
    /// Create a new record.
    pub fn new(field_number: u32, value: Value) -> Self {
        Record {
            field_number,
            value,
        }
    }
}

/// A wire-format value. The variant determines the wire type.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Wire type 0: VARINT
    /// Stored as decoded u64. Non-canonical encodings are normalized.
    Varint(u64),

    /// Wire type 1: I64
    /// Fixed 64-bit value, stored as native integer (wire format is
    /// little-endian).
    I64(i64),

    /// Wire type 2: LEN
    /// The raw bytes after the length prefix.
    LenDelimited(Vec<u8>),

    /// Wire types 3/4: Group
    /// The records between SGROUP and EGROUP tags.
    Group(Vec<Record>),

    /// Wire type 5: I32
    /// Fixed 32-bit value, stored as native integer (wire format is
    /// little-endian).
    I32(i32),
}
