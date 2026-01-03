use {
    derive_more::{
        Deref,
        DerefMut,
    },
    getset::{
        Getters,
        Setters,
    },
    serde::{
        Deserialize,
        Serialize,
    },
};
#[derive(Debug, Clone, Default, Serialize, Deserialize, Getters, Setters, Deref, DerefMut)]
pub struct Attempt<T, W> {
    #[deref]
    #[deref_mut]
    value: T,
    warnings: Vec<W>,
}
impl<T, W> From<Attempt<T, W>> for Result<T, Vec<W>> {
    fn from(attempt: Attempt<T, W>) -> Self {
        if attempt.warnings.is_empty() {
            Ok(attempt.value)
        } else {
            Err(attempt.warnings)
        }
    }
}
impl<T, W> From<T> for Attempt<T, W> {
    fn from(value: T) -> Self {
        Self {
            value,
            warnings: Vec::default(),
        }
    }
}
