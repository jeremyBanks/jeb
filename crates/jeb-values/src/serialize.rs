use serde::Serialize;

use crate::Value;

pub fn to_value<T>(value: T) -> Result<Value, T>
where
    T: Serialize,
{
    Err(value)
}
