use std::primitive;

pub mod blip;
pub mod blob;
pub mod blobbable;
pub mod serialization;

pub use self::blobbable::Blobbable;
pub use self::serialization::BlobSerialization;
pub use self::serialization::Cbor;
pub use self::serialization::FlexBuffers;
pub use self::serialization::Json;
pub use self::serialization::Postcard;
pub use self::serialization::Unknown;

/// [`Blob<T, Json>`][blob::Blob]
pub type JsonBlob<T> = blob::Blob<T, Json>;
/// [`Blob<T, Postcard>`][blob::Blob]
pub type PostcardBlob<T> = blob::Blob<T, Postcard>;
/// [`Blob<T, FlexBuffers>`][blob::Blob]
pub type FlexBuffersBlob<T> = blob::Blob<T, FlexBuffers>;
/// [`Blob<T, Cbor>`][blob::Blob]
pub type CborBlob<T> = blob::Blob<T, Cbor>;

/// [`Blip<T, Json>`][blip::Blip]
pub type JsonBlip<T> = blip::Blip<T, Json>;
/// [`Blip<T, Postcard>`][blip::Blip]
pub type PostcardBlip<T> = blip::Blip<T, Postcard>;
/// [`Blip<T, FlexBuffers>`][blip::Blip]
pub type FlexBuffersBlip<T> = blip::Blip<T, FlexBuffers>;
/// [`Blip<T, Cbor>`][blip::Blip]
pub type CborBlip<T> = blip::Blip<T, Cbor>;

/// [`Blip<T, Postcard>`][blip::Blip], using our default serialization format
pub type Blip<T> = PostcardBlip<T>;
/// [`Blob<T, Postcard>`][blob::Blob], using our default serialization format
pub type Blob<T> = PostcardBlob<T>;

/// [`Blip<Unknown, Unknown>`][blip::Blip], a [`Blip`] representing data of an
/// unknown type using an unknown serialization format.
pub type UnknownBlip = blip::Blip<Unknown, Unknown>;

/// [`Blob<Unknown, Unknown>`][blob::Blob], a [`Blob`] containing data of an
/// unknown type using an unknown serialization format.
pub type UnknownBlob = blob::Blob<Unknown, Unknown>;

/// [`[u8]`][slice], an externally-/un-sized byte string
#[allow(non_camel_case_types)]
pub type bytes = [u8];

/// [`str`][primitive::str], an externally-/un-sized UTF-8 text string
#[allow(non_camel_case_types)]
pub type string = primitive::str;
