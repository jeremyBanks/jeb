use std::borrow::Borrow;
use std::convert::Infallible;
use std::fmt;
use std::fmt::Debug;
use std::fmt::Display;
use std::hash::Hash;
use std::rc::Rc;

use bstr::BStr;
use serde::Deserialize;
use serde::Serialize;
use static_assertions::assert_impl_all;
use thiserror::Error;

use super::BlobSerialization;
use crate::blobs::blob::Blob;
use crate::blobs::serialization::Cbor;
use crate::blobs::serialization::FlexBuffers;
use crate::blobs::serialization::Json;
use crate::blobs::serialization::Postcard;
use crate::copyvec::InlineVec;
use crate::generic::default;
use crate::generic::PhantomType;
use crate::never;
use crate::Blobbable;

/// A [`Blip`] represents a [`Blob`], stored inline if it's under 32 bytes,
/// otherwise represented by its 32-byte BLAKE3 hash digest.
/// This type is `Copy`.
pub struct Blip<T, S>
where
    T: ?Sized,
    S: BlobSerialization,
{
    bytes: InlineVec<u8, 32>,
    t: PhantomType<T>,
    s: PhantomType<S>,
}

assert_impl_all!(Blip<never, Json>: Sized, Copy, Serialize, Sync, Send);
assert_impl_all!(Blip<Infallible, Postcard>: Sized, Copy, Serialize, Sync, Send);
assert_impl_all!(Blip<Rc<u8>, FlexBuffers>: Sized, Copy, Serialize, Sync, Send);
assert_impl_all!(Blip<[u8], Cbor>: Sized, Copy, Serialize, Sync, Send);
assert_impl_all!(Blip<dyn Debug, Postcard>: Sized, Copy, Serialize, Sync, Send);

/// Creates a new [`Blip`] from a reference to any [`Blobbable`] value.
pub fn blip<T, S, Ref>(value: Ref) -> Blip<T, S>
where
    T: Blobbable + ?Sized,
    Ref: Sized + Borrow<T>,
    S: BlobSerialization,
{
    T::to_blip(value.borrow())
}

impl<T, S> Blip<T, S>
where
    T: ?Sized,
    S: BlobSerialization,
{
    /// Creates a new [`Blip`] from a reference to any [`Blobbable`] value.
    pub fn new<Ref>(value: Ref) -> Self
    where
        T: Blobbable,
        Ref: Sized + Borrow<T>,
    {
        T::to_blip(value.borrow())
    }

    /// Returns the Blip representing a given Blob.
    pub fn for_blob(blob: &Blob<T, S>) -> Self {
        blob.blip()
    }

    /// Creates a blip from the corresponding raw bytes (either a hash or an
    /// inline value, depending on length).
    pub fn try_from_raw_bytes(blip_bytes: &[u8]) -> Result<Self, TooLongForBlipError> {
        Ok(Self {
            bytes: InlineVec::try_from_slice(blip_bytes)
                .map_err(|_| TooLongForBlipError(blip_bytes.len()))?,
            ..default()
        })
    }
    pub(crate) fn retype<R: ?Sized, Q: BlobSerialization>(self) -> Blip<R, Q> {
        Blip {
            bytes: self.bytes,
            ..default()
        }
    }

    /// Whether this Blip contains an inline value.
    pub fn is_inline(&self) -> bool {
        self.bytes.len() < 32
    }

    /// Whether this Blip contains a hash identifying external-stored content.
    pub fn is_hash(&self) -> bool {
        self.bytes.len() == 32
    }

    /// Returns the inline value as a Blob, if present.
    pub fn inline_blob(&self) -> Option<Blob<T, S>> {
        if self.is_inline() {
            Some(Blob::from_raw_bytes(self.bytes.as_ref()))
        } else {
            None
        }
    }
}

impl<T, S> Copy for Blip<T, S>
where
    T: ?Sized,
    S: BlobSerialization,
{
}

impl<T, S> Blob<T, S>
where
    T: ?Sized,
    S: BlobSerialization,
{
    /// Returns the Blip representing this Blob.
    pub fn blip(&self) -> Blip<T, S> {
        let bytes = self.as_ref();
        if bytes.len() <= 31 {
            Blip::try_from_raw_bytes(bytes).unwrap()
        } else {
            let bytes = blake3::hash(bytes).as_bytes().to_vec();
            Blip::try_from_raw_bytes(bytes.as_ref()).unwrap()
        }
    }
}

