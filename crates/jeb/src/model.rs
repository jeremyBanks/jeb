use derive_more::{Deref, DerefMut, From, Into, IntoIterator, TryInto};


#[derive(Debug, Clone, Default)]
pub struct JsonObject(indexmap::IndexMap<String, JsonValue>);

#[derive(Debug, Clone, Default, From, TryInto)]
pub enum JsonValue {
    #[default]
    Null,
    Bool(bool),
    Unsigned(u64),
    Signed(i64),
    Float(f64),
    String(String),
    Array(JsonArray),
    Object(JsonObject),
}

#[derive(Debug, Clone, Default, From, Into, IntoIterator, Deref, DerefMut)]
pub struct JsonArray(Vec<JsonValue>);

pub type Bytes = Vec<u8>;

#[derive(Debug, Clone, From, TryInto)]
pub enum Item {
    JsonValue(JsonValue),
    JsonObject(JsonObject),
    Bytes(Bytes),
    String(String),
}

#[derive(Debug, Clone, Default, From, Into, IntoIterator, Deref, DerefMut)]
pub struct ItemNodeList(Vec<ItemNode>);

#[derive(Debug, Clone, From, TryInto)]
pub enum ItemNode {
    Leaf(Item),
    Branch(ItemNodeList),
}

// MARK: Stream Data Model (Conceptual Model Implementation)

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
    TextMap(indexmap::IndexMap<String, Structured>),

    /// Ordered key-value collection (binary-keyed)
    BinaryMap(indexmap::IndexMap<Vec<u8>, Structured>),
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
