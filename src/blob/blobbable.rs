use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::blob::Blip;
use crate::blob::Blob;

/// A type that can be serialized to/from a [`Blob`].
///
/// This is automatically implemented for any type that's [`Serialize`] and
/// [`DeserializeOwned`], which are serialized using [`postcard`],
pub trait Blobbable {
    /// Serializes this value into a [`Blob`].
    fn to_blob(&self) -> Blob<Self>;

    /// Deserialize a [`Blob`] into this type as an owned value if possible,
    /// else None.
    fn owned_from_blob(blob: &Blob<Self>) -> Option<Self>
    where Self: Sized {
        None
    }

    /// Deserialize a [`Blob`] into this type as a reference if possible, else
    /// None.
    fn borrowed_from_blob(blob: &Blob<Self>) -> Option<&Self> {
        None
    }

    /// Deserializes a [`Blob`] into this type as an owned value.
    fn from_blob(blob: &Blob<Self>) -> Self
    where Self: Clone {
        Self::owned_from_blob(blob).unwrap_or(
            Self::borrowed_from_blob(&blob)
                .expect("either blob_to or blob_as must be implemented")
                .clone(),
        )
    }

    /// Serialize this value into a [`Blob`] and then return its [`Blip`].
    fn to_blip(&self) -> Blip<Self> {
        self.to_blob().blip()
    }
}

impl Blobbable for str {
    fn to_blob(&self) -> Blob<Self> {
        Blob::from_raw_bytes(self.as_ref())
    }

    fn borrowed_from_blob(blob: &Blob<Self>) -> Option<&Self> {
        std::str::from_utf8(blob.as_ref()).unwrap().into()
    }
}

impl Blobbable for [u8] {
    fn to_blob(&self) -> Blob<Self> {
        Blob::from_raw_bytes(self.as_ref())
    }

    fn borrowed_from_blob(blob: &Blob<Self>) -> Option<&Self> {
        blob.as_ref().into()
    }
}

impl<T> Blobbable for T
where T: Serialize + DeserializeOwned
{
    fn to_blob(&self) -> Blob<Self> {
        Blob::from_raw_vec(postcard::to_stdvec(self).unwrap())
    }

    fn owned_from_blob(blob: &Blob<Self>) -> Option<Self> {
        Some(postcard::from_bytes::<Self>(blob.as_ref()).unwrap())
    }
}
