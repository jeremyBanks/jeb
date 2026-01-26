use serde::de::DeserializeOwned;
use serde::Serialize;

use super::serialization::BlobSerialization;
use super::Postcard;
use crate::blobs::blip::Blip;
use crate::blobs::blob::Blob;

/// A type that can be serialized to/from a [`Blob`] using a
/// [`BlobSerialization`].
///
/// This is automatically implemented for any type that's [`Serialize`] and
/// [`DeserializeOwned`], as well as some special cases including `[u8]` and
/// `str`.
pub trait Blobbable {
    /// Serializes this value into a [`Blob`].
    fn to_blob<Serialization: BlobSerialization>(&self) -> Blob<Self, Serialization>;

    /// Deserialize a [`Blob`] into this type as an owned value if possible,
    /// else None.
    ///
    /// Either `owned_from_blob` or `borrowed_from_blob` must be implemented
    /// and return a non-`None` value.
    fn owned_from_blob<Serialization: BlobSerialization>(
        blob: &Blob<Self, Serialization>,
    ) -> Option<Self>
    where Self: Sized {
        None
    }

    /// Deserialize a [`Blob`] into this type as a reference if possible, else
    /// None.
    fn borrowed_from_blob<Serialization: BlobSerialization>(
        blob: &Blob<Self, Serialization>,
    ) -> Option<&Self> {
        None
    }

    /// Deserializes a [`Blob`] into this type as an owned value.
    fn from_blob<Serialization: BlobSerialization>(blob: &Blob<Self, Serialization>) -> Self
    where Self: Clone {
        Self::owned_from_blob(blob).unwrap_or_else(|| {
            Self::borrowed_from_blob(blob)
                .expect("either blob_to or blob_as must be implemented")
                .clone()
        })
    }

    /// Serialize this value into a [`Blob`] and then return its [`Blip`].
    fn to_blip<Serialization: BlobSerialization>(&self) -> Blip<Self, Serialization> {
        self.to_blob().blip()
    }
}

impl Blobbable for str {
    fn to_blob<Serialization: BlobSerialization>(&self) -> Blob<Self, Serialization> {
        Blob::from_raw_bytes(self.as_ref())
    }

    fn borrowed_from_blob<Serialization: BlobSerialization>(
        blob: &Blob<Self, Serialization>,
    ) -> Option<&Self> {
        std::str::from_utf8(blob.as_ref()).unwrap().into()
    }
}

impl Blobbable for [u8] {
    fn to_blob<Serialization: BlobSerialization>(&self) -> Blob<Self, Serialization> {
        Blob::from_raw_bytes(self)
    }

    fn borrowed_from_blob<Serialization: BlobSerialization>(
        blob: &Blob<Self, Serialization>,
    ) -> Option<&Self> {
        blob.as_ref().into()
    }
}

impl<T> Blobbable for T
where T: Serialize + DeserializeOwned
{
    fn to_blob<Serialization: BlobSerialization>(&self) -> Blob<Self, Serialization> {
        Serialization::serialize_as_blob(self)
    }

    fn owned_from_blob<Serialization: BlobSerialization>(
        blob: &Blob<Self, Serialization>,
    ) -> Option<Self> {
        Some(Serialization::deserialize_from_blob(blob))
    }
}
