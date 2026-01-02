impl From<serde_json::Value> for crate::Value {
    fn from(value: serde_json::Value) -> Self {
        use serde_json::Value::*;
        match value {
            Null => crate::Value::Null,
            Bool(x) => x.into(),
            Number(x) => x.into(),
            String(x) => x.into(),
            Array(x) => x.into(),
            Object(x) => x.into(),
        }
    }
}
impl From<Vec<serde_json::Value>> for crate::Value {
    fn from(value: Vec<serde_json::Value>) -> Self {
        crate::Value::Array(value.into_iter().map(crate::Value::from).collect())
    }
}
impl From<serde_json::Map<String, serde_json::Value>> for crate::Value {
    fn from(value: serde_json::Map<String, serde_json::Value>) -> Self {
        crate::Value::TextMap(
            value
                .into_iter()
                .map(|(k, v)| (k.into(), crate::Value::from(v)))
                .collect(),
        )
    }
}
impl From<serde_json::Number> for crate::Value {
    fn from(value: serde_json::Number) -> Self {
        if let Some(value) = value.as_u64() {
            value.into()
        } else if let Some(value) = value.as_i64() {
            value.into()
        } else if let Some(value) = value.as_f64()
            && let Some(value) = crate::number::Number::new(value)
        {
            value.into()
        } else {
            value.to_string().into()
        }
    }
}
