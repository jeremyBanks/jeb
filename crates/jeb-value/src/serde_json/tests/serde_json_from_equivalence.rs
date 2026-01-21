/// Tests to verify that the direct From<serde_json::Value> implementations
/// produce identical results to using serde's serialize/deserialize.
///
/// This verifies the claim in serde_json/mod.rs that the direct conversions
/// are equivalent to using the serialize trait, just with less overhead.
use crate::{Null, Number, Value};
/// Helper to convert a jeb Value to serde_json::Value via serialization
fn to_serde_json_via_serde(value: &Value) -> serde_json::Value {
    serde_json::to_value(value).expect("serialization should succeed")
}
/// Helper to convert a serde_json::Value to jeb Value via deserialization
fn from_serde_json_via_serde(json: &serde_json::Value) -> Value {
    serde_json::from_value(json.clone()).expect("deserialization should succeed")
}
#[test]
fn test_from_json_null() {
    let json = serde_json::Value::Null(Null::new());
    let direct: Value = json.clone().into();
    let via_serde = from_serde_json_via_serde(&json);
    assert_eq!(direct, via_serde);
    assert_eq!(direct, Value::Null(Null::new()));
}
#[test]
fn test_from_json_bool() {
    for &bool_val in &[true, false] {
        let json = serde_json::Value::Bool(bool_val);
        let direct: Value = json.clone().into();
        let via_serde = from_serde_json_via_serde(&json);
        assert_eq!(direct, via_serde);
        assert_eq!(direct, Value::Boolean(bool_val.into()));
    }
}
#[test]
fn test_from_json_number_unsigned() {
    for &num in &[0u64, 1, 42, 100] {
        let json = serde_json::json!(num);
        let direct: Value = json.clone().into();
        let via_serde = from_serde_json_via_serde(&json);
        assert_eq!(direct, via_serde, "Failed for u64: {}", num);
        assert_eq!(direct, Value::from(num));
    }
}
#[test]
fn test_from_json_number_signed() {
    for &num in &[-1i64, -42, -100] {
        let json = serde_json::json!(num);
        let direct: Value = json.clone().into();
        let via_serde = from_serde_json_via_serde(&json);
        assert_eq!(direct, via_serde, "Failed for i64: {}", num);
        assert_eq!(direct, Value::from(num));
    }
}
#[test]
fn test_from_json_number_float() {
    for &num in &[
        0.0,
        1.5,
        -1.5,
        42.42,
        -42.42,
        core::f64::consts::PI,
        f64::MIN,
        f64::MAX,
    ] {
        let json = serde_json::json!(num);
        let direct: Value = json.clone().into();
        let via_serde = from_serde_json_via_serde(&json);
        assert_eq!(direct, via_serde, "Failed for f64: {}", num);
        match direct {
            Value::Number(n) => {
                assert_eq!(*n, num);
            }
            _ => panic!("Expected Number variant for {}", num),
        }
    }
}
#[test]
fn test_from_json_number_zero_sign() {
    let pos_zero_json = serde_json::json!(0.0);
    let neg_zero_json = serde_json::json!(-0.0);
    let pos_direct: Value = pos_zero_json.clone().into();
    let neg_direct: Value = neg_zero_json.clone().into();
    let pos_via_serde = from_serde_json_via_serde(&pos_zero_json);
    let neg_via_serde = from_serde_json_via_serde(&neg_zero_json);
    assert_eq!(pos_direct, pos_via_serde);
    assert_eq!(neg_direct, neg_via_serde);
    assert_ne!(pos_direct, neg_direct);
}
#[test]
fn test_from_json_string() {
    for s in &["", "hello", "world", "with spaces", "unicode: 你好", "emoji: 🚀"] {
        let json = serde_json::Value::String(s.to_string());
        let direct: Value = json.clone().into();
        let via_serde = from_serde_json_via_serde(&json);
        assert_eq!(direct, via_serde, "Failed for string: {}", s);
        assert_eq!(direct, Value::from(*s));
    }
}
#[test]
fn test_from_json_array_empty() {
    let json = serde_json::json!([]);
    let direct: Value = json.clone().into();
    let via_serde = from_serde_json_via_serde(&json);
    assert_eq!(direct, via_serde);
    assert_eq!(direct, Value::Array(vec![]));
}
#[test]
fn test_from_json_array_primitives() {
    let json = serde_json::json!([null, true, false, 42, -42, core::f64::consts::PI, "hello"]);
    let direct: Value = json.clone().into();
    let via_serde = from_serde_json_via_serde(&json);
    assert_eq!(direct, via_serde);
    match direct {
        Value::Array(arr) => {
            assert_eq!(arr.len(), 7);
            assert_eq!(arr[0], Value::Null(Null::new()));
            assert_eq!(arr[1], Value::Boolean(true.into()));
            assert_eq!(arr[2], Value::Boolean(false.into()));
            assert_eq!(arr[3], Value::from(42u64));
            assert_eq!(arr[4], Value::from(-42i64));
            assert_eq!(
                arr[5],
                Value::Number(Number::new(core::f64::consts::PI).unwrap())
            );
            assert_eq!(arr[6], Value::from("hello"));
        }
        _ => panic!("Expected Array variant"),
    }
}
#[test]
fn test_from_json_array_nested() {
    let json = serde_json::json!([[1, 2], [3, 4], [[5]]]);
    let direct: Value = json.clone().into();
    let via_serde = from_serde_json_via_serde(&json);
    assert_eq!(direct, via_serde);
}
#[test]
fn test_from_json_object_empty() {
    let json = serde_json::json!({});
    let direct: Value = json.clone().into();
    let via_serde = from_serde_json_via_serde(&json);
    assert_eq!(direct, via_serde);
    match direct {
        Value::StringMap(map) => {
            assert_eq!(map.len(), 0);
        }
        _ => panic!("Expected StringMap variant"),
    }
}
#[test]
fn test_from_json_object_simple() {
    let json = serde_json::json!({"null": null, "bool": true, "number": 42, "string": "hello"});
    let direct: Value = json.clone().into();
    let via_serde = from_serde_json_via_serde(&json);
    assert_eq!(direct, via_serde);
    match direct {
        Value::StringMap(map) => {
            use crate::String;
            assert_eq!(map.len(), 4);
            assert_eq!(map.get(&String::from("null")), Some(&Value::Null(Null::new())));
            assert_eq!(
                map.get(&String::from("bool")),
                Some(&Value::Boolean(true.into()))
            );
            assert_eq!(map.get(&String::from("number")), Some(&Value::from(42u64)));
            assert_eq!(
                map.get(&String::from("string")),
                Some(&Value::from("hello"))
            );
        }
        _ => panic!("Expected StringMap variant"),
    }
}
#[test]
fn test_from_json_object_nested() {
    let json = serde_json::json!({"outer": {"inner": {"value": 42}}});
    let direct: Value = json.clone().into();
    let via_serde = from_serde_json_via_serde(&json);
    assert_eq!(direct, via_serde);
}
#[test]
fn test_from_json_complex_nested() {
    let json = serde_json::json!({
        "users": [
            {"name": "Alice", "age": 30, "active": true, "balance": 123.45},
            {"name": "Bob", "age": 25, "active": false, "balance": -10.5}
        ],
        "metadata": {"version": "1.0", "count": 2}
    });
    let direct: Value = json.clone().into();
    let via_serde = from_serde_json_via_serde(&json);
    assert_eq!(direct, via_serde);
}
#[test]
fn test_roundtrip_primitives() {
    let values = vec![
        Value::Null(Null::new()),
        Value::Boolean(true.into()),
        Value::Boolean(false.into()),
        Value::from(0u64),
        Value::from(42u64),
        Value::from(-42i64),
        Value::Number(Number::new(0.0).unwrap()),
        Value::Number(Number::new(-0.0).unwrap()),
        Value::Number(Number::new(core::f64::consts::PI).unwrap()),
        Value::Number(Number::new(-core::f64::consts::PI).unwrap()),
        Value::from(""),
        Value::from("hello"),
        Value::from("unicode: 你好"),
    ];
    for original in values {
        let json = to_serde_json_via_serde(&original);
        let via_direct: Value = json.clone().into();
        let via_serde = from_serde_json_via_serde(&json);
        assert_eq!(via_direct, via_serde, "Mismatch for {:?}", original);
        assert_eq!(via_direct, original, "Round-trip failed for {:?}", original);
    }
}
#[test]
fn test_roundtrip_arrays() {
    let values = vec![
        Value::Array(vec![]),
        Value::from([Value::from(1u64), Value::from(2u64), Value::from(3u64)]),
        Value::from([Value::Null(Null::new()), Value::Boolean(true.into()), Value::from("test")]),
        Value::from([
            Value::from([Value::from(1u64)]),
            Value::from([Value::from(2u64)]),
        ]),
    ];
    for original in values {
        let json = to_serde_json_via_serde(&original);
        let via_direct: Value = json.clone().into();
        let via_serde = from_serde_json_via_serde(&json);
        assert_eq!(via_direct, via_serde, "Mismatch for {:?}", original);
        assert_eq!(via_direct, original, "Round-trip failed for {:?}", original);
    }
}
#[test]
fn test_roundtrip_objects() {
    use crate::String;
    let values = vec![
        Value::StringMap(Default::default()),
        [(String::from("a"), Value::from(1u64))]
            .into_iter()
            .collect::<Value>(),
        [
            (String::from("null"), Value::Null(Null::new())),
            (String::from("bool"), Value::Boolean(true.into())),
            (String::from("number"), Value::from(42u64)),
            (String::from("string"), Value::from("hello")),
        ]
        .into_iter()
        .collect::<Value>(),
    ];
    for original in values {
        let json = to_serde_json_via_serde(&original);
        let via_direct: Value = json.clone().into();
        let via_serde = from_serde_json_via_serde(&json);
        assert_eq!(via_direct, via_serde, "Mismatch for {:?}", original);
        assert_eq!(via_direct, original, "Round-trip failed for {:?}", original);
    }
}
#[test]
fn test_from_json_number_special_values() {
    let test_cases = vec![(0u64, "zero"), (1u64, "one")];
    for (num, desc) in test_cases {
        let json = serde_json::json!(num);
        let direct: Value = json.clone().into();
        let via_serde = from_serde_json_via_serde(&json);
        assert_eq!(direct, via_serde, "Failed for {}", desc);
    }
}
#[test]
fn test_from_json_deeply_nested() {
    let mut json = serde_json::json!({"value": 42});
    for _ in 0..10 {
        json = serde_json::json!({"nested": json});
    }
    let direct: Value = json.clone().into();
    let via_serde = from_serde_json_via_serde(&json);
    assert_eq!(direct, via_serde);
}
#[test]
fn test_from_json_mixed_array_types() {
    let json = serde_json::json!([
        null,
        true,
        42,
        -42,
        core::f64::consts::PI,
        "string",
        [1, 2, 3],
        {"key": "value"}
    ]);
    let direct: Value = json.clone().into();
    let via_serde = from_serde_json_via_serde(&json);
    assert_eq!(direct, via_serde);
}
