use {
    core::hash::Hash,
    derive_more::{AsRef, Deref, Display, Into},
    serde::{Deserialize, Serialize},
};



#[derive(AsRef, Clone, Debug, Default, Deref, Copy, Display, Into, Serialize)]
#[serde(transparent)]
#[repr(transparent)]
#[must_use]
pub struct Float(pub(in crate::model) f64);

impl Float {
    #[must_use]
    pub const fn new(value: f64) -> Option<Self> {
        if value.is_finite() {
            Some(Float(value))
        } else {
            None
        }
    }
}

impl<'de> Deserialize<'de> for Float {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = f64::deserialize(deserializer)?;
        Float::new(value).ok_or_else(|| {
            serde::de::Error::invalid_value(
                serde::de::Unexpected::Float(value),
                &"a finite floating point number",
            )
        })
    }
}

impl Ord for Float {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl PartialEq for Float {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == core::cmp::Ordering::Equal
    }
}

impl Eq for Float {}

impl PartialOrd for Float {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for Float {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(self.0.to_bits());
    }
}

impl TryFrom<f64> for Float {
    type Error = f64;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Float::new(value).ok_or(value)
    }
}

impl TryFrom<f32> for Float {
    type Error = f32;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        Float::new(value.into()).ok_or(value)
    }
}

impl From<i32> for Float {
    fn from(value: i32) -> Self {
        Float(value.into())
    }
}

impl From<u32> for Float {
    fn from(value: u32) -> Self {
        Float(value.into())
    }
}

impl From<i16> for Float {
    fn from(value: i16) -> Self {
        Float(value.into())
    }
}

impl From<u16> for Float {
    fn from(value: u16) -> Self {
        Float(value.into())
    }
}

impl From<i8> for Float {
    fn from(value: i8) -> Self {
        Float(value.into())
    }
}

impl From<u8> for Float {
    fn from(value: u8) -> Self {
        Float(value.into())
    }
}
