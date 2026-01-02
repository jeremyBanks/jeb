use crate::{
    Bytes,
    Text,
};
impl From<&str> for Bytes {
    fn from(value: &str) -> Self {
        Bytes(value.as_bytes().to_vec())
    }
}
impl From<Text> for Bytes {
    fn from(value: Text) -> Self {
        Bytes(value.0.into_bytes())
    }
}
