use crate::{Bytes, String};

impl From<&str> for Bytes {
    fn from(value: &str) -> Self {
        Bytes(value.as_bytes().to_vec())
    }
}

impl From<String> for Bytes {
    fn from(value: String) -> Self {
        Bytes(value.into_inner().into_bytes())
    }
}
