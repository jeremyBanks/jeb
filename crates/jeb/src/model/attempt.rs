use {
    getset::{Getters, Setters},
    serde::{Deserialize, Serialize},
};

#[derive(Debug, Clone, Serialize, Deserialize, Getters, Setters)]
pub struct Attempt<T, W> {
    value: T,
    warnings: Vec<W>,
}
