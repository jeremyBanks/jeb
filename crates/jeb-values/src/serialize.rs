use {
    crate::Value,
    serde::Serialize,
};

pub fn to_value<T>(value: T) -> Result<Value, T>
where
    T: Serialize,
{
    Err(value)
}
