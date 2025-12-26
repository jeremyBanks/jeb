use serde::de::DeserializeOwned;

use crate::Value;

pub fn from_value<T>(value: Value) -> Result<T, Value>
where
    T: DeserializeOwned,
{
    Err(value)
}
