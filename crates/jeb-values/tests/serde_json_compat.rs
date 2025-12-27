use jeb_values::{from_value, Value};
use serde::{Deserialize, Serialize};

/// Test that our deserializer can accept serde_json-style data

#[test]
fn test_option_compat() {
    // serde_json serializes None as null, Some(x) as bare x
    // We should accept both formats

    #[derive(Debug, PartialEq, Deserialize)]
    struct Container {
        value: Option<i32>,
    }

    // Our format: None -> null, Some(42) -> {"Some": 42}
    let our_none = Value::TextMap(
        [(
            jeb_values::Text::from("value".to_string()),
            Value::Null,
        )]
        .into_iter()
        .collect(),
    );
    let container: Container = from_value(our_none).unwrap();
    assert_eq!(container.value, None);

    // serde_json format: Some(42) -> bare 42 (we accept this too)
    let json_some = Value::TextMap(
        [(
            jeb_values::Text::from("value".to_string()),
            Value::Unsigned(42),
        )]
        .into_iter()
        .collect(),
    );
    let container: Container = from_value(json_some).unwrap();
    assert_eq!(container.value, Some(42));

    // Our format: Some(42) -> {"Some": 42}
    let our_some = Value::TextMap(
        [(
            jeb_values::Text::from("value".to_string()),
            Value::TextMap(
                [(
                    jeb_values::Text::from("Some".to_string()),
                    Value::Unsigned(42),
                )]
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
    // Should accept both signed and unsigned for compatible values

    // Unsigned as signed (positive)
    let value = Value::Unsigned(42);
    let as_i64: i64 = from_value(value).unwrap();
    assert_eq!(as_i64, 42);

    // Signed as unsigned (non-negative)
    let value = Value::Signed(42);
    let as_u64: u64 = from_value(value).unwrap();
    assert_eq!(as_u64, 42);

    // Negative signed should fail as unsigned
    let value = Value::Signed(-42);
    let result: Result<u64, _> = from_value(value);
    assert!(result.is_err());
}

#[test]
fn test_array_as_bytes() {
    // serde_json serializes bytes as arrays
    // We should accept this
    let arr = Value::Array(vec![
        Value::Unsigned(72),
        Value::Unsigned(101),
        Value::Unsigned(108),
        Value::Unsigned(108),
        Value::Unsigned(111),
    ]);

    let bytes: Vec<u8> = from_value(arr).unwrap();
    assert_eq!(bytes, b"Hello");
}

#[test]
fn test_empty_map_from_array() {
    use std::collections::HashMap;

    // Empty array should deserialize as empty map
    let arr = Value::Array(vec![]);
    let map: HashMap<String, i32> = from_value(arr).unwrap();
    assert!(map.is_empty());
}

#[test]
fn test_struct_from_array() {
    // Structs can be deserialized from arrays (positional)
    #[derive(Debug, PartialEq, Deserialize)]
    struct Point {
        x: i32,
        y: i32,
    }

    let arr = Value::Array(vec![Value::Signed(10), Value::Signed(20)]);
    let point: Point = from_value(arr).unwrap();
    assert_eq!(point, Point { x: 10, y: 20 });
}

#[test]
fn test_enum_as_text() {
    // Unit enums serialize as Text (variant name)
    #[derive(Debug, PartialEq, Deserialize)]
    enum Status {
        Active,
        Inactive,
        Pending,
    }

    let value = Value::Text(jeb_values::Text::from("Active".to_string()));
    let status: Status = from_value(value).unwrap();
    assert_eq!(status, Status::Active);

    let value = Value::Text(jeb_values::Text::from("Pending".to_string()));
    let status: Status = from_value(value).unwrap();
    assert_eq!(status, Status::Pending);
}
