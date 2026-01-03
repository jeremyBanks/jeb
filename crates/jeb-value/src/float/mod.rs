use {
    core::hash::Hash,
    derive_more::{
        AsRef,
        Deref,
        Display,
        Into,
    },
};
// [impl jeb-value.dependencies.cfg]
#[cfg_attr(
    feature = "wasm",
    wasm_bindgen::prelude::wasm_bindgen
)]
// [impl jeb-value.dependencies.cfg]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize),
    serde(transparent)
)]
// [impl jeb-value.variants.clone]
// [impl jeb-value.variants.debug]
// [impl jeb-value.variants.deref]
// [impl jeb-value.variants.as-ref]
#[derive(AsRef, Clone, Copy, Debug, Default, Deref, Display, Into)]
// [impl jeb-value.variants.transparent]
#[repr(transparent)]
// [impl jeb-value.variant.must-use]
#[must_use]
// [impl jeb-value.number]
pub struct Float(pub(crate) f64);
impl Float {
    #[must_use]
    // [impl jeb-value.number.finite]
    pub const fn new(value: f64) -> Option<Self> {
        if value.is_finite() {
            Some(Float(value))
        } else {
            None
        }
    }
}
// [impl jeb-value.dependencies.cfg]
#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Float {
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
// [impl jeb-value.variants.cmp]
// [impl jeb-value.number.cmp]
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
