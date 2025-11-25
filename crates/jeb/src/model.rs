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