#[derive(Debug, Error, Copy, Clone)]
#[error("Blips must be between 0 and 32 bytes, but the input was {0} bytes.")]
/// Blips must be between 0 and 32 bytes, but the input was longer.
pub struct TooLongForBlipError(usize);

impl<T, S> From<Blob<T, S>> for Blip<T, S>
where
    T: ?Sized,
    S: BlobSerialization,
{
    fn from(value: Blob<T, S>) -> Self {
        value.blip()
    }
}

impl<T, S> From<&Blob<T, S>> for Blip<T, S>
where
    T: ?Sized,
    S: BlobSerialization,
{
    fn from(value: &Blob<T, S>) -> Self {
        value.blip()
    }
}

#[derive(Debug, Error, Copy, Clone)]
/// This Blip represents a value that's too long to store inline.
#[error("This Blip represents a value that's too long to store inline.")]
pub struct TooLongForInlineBlipError;

impl<T, S> TryFrom<Blip<T, S>> for Blob<T, S>
where
    T: ?Sized,
    S: BlobSerialization,
{
    type Error = TooLongForInlineBlipError;

    fn try_from(value: Blip<T, S>) -> Result<Self, Self::Error> {
        value.inline_blob().ok_or(TooLongForInlineBlipError)
    }
}

impl<T, S> Hash for Blip<T, S>
where
    T: ?Sized,
    S: BlobSerialization,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.bytes.hash(state);
    }
}

impl<T, S> PartialEq for Blip<T, S>
where
    T: ?Sized,
    S: BlobSerialization,
{
    fn eq(&self, other: &Self) -> bool {
        self.bytes.eq(&other.bytes)
    }
}

impl<T, S> Eq for Blip<T, S>
where
    T: ?Sized,
    S: BlobSerialization,
{
}

impl<T, S> PartialOrd for Blip<T, S>
where
    T: ?Sized,
    S: BlobSerialization,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.bytes.partial_cmp(&other.bytes)
    }
}

impl<T, S> Ord for Blip<T, S>
where
    T: ?Sized,
    S: BlobSerialization,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.bytes.cmp(&other.bytes)
    }
}

impl<T, S> Debug for Blip<T, S>
where
    T: ?Sized,
    S: BlobSerialization,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(BStr::new(&self.bytes), f)
    }
}

impl<T, S> Display for Blip<T, S>
where
    T: ?Sized,
    S: BlobSerialization,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(BStr::new(&self.bytes), f)
    }
}

impl<T, S> Serialize for Blip<T, S>
where
    T: ?Sized,
    S: BlobSerialization,
{
    fn serialize<Ser>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error>
    where Ser: serde::Serializer {
        serde_bytes::Bytes::new(self.bytes.as_ref()).serialize(serializer)
    }
}

impl<'de, T, S> Deserialize<'de> for Blip<T, S>
where
    T: ?Sized,
    S: BlobSerialization,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        let bytes = serde_bytes::ByteBuf::deserialize(deserializer)?.into_vec();
        Self::try_from_raw_bytes(&bytes).map_err(|err| serde::de::Error::custom(err.to_string()))
    }
}

impl<T, S> Default for Blip<T, S>
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

impl<T, S> Clone for Blip<T, S>
where
    T: ?Sized,
    S: BlobSerialization,
{
    fn clone(&self) -> Self {
        Self {
            bytes: self.bytes,
            ..default()
        }
    }
}

impl<T, S> AsRef<[u8]> for Blip<T, S>
where
    T: ?Sized,
    S: BlobSerialization,
{
    fn as_ref(&self) -> &[u8] {
        self.bytes.as_ref()
    }
}

impl<T, S> PartialEq<Blob<T, S>> for Blip<T, S>
where
    T: ?Sized,
    S: BlobSerialization,
{
    fn eq(&self, other: &Blob<T, S>) -> bool {
        self == &other.blip()
    }
}

impl<T, S> PartialEq<Blip<T, S>> for Blob<T, S>
where
    T: ?Sized,
    S: BlobSerialization,
{
    fn eq(&self, other: &Blip<T, S>) -> bool {
        &self.blip() == other
    }
}

impl<T, S> PartialEq<T> for Blip<T, S>
where
    T: ?Sized + Blobbable,
    S: BlobSerialization,
{
    fn eq(&self, other: &T) -> bool {
        self == &other.to_blip()
    }
}
