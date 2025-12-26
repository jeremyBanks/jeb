use {
    crate::Value,
    serde::de::DeserializeOwned,
};

pub fn from_value<T>(value: Value) -> Result<T, Value>
where
    T: DeserializeOwned,
{
    Err(value)
}
