use {
    super::bytes::Bytes,
    derive_more::{
        AsMut,
        AsRef,
        Deref,
        DerefMut,
        Display,
        From,
        Index,
        IndexMut,
        Into,
    },
};

mod iterator;

// [impl jeb-value.features.core.cfg]
#[cfg_attr(
    feature = "serde",
    derive(
        serde::Serialize,
        serde::Deserialize
    ),
    serde(transparent)
)]
// [impl jeb-value.variant.common.clone]
// [impl jeb-value.variant.common.debug]
// [impl jeb-value.variant.common.deref]
// [impl jeb-value.variant.common.as-ref]
// [impl jeb-value.variant.common.eq-delegate-inner]
// [impl jeb-value.variant.common.partial-eq-delegate-inner]
// [impl jeb-value.variant.common.ord-delegate-inner]
// [impl jeb-value.variant.common.partial-ord-delegate-inner]
// [impl jeb-value.variant.common.hash-delegate-inner]
// [impl jeb-value.variant.common.borrow]
// [impl jeb-value.variant.common.mut]
#[derive(
    AsMut,
    AsRef,
    Clone,
    Debug,
    Default,
    Deref,
    DerefMut,
    Display,
    Eq,
    From,
    Hash,
    Index,
    IndexMut,
    Ord,
    PartialEq,
    PartialOrd,
)]
// [impl jeb-value.variant.common.transparent]
#[repr(transparent)]
#[as_ref(std::string::String, str, [u8])]
// [impl jeb-value.variant.common.must-use]
#[must_use]
// [impl jeb-value.string.struct]
pub struct String(pub(crate) std::string::String);

// [impl jeb-value.variant.common.constructor]
// [impl jeb-value.variant.common.try-from-inner]
// [impl jeb-value.string.from-inner]
impl String {
    /// Creates a new `String` from a `std::string::String`.
    #[must_use]
    pub fn new(value: std::string::String) -> Self {
        String(value)
    }
}

// [impl jeb-value.variant.common.into-inner]
// [impl jeb-value.variant.common.to-inner]
// [impl jeb-value.variant.common.as-inner]
impl String {
    /// Consumes the `String` and returns the inner `std::string::String`.
    #[must_use]
    pub fn into_inner(self) -> std::string::String {
        self.0
    }

    /// Returns a clone of the inner `std::string::String`.
    #[must_use]
    pub fn to_inner(&self) -> std::string::String {
        self.0.clone()
    }

    /// Returns a reference to the inner `std::string::String`.
    #[must_use]
    pub fn as_inner(&self) -> &std::string::String {
        &self.0
    }
}

// [impl jeb-value.variant.common.into-named-inner]
// [impl jeb-value.variant.common.to-named-inner]
// [impl jeb-value.variant.common.as-named-inner]
impl String {
    /// Consumes the `String` and returns the inner `std::string::String`.
    #[must_use]
    pub fn into_string(self) -> std::string::String {
        self.0
    }

    /// Returns a clone of the inner `std::string::String`.
    #[must_use]
    pub fn to_string_inner(&self) -> std::string::String {
        self.0.clone()
    }

    /// Returns a reference to the inner `std::string::String`.
    #[must_use]
    pub fn as_string(&self) -> &std::string::String {
        &self.0
    }
}

// [impl jeb-value.variant.common.inner-from]
impl From<String> for std::string::String {
    fn from(value: String) -> Self {
        value.0
    }
}

// [impl jeb-value.string.from-str]
impl From<&str> for String {
    fn from(s: &str) -> Self {
        String(s.to_string())
    }
}

// [impl jeb-value.variant.common.try-from-other-via-inner]
impl TryFrom<Bytes> for String {
    type Error = core::str::Utf8Error;

    fn try_from(value: Bytes) -> Result<Self, Self::Error> {
        let s = core::str::from_utf8(&value)?;
        Ok(String(s.to_string()))
    }
}
