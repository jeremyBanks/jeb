use {
    crate::{
        Null,
        Value,
        from_value,
    },
    serde::Deserialize,
};
/// Test that our deserializer can accept serde_json-style data
#[test]
fn test_option_compat() {
    #[derive(Debug, PartialEq, Deserialize)]
    struct Container {
        value: Option<i32>,
    }
    let our_none = Value::StringMap(
        [(crate::String::from("value"), Value::Null(Null::new()))]
            .into_iter()
            .collect(),
    );
    let container: Container = from_value(our_none).unwrap();
    assert_eq!(container.value, None);
    let json_some = Value::StringMap(
        [(crate::String::from("value"), Value::from(42u64))]
            .into_iter()
            .collect(),
    );
    let container: Container = from_value(json_some).unwrap();
    assert_eq!(container.value, Some(42));
    let our_some = Value::StringMap(
        [(
            crate::String::from("value"),
            Value::StringMap(
                [(crate::String::from("Some"), Value::from(42u64))]
                    .into_iter()
                    .collect(),
            ),
        )]
        .into_iter()
        .collect(),
    );
    let container: Container = from_value(our_some).unwrap();
    assert_eq!(container.value, Some(42));
}
#[test]
fn test_integer_cross_conversion() {
    let value = Value::from(42u64);
    let as_i64: i64 = from_value(value).unwrap();
    assert_eq!(as_i64, 42);
    let value = Value::from(42i64);
    let as_u64: u64 = from_value(value).unwrap();
    assert_eq!(as_u64, 42);
    let value = Value::from(-42i64);
    let result: Result<u64, _> = from_value(value);
    assert!(result.is_err());
}
#[test]
fn test_array_as_bytes() {
    let arr = Value::Array(
        vec![
            Value::from(72u64),
            Value::from(101u64),
            Value::from(108u64),
            Value::from(108u64),
            Value::from(111u64),
        ]
        .into(),
    );
    let bytes: Vec<u8> = from_value(arr).unwrap();
    assert_eq!(bytes, b"Hello");
}
#[test]
fn test_empty_map_from_array() {
    use std::collections::HashMap;
    let arr = Value::Array(vec![].into());
    let map: HashMap<std::string::String, i32> = from_value(arr).unwrap();
    assert!(map.is_empty());
}
#[test]
fn test_struct_from_array() {
    #[derive(Debug, PartialEq, Deserialize)]
    struct Point {
        x: i32,
        y: i32,
    }
    let arr = Value::Array(vec![Value::from(10i64), Value::from(20i64)].into());
    let point: Point = from_value(arr).unwrap();
    assert_eq!(point, Point { x: 10, y: 20 });
}
#[test]
fn test_enum_as_string() {
    #[derive(Debug, PartialEq, Deserialize)]
    enum Status {
        Active,
        Inactive,
        Pending,
    }
    let value = Value::String(crate::String::from("Active"));
    let status: Status = from_value(value).unwrap();
    assert_eq!(status, Status::Active);
    let value = Value::String(crate::String::from("Pending"));
    let status: Status = from_value(value).unwrap();
    assert_eq!(status, Status::Pending);
}
