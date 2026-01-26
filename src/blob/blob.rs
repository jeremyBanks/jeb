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

use super::Blobbable;
use crate::generic::default;
use crate::generic::Type;
use crate::never;

/// A blob is a byte vector, containing the Postcard binary serialization of a
/// value of a given type, or else a raw byte or character string (`Blob<[u8]>`,
/// `Blob<str>`).
pub struct Blob<T>
where T: ?Sized
{
    bytes: Vec<u8>,
    t: Type<T>,
}

assert_impl_all!(Blob<never>: Sized, Serialize, DeserializeOwned, Sync, Send);
assert_impl_all!(Blob<Infallible>: Sized, Serialize, DeserializeOwned, Sync, Send);
assert_impl_all!(Blob<Rc<u8>>: Sized, Serialize, DeserializeOwned, Sync, Send);
assert_impl_all!(Blob<[u8]>: Sized, Serialize, DeserializeOwned, Sync, Send);
assert_impl_all!(Blob<dyn Debug>: Sized, Serialize, DeserializeOwned, Sync, Send);

impl<T> Blob<T>
where T: ?Sized
{
    /// Creates a new blob from a reference to any blobbable value.
    pub fn new<Ref>(value: Ref) -> Self
    where
        T: Blobbable,
        Ref: Sized + Borrow<T>,
    {
        value.borrow().to_blob()
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
        Self {
            bytes,
            t: default(),
        }
    }

    fn retype<R: ?Sized>(self) -> Blob<R> {
        Blob {
            bytes: self.bytes,
            t: default(),
        }
    }
}

impl<T> Hash for Blob<T>
where T: ?Sized
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.bytes.hash(state);
    }
}

impl<T> PartialEq for Blob<T>
where T: ?Sized
{
    fn eq(&self, other: &Self) -> bool {
        self.bytes.eq(&other.bytes)
    }
}

impl<T> Eq for Blob<T> where T: ?Sized {}

impl<T> PartialOrd for Blob<T>
where T: ?Sized
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.bytes.partial_cmp(&other.bytes)
    }
}

impl<T> Ord for Blob<T>
where T: ?Sized
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.bytes.cmp(&other.bytes)
    }
}

impl<T> Debug for Blob<T>
where T: ?Sized
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(BStr::new(&self.bytes), f)
    }
}

impl<T> Display for Blob<T>
where T: ?Sized
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(BStr::new(&self.bytes), f)
    }
}

impl<T> Serialize for Blob<T>
where T: ?Sized
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        serde_bytes::Bytes::new(&self.bytes.as_ref()).serialize(serializer)
    }
}

impl<'de, T> Deserialize<'de> for Blob<T>
where T: ?Sized
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        let bytes = serde_bytes::ByteBuf::deserialize(deserializer)?.into_vec();
        Ok(bytes.as_slice().to_blob().retype())
    }
}

impl<T> Default for Blob<T>
where T: ?Sized
{
    fn default() -> Self {
        Self {
            bytes: default(),
            t: default(),
        }
    }
}

impl<T> Clone for Blob<T>
where T: ?Sized
{
    fn clone(&self) -> Self {
        Self {
            bytes: self.bytes.clone(),
            t: default(),
        }
    }
}

impl<T> AsRef<[u8]> for Blob<T>
where T: ?Sized
{
    fn as_ref(&self) -> &[u8] {
        self.bytes.as_ref()
    }
}

impl<T> AsMut<[u8]> for Blob<T>
where T: ?Sized
{
    fn as_mut(&mut self) -> &mut [u8] {
        self.bytes.as_mut()
    }
}

impl<T> Deref for Blob<T>
where T: ?Sized
{
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        self.as_ref()
    }
}

impl<T> DerefMut for Blob<T>
where T: ?Sized
{
    fn deref_mut(&mut self) -> &mut [u8] {
        self.as_mut()
    }
}
