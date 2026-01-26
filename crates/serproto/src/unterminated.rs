use std::fmt;

use derive_more::AsRef;
use derive_more::Deref;
use derive_more::From;
use derive_more::Into;
use serde::de::Visitor;
use serde::ser::SerializeTuple;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde_bytes::ByteBuf;
use serde_bytes::Bytes;
use serde_bytes::Deserialize as DeserializeBytes;
use serde_bytes::Serialize as SerializeBytes;

// How to create a de-facto custom type within the serde data model?

// Use a guid for your struct name?
// Only change it when you want to break compatibility.
// Use truncated blake3 of... something? A plain text
// description of the desired behaviour?

#[derive(Serialize, Deserialize, Deref, AsRef, From)]
#[repr(transparent)]
#[serde(transparent)]
#[allow(non_camel_case_types, unused)]
#[doc(hidden)]
struct D41D8CD98F00B204E9800998ECF8427E<T>(T);
type AsUnterminatedBytes<T> = D41D8CD98F00B204E9800998ECF8427E<T>;
#[allow(non_upper_case_globals)]
static AsUnterminatedBytes: &str = "D41D8CD98F00B204E9800998ECF8427E";

/// Give these an internal type with a hyper-specific long name, which our
/// serializer can look out for specifically. If you want to ensure actual
/// binding, you could use a proc macro to generate a random struct name at
/// build, and expose that for use in this file via a macro, something like:
pub static UNTERMINATED_BYTES_IDENT: &str = "UnterminatedByteBuf_123456";
#[macro_export]
macro_rules! pub_struct_unterminated_bytes {
    (
      $(#![$meta:meta])*
      $($ident:ident $($tt:tt)*)?
    ) => {
        $(#[$meta])*
        pub struct UnterminatedByteBuf_123456 {
          $($ident $($tt)*)?
        }
    };
}
// I'm not sure how else we could verify that you're actually referring to the
// specific type we want, and not potentially a different version of the same
// crate, or another copy needed to link with something else. If we even care
// about that. And there should probably be some way to disable it, as with
// CONST_RANDOM_SEED.

pub_struct_unterminated_bytes! {
    #![derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, AsRef, From, Into, Deref)]
    bytes: ByteBuf,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, AsRef, From, Into, Deref)]
pub struct UnterminatedBytes<'bytes> {
    bytes: &'bytes Bytes,
}
