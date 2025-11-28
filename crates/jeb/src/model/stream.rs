// Pipeline stream model types for the synchronous pipeline system

use indexmap::IndexMap;

/// Stream item types as defined in the conceptual model.
/// Each item in a stream has one of three top-level types.
#[derive(Debug, Clone)]
pub enum StreamItem {
    /// Unparsed textual data (UTF-8 Unicode string).
    /// Represents "raw character data that hasn't been interpreted as a data structure yet."
    Text(String),

    /// Unparsed binary data (byte array).
    /// This is the rawest form of data in the system.
    Bytes(Vec<u8>),

    /// Parsed structured data in an extended JSON data model.
    Structured(Structured),
}

/// Structured data type with extended JSON model.
/// Supports multiple number types, text and binary strings, and ordered maps.
#[derive(Debug, Clone)]
pub enum Structured {
    /// Null value
    Null,

    /// Boolean values
    Bool(bool),

    /// 64-bit signed integer
    SignedInt(i64),

    /// 64-bit unsigned integer
    UnsignedInt(u64),

    /// 64-bit finite float
    Float(f64),

    /// Text string (Unicode/UTF-8)
    TextString(String),

    /// Binary string (byte array)
    BinaryString(Vec<u8>),

    /// Ordered sequence of structured values
    Array(Vec<Structured>),

    /// Ordered key-value collection (text-keyed)
    TextMap(IndexMap<String, Structured>),

    /// Ordered key-value collection (binary-keyed)
    BinaryMap(IndexMap<Vec<u8>, Structured>),
}

/// Error value sent through the error pipeline.
/// Maps with text keys containing node index, message, and context.
#[derive(Debug, Clone)]
pub struct ErrorValue {
    /// Index of the node that produced this error
    pub node_index: usize,

    /// Error message
    pub message: String,

    /// Optional context information
    pub context: Option<Structured>,
}

/// Warning information for implicit coercions
#[derive(Debug, Clone)]
pub struct Warning {
    /// Index of the node that produced this warning
    pub node_index: usize,

    /// Warning message describing the coercion
    pub message: String,

    /// The coercion that was performed
    pub coercion: CoercionKind,
}

/// Types of coercions that can occur
#[derive(Debug, Clone)]
pub enum CoercionKind {
    /// Bytes converted to Text
    BytesToText,

    /// Text converted to Structured (wrapped as string)
    TextToStructured,

    /// Bytes converted to Structured (wrapped as string)
    BytesToStructured,

    /// Structured unwrapped to Text
    StructuredToText,

    /// Structured unwrapped to Bytes
    StructuredToBytes,

    /// Custom coercion with description
    Custom(String),
}
