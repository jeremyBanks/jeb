use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::blobs::blob::Blob;
use crate::Blobbable;

/// A [`serde`] implementation that can be used to serialize a [`Blobbable`]
/// into a [`Blob`]/[`Blip`][crate::blobs::blip::Blip].
pub trait BlobSerialization: Sized {
    /// Serializes a value as a Blob using this serialization implementation.
    fn serialize_as_blob<T: Serialize + Blobbable>(value: &T) -> Blob<T, Self> {
        Blob::<[u8], Self>::from_raw_bytes(&Self::serialize_as_bytes(value)).retype()
    }
    /// Serializes a value as a bytes using this serialization implementation.
    fn serialize_as_bytes<T: Serialize + Blobbable>(value: &T) -> Vec<u8>;
    /// Deserializes a value from a Blob using this serialization
    /// implementation.
    fn deserialize_from_blob<T: DeserializeOwned + Blobbable>(value: &Blob<T, Self>) -> T {
        Self::deserialize_from_bytes(value.as_ref())
    }
    /// Deserializes a value from a bytes using this serialization
    /// implementation.
    fn deserialize_from_bytes<T: DeserializeOwned + Blobbable>(value: &[u8]) -> T;
}

/// A non-functional/panicking [`BlobSerialization`] implementation for use when
/// the correct serialization is unknown.
pub enum Unknown {}
impl BlobSerialization for Unknown {
    fn serialize_as_bytes<T: Serialize + Blobbable>(_value: &T) -> Vec<u8> {
        panic!("Attempted to serialize with an Unknown serialization")
    }
    fn deserialize_from_bytes<T: DeserializeOwned + Blobbable>(_value: &[u8]) -> T {
        panic!("Attempted to deserialize with an Unknown serialization")
    }
}

/// [`postcard`]
pub enum Postcard {}
impl BlobSerialization for Postcard {
    fn serialize_as_bytes<T: Serialize + Blobbable>(value: &T) -> Vec<u8> {
        postcard::to_allocvec(value).unwrap()
    }
    fn deserialize_from_bytes<T: DeserializeOwned + Blobbable>(value: &[u8]) -> T {
        postcard::from_bytes(value).unwrap()
    }
}

/// [`serde_json`]
pub enum Json {}
impl BlobSerialization for Json {
    fn serialize_as_bytes<T: Serialize + Blobbable>(value: &T) -> Vec<u8> {
        serde_json::to_vec(value).unwrap()
    }
    fn deserialize_from_bytes<T: DeserializeOwned + Blobbable>(value: &[u8]) -> T {
        serde_json::from_slice(value).unwrap()
    }
}

/// [`flexbuffers`]
pub enum FlexBuffers {}
impl BlobSerialization for FlexBuffers {
    fn serialize_as_bytes<T: Serialize + Blobbable>(value: &T) -> Vec<u8> {
        flexbuffers::to_vec(value).unwrap()
    }
    fn deserialize_from_bytes<T: DeserializeOwned + Blobbable>(value: &[u8]) -> T {
        flexbuffers::from_slice(value).unwrap()
    }
}

/// [`serde_cbor`]
pub enum Cbor {}
impl BlobSerialization for Cbor {
    fn serialize_as_bytes<T: Serialize + Blobbable>(value: &T) -> Vec<u8> {
        serde_cbor::to_vec(value).unwrap()
    }
    fn deserialize_from_bytes<T: DeserializeOwned + Blobbable>(value: &[u8]) -> T {
        serde_cbor::from_slice(value).unwrap()
    }
}
