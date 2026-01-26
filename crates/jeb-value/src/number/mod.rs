use {
    core::hash::Hash,
    derive_more::{
        AsRef,
        Deref,
        Display,
        Into,
    },
};

// [impl jeb-value.features.core.cfg]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize),
    serde(transparent)
)]
// [impl jeb-value.variant.common.clone]
// [impl jeb-value.variant.common.debug]
// [impl jeb-value.variant.common.deref]
// [impl jeb-value.variant.common.as-ref]
#[derive(AsRef, Clone, Copy, Debug, Default, Deref, Display)]
// [impl jeb-value.variant.common.transparent]
#[repr(transparent)]
// [impl jeb-value.variant.common.must-use]
#[must_use]
// [impl jeb-value.number.struct]
pub struct Number(pub(crate) f64);

// [impl jeb-value.number.constructor]
// [impl jeb-value.number.finite]
impl Number {
    /// Creates a new `Number` from an `f64` if it is finite.
    /// Returns `None` if the value is NaN or infinite.
    #[must_use]
    pub const fn new(value: f64) -> Option<Self> {
        if value.is_finite() {
            Some(Number(value))
        } else {
            None
        }
    }
}

// [impl jeb-value.variant.common.into-inner]
// [impl jeb-value.variant.common.to-inner]
// [impl jeb-value.variant.common.as-inner]
impl Number {
    /// Consumes the `Number` and returns the inner `f64`.
    #[must_use]
    pub fn into_inner(self) -> f64 {
        self.0
    }

    /// Returns a copy of the inner `f64`.
    #[must_use]
    #[expect(clippy::wrong_self_convention, reason = "consistent API across all types")]
    pub fn to_inner(&self) -> f64 {
        self.0
    }

    /// Returns a reference to the inner `f64`.
    #[must_use]
    pub fn as_inner(&self) -> &f64 {
        &self.0
    }
}

// [impl jeb-value.variant.common.into-named-inner]
// [impl jeb-value.variant.common.to-named-inner]
// [impl jeb-value.variant.common.as-named-inner]
impl Number {
    /// Consumes the `Number` and returns the inner `f64`.
    #[must_use]
    pub fn into_f64(self) -> f64 {
        self.0
    }

    /// Returns a copy of the inner `f64`.
    #[must_use]
    #[expect(clippy::wrong_self_convention, reason = "consistent API across all types")]
    pub fn to_f64(&self) -> f64 {
        self.0
    }

    /// Returns a reference to the inner `f64`.
    #[must_use]
    pub fn as_f64(&self) -> &f64 {
        &self.0
    }
}

// [impl jeb-value.variant.common.inner-from]
impl From<Number> for f64 {
    fn from(value: Number) -> Self {
        value.0
    }
}

// [impl jeb-value.number.try-from-inner]
impl TryFrom<f64> for Number {
    type Error = NotFiniteError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Number::new(value).ok_or(NotFiniteError)
    }
}

/// Error returned when trying to create a `Number` from a non-finite `f64`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotFiniteError;

impl core::fmt::Display for NotFiniteError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "value is not finite (NaN or infinite)")
    }
}

impl std::error::Error for NotFiniteError {}

// [impl jeb-value.features.core.cfg]
#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Number {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = f64::deserialize(deserializer)?;
        Number::new(value).ok_or_else(|| {
            serde::de::Error::invalid_value(
                serde::de::Unexpected::Float(value),
                &"a finite floating point number",
            )
        })
    }
}

// [impl jeb-value.variant.common.ord]
// [impl jeb-value.number.cmp-no-delegate]
// [impl jeb-value.number.ord-total-cmp]
impl Ord for Number {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.0.total_cmp(&other.0)
    }
}

// [impl jeb-value.variant.common.partial-eq]
// [impl jeb-value.number.cmp-no-delegate]
// [impl jeb-value.number.partial-eq-total-cmp]
impl PartialEq for Number {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == core::cmp::Ordering::Equal
    }
}

// [impl jeb-value.variant.common.eq]
// [impl jeb-value.number.cmp-no-delegate]
// [impl jeb-value.number.eq-total-cmp]
impl Eq for Number {}

// [impl jeb-value.variant.common.partial-ord]
// [impl jeb-value.number.cmp-no-delegate]
// [impl jeb-value.number.partial-ord-total-cmp]
impl PartialOrd for Number {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

// [impl jeb-value.variant.common.hash]
// [impl jeb-value.number.cmp-no-delegate]
// [impl jeb-value.number.hash-to-be-bytes]
impl Hash for Number {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        state.write(&self.0.to_be_bytes());
    }
}
