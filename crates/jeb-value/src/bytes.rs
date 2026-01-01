use {
    crate::text::Text, core::hash::Hash,
    derive_more::{
        AsMut, AsRef, Deref, DerefMut, From, Index, IndexMut, Into, IntoIterator,
    },
};
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
#[derive(
    AsMut,
    AsRef,
    Clone,
    Debug,
    Default,
    Deref,
    DerefMut,
    Eq,
    From,
    Hash,
    Index,
    IndexMut,
    Into,
    IntoIterator,
    Ord,
    PartialEq,
    PartialOrd,
)]
#[repr(transparent)]
#[as_ref(Vec<u8>, [u8])]
#[into_iterator(owned, ref, ref_mut)]
#[must_use]
pub struct Bytes(pub(crate) Vec<u8>);
impl From<&[u8]> for Bytes {
    fn from(value: &[u8]) -> Self {
        Bytes(value.to_vec())
    }
}
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
#[cfg(feature = "serde")]
impl serde::Serialize for Bytes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(self.as_ref())
    }
}
#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Bytes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct BytesVisitor;
        impl<'de> serde::de::Visitor<'de> for BytesVisitor {
            type Value = Bytes;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a byte array")
            }
            fn visit_bytes<E>(self, v: &[u8]) -> Result<Bytes, E>
            where
                E: serde::de::Error,
            {
                Ok(Bytes::from(v.to_vec()))
            }
            fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Bytes, E>
            where
                E: serde::de::Error,
            {
                Ok(Bytes::from(v))
            }
            fn visit_seq<A>(self, mut seq: A) -> Result<Bytes, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut bytes = Vec::new();
                while let Some(byte) = seq.next_element()? {
                    bytes.push(byte);
                }
                Ok(Bytes::from(bytes))
            }
        }
        deserializer.deserialize_bytes(BytesVisitor)
    }
}
