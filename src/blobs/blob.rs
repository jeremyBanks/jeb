use std::borrow::Borrow;
use std::convert::Infallible;
use std::fmt;
use std::fmt::Debug;
use std::fmt::Display;
use std::hash::Hash;
use std::ops::Deref;
use std::ops::DerefMut;
use std::rc::Rc;

use bstr::BStr;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde::Serialize;
use static_assertions::assert_impl_all;

use super::bytes;
use super::BlobSerialization;
use super::Blobbable;
use crate::blobs::Cbor;
use crate::blobs::FlexBuffers;
use crate::blobs::Json;
use crate::blobs::Postcard;
use crate::default;
use crate::never;
use crate::PhantomType;

/// A [`Blob`] is a byte vector, containing the binary serialization of a
/// value of the given type `T`.
pub struct Blob<T, S>
where
    T: ?Sized,
    S: BlobSerialization,
{
    bytes: Vec<u8>,
    t: PhantomType<T>,
    s: PhantomType<S>,
}

assert_impl_all!(Blob<never, Postcard>: Sized, Serialize, DeserializeOwned, Sync, Send);
assert_impl_all!(Blob<Infallible, Json>: Sized, Serialize, DeserializeOwned, Sync, Send);
assert_impl_all!(Blob<Rc<u8>, Cbor>: Sized, Serialize, DeserializeOwned, Sync, Send);
assert_impl_all!(Blob<[u8], FlexBuffers>: Sized, Serialize, DeserializeOwned, Sync, Send);
assert_impl_all!(Blob<dyn Debug, Postcard>: Sized, Serialize, DeserializeOwned, Sync, Send);

/// Creates a new [`Blob`] from a reference to any [`Blobbable`] value.
pub fn blob<T, S, Ref>(value: Ref) -> Blob<T, S>
where
    T: Blobbable + ?Sized,
    Ref: Sized + Borrow<T>,
    S: BlobSerialization,
{
    T::to_blob(value.borrow())
}

impl<T, S> Blob<T, S>
where
    T: ?Sized,
    S: BlobSerialization,
{
    /// Creates a new blob from a reference to any blobbable value.
    pub fn new<Ref>(value: Ref) -> Self
    where
        T: Blobbable,
        Ref: Sized + Borrow<T>,
    {
        T::to_blob(value.borrow())
    }

    /// Creates a blob from bytes directly, without making sure the type
    /// matches.
    ///
    /// This is memory-safe, but might cause a panic if you use the wrong type.
    pub fn from_raw_bytes(bytes: &[u8]) -> Self {
        Self::from_raw_vec(bytes.to_vec())
    }

    /// Creates a blob from bytes directly, without making sure the type
    /// matches.
    ///
    /// This is memory-safe, but might cause a panic if you use the wrong type.
    pub fn from_raw_vec(bytes: Vec<u8>) -> Self {
        Self { bytes, ..default() }
    }

    pub(crate) fn retype<R: ?Sized, Q: BlobSerialization>(self) -> Blob<R, Q> {
        Blob {
            bytes: self.bytes,
            ..default()
        }
    }
}

impl<T, S> Hash for Blob<T, S>
where
    T: ?Sized,
    S: BlobSerialization,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.bytes.hash(state);
    }
}

impl<T, S> PartialEq for Blob<T, S>
where
    T: ?Sized,
    S: BlobSerialization,
{
    fn eq(&self, other: &Self) -> bool {
        self.bytes.eq(&other.bytes)
    }
}

impl<T, S> Eq for Blob<T, S>
where
    T: ?Sized,
    S: BlobSerialization,
{
}

impl<T, S> PartialOrd for Blob<T, S>
where
    T: ?Sized,
    S: BlobSerialization,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.bytes.partial_cmp(&other.bytes)
    }
}

impl<T, S> Ord for Blob<T, S>
where
    T: ?Sized,
    S: BlobSerialization,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.bytes.cmp(&other.bytes)
    }
}

impl<T, S> Debug for Blob<T, S>
where
    T: ?Sized,
    S: BlobSerialization,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(BStr::new(&self.bytes), f)
    }
}

impl<T, S> Display for Blob<T, S>
where
    T: ?Sized,
    S: BlobSerialization,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(BStr::new(&self.bytes), f)
    }
}

impl<T, S> Serialize for Blob<T, S>
where
    T: ?Sized,
    S: BlobSerialization,
{
    fn serialize<Ser>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error>
    where Ser: serde::Serializer {
        serde_bytes::Bytes::new(self.bytes.as_ref()).serialize(serializer)
    }
}

impl<'de, T, S> Deserialize<'de> for Blob<T, S>
where
    T: ?Sized,
    S: BlobSerialization,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        let serialized = serde_bytes::ByteBuf::deserialize(deserializer)?.into_vec();
        Ok(Blob::from_raw_bytes(serialized.as_ref()))
    }
}

impl<T, S> Default for Blob<T, S>
where
    T: ?Sized,
    S: BlobSerialization,
{
    fn default() -> Self {
        Self {
            bytes: default(),
            t: default(),
            s: default(),
        }
    }
}

impl<T, S> Clone for Blob<T, S>
where
    T: ?Sized,
    S: BlobSerialization,
{
    fn clone(&self) -> Self {
        Self {
            bytes: self.bytes.clone(),
            ..default()
        }
    }
}

impl<T, S> AsRef<[u8]> for Blob<T, S>
where
    T: ?Sized,
    S: BlobSerialization,
{
    fn as_ref(&self) -> &[u8] {
        self.bytes.as_ref()
    }
}

impl<T, S> AsMut<[u8]> for Blob<T, S>
where
    T: ?Sized,
    S: BlobSerialization,
{
    fn as_mut(&mut self) -> &mut [u8] {
        self.bytes.as_mut()
    }
}

impl<T, S> Deref for Blob<T, S>
where
    T: ?Sized,
    S: BlobSerialization,
{
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        self.as_ref()
    }
}

impl<T, S> DerefMut for Blob<T, S>
where
    T: ?Sized,
    S: BlobSerialization,
{
    fn deref_mut(&mut self) -> &mut [u8] {
        self.as_mut()
    }
}

impl<T, S> PartialEq<T> for Blob<T, S>
where
    T: ?Sized + Blobbable,
    S: BlobSerialization,
{
    fn eq(&self, other: &T) -> bool {
        self == &other.to_blob()
    }
}
