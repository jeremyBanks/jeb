use {
    jeb_value::{Value, from_value},
    serde::Deserialize,
};
/// Test that our deserializer can accept serde_json-style data
#[test]
fn test_option_compat() {
    #[derive(Debug, PartialEq, Deserialize)]
    struct Container {
        value: Option<i32>,
    }
    let our_none = Value::TextMap(
        [(jeb_value::Text::from("value".to_string()), Value::Null)].into_iter().collect(),
    );
    let container: Container = from_value(our_none).unwrap();
    assert_eq!(container.value, None);
    let json_some = Value::TextMap(
        [(jeb_value::Text::from("value".to_string()), Value::Unsigned(42))]
            .into_iter()
            .collect(),
    );
    let container: Container = from_value(json_some).unwrap();
    assert_eq!(container.value, Some(42));
    let our_some = Value::TextMap(
        [
            (
                jeb_value::Text::from("value".to_string()),
                Value::TextMap(
                    [(jeb_value::Text::from("Some".to_string()), Value::Unsigned(42))]
                        .into_iter()
                        .collect(),
                ),
            ),
        ]
            .into_iter()
            .collect(),
    );
    let container: Container = from_value(our_some).unwrap();
    assert_eq!(container.value, Some(42));
}
#[test]
fn test_integer_cross_conversion() {
    let value = Value::Unsigned(42);
    let as_i64: i64 = from_value(value).unwrap();
    assert_eq!(as_i64, 42);
    let value = Value::Signed(42);
    let as_u64: u64 = from_value(value).unwrap();
    assert_eq!(as_u64, 42);
    let value = Value::Signed(-42);
    let result: Result<u64, _> = from_value(value);
    assert!(result.is_err());
}
#[test]
fn test_array_as_bytes() {
    let arr = Value::Array(
        vec![
            Value::Unsigned(72), Value::Unsigned(101), Value::Unsigned(108),
            Value::Unsigned(108), Value::Unsigned(111),
        ],
    );
    let bytes: Vec<u8> = from_value(arr).unwrap();
    assert_eq!(bytes, b"Hello");
}
#[test]
fn test_empty_map_from_array() {
    use std::collections::HashMap;
    let arr = Value::Array(vec![]);
    let map: HashMap<String, i32> = from_value(arr).unwrap();
    assert!(map.is_empty());
}
#[test]
fn test_struct_from_array() {
    #[derive(Debug, PartialEq, Deserialize)]
    struct Point {
        x: i32,
        y: i32,
    }
    let arr = Value::Array(vec![Value::Signed(10), Value::Signed(20)]);
    let point: Point = from_value(arr).unwrap();
    assert_eq!(point, Point { x : 10, y : 20 });
}
#[test]
fn test_enum_as_text() {
    #[derive(Debug, PartialEq, Deserialize)]
    enum Status {
        Active,
        Inactive,
        Pending,
    }
    let value = Value::Text(jeb_value::Text::from("Active".to_string()));
    let status: Status = from_value(value).unwrap();
    assert_eq!(status, Status::Active);
    let value = Value::Text(jeb_value::Text::from("Pending".to_string()));
    let status: Status = from_value(value).unwrap();
    assert_eq!(status, Status::Pending);
}
