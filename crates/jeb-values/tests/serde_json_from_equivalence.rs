/// Tests to verify that the direct From<serde_json::Value> implementations
/// produce identical results to using serde's serialize/deserialize.
///
/// This verifies the claim in serde_json/mod.rs that the direct conversions
/// are equivalent to using the serialize trait, just with less overhead.
use jeb_values::{
    Float,
    Value,
};

/// Helper to convert a jeb Value to serde_json::Value via serialization
fn to_serde_json_via_serde(value: &Value) -> serde_json::Value {
    serde_json::to_value(value).expect("serialization should succeed")
}

/// Helper to convert a serde_json::Value to jeb Value via deserialization
fn from_serde_json_via_serde(json: &serde_json::Value) -> Value {
    serde_json::from_value(json.clone()).expect("deserialization should succeed")
}

// ============================================================================
// Direct From<serde_json::Value> tests
// ============================================================================

#[test]
fn test_from_json_null() {
    let json = serde_json::Value::Null;

    // Direct conversion
    let direct: Value = json.clone().into();

    // Via serde
    let via_serde = from_serde_json_via_serde(&json);

    assert_eq!(direct, via_serde);
    assert_eq!(direct, Value::Null);
}

#[test]
fn test_from_json_bool() {
    for &bool_val in &[true, false] {
        let json = serde_json::Value::Bool(bool_val);

        // Direct conversion
        let direct: Value = json.clone().into();

        // Via serde
        let via_serde = from_serde_json_via_serde(&json);

        assert_eq!(direct, via_serde);
        assert_eq!(direct, Value::Bool(bool_val));
    }
}

#[test]
fn test_from_json_number_unsigned() {
    for &num in &[0u64, 1, 42, 100, u64::MAX] {
        let json = serde_json::json!(num);

        // Direct conversion
        let direct: Value = json.clone().into();

        // Via serde
        let via_serde = from_serde_json_via_serde(&json);

        assert_eq!(direct, via_serde, "Failed for u64: {}", num);
        assert_eq!(direct, Value::Unsigned(num));
    }
}

#[test]
fn test_from_json_number_signed() {
    for &num in &[-1i64, -42, -100, i64::MIN, i64::MAX] {
        let json = serde_json::json!(num);

        // Direct conversion
        let direct: Value = json.clone().into();

        // Via serde
        let via_serde = from_serde_json_via_serde(&json);

        assert_eq!(direct, via_serde, "Failed for i64: {}", num);

        if num < 0 {
            assert_eq!(direct, Value::Signed(num));
        } else {
            // Positive i64 might convert to Unsigned
            assert!(matches!(direct, Value::Unsigned(_) | Value::Signed(_)));
        }
    }
}

#[test]
fn test_from_json_number_float() {
    for &num in &[0.0, 1.5, -1.5, 42.42, -42.42, 3.14159, f64::MIN, f64::MAX] {
        let json = serde_json::json!(num);

        // Direct conversion
        let direct: Value = json.clone().into();

        // Via serde
        let via_serde = from_serde_json_via_serde(&json);

        assert_eq!(direct, via_serde, "Failed for f64: {}", num);

        match direct {
            Value::Float(f) => {
                assert_eq!(*f, num);
            }
            _ => panic!("Expected Float variant for {}", num),
        }
    }
}

#[test]
fn test_from_json_number_zero_sign() {
    // Test positive and negative zero
    let pos_zero_json = serde_json::json!(0.0);
    let neg_zero_json = serde_json::json!(-0.0);

    // Direct conversion
    let pos_direct: Value = pos_zero_json.clone().into();
    let neg_direct: Value = neg_zero_json.clone().into();

    // Via serde
    let pos_via_serde = from_serde_json_via_serde(&pos_zero_json);
    let neg_via_serde = from_serde_json_via_serde(&neg_zero_json);

    assert_eq!(pos_direct, pos_via_serde);
    assert_eq!(neg_direct, neg_via_serde);

    // Verify they're different (positive vs negative zero)
    assert_ne!(pos_direct, neg_direct);
}

#[test]
fn test_from_json_string() {
    for s in &[
        "",
        "hello",
        "world",
        "with spaces",
        "unicode: 你好",
        "emoji: 🚀",
    ] {
        let json = serde_json::Value::String(s.to_string());

        // Direct conversion
        let direct: Value = json.clone().into();

        // Via serde
        let via_serde = from_serde_json_via_serde(&json);

        assert_eq!(direct, via_serde, "Failed for string: {}", s);
        assert_eq!(direct, Value::from(*s));
    }
}

#[test]
fn test_from_json_array_empty() {
    let json = serde_json::json!([]);

    // Direct conversion
    let direct: Value = json.clone().into();

    // Via serde
    let via_serde = from_serde_json_via_serde(&json);

    assert_eq!(direct, via_serde);
    assert_eq!(direct, Value::Array(vec![]));
}

#[test]
fn test_from_json_array_primitives() {
    let json = serde_json::json!([null, true, false, 42, -42, 3.14, "hello"]);

    // Direct conversion
    let direct: Value = json.clone().into();

    // Via serde
    let via_serde = from_serde_json_via_serde(&json);

    assert_eq!(direct, via_serde);

    match direct {
        Value::Array(arr) => {
            assert_eq!(arr.len(), 7);
            assert_eq!(arr[0], Value::Null);
            assert_eq!(arr[1], Value::Bool(true));
            assert_eq!(arr[2], Value::Bool(false));
            assert_eq!(arr[3], Value::Unsigned(42));
            assert_eq!(arr[4], Value::Signed(-42));
            assert_eq!(arr[5], Value::Float(Float::try_from(3.14).unwrap()));
            assert_eq!(arr[6], Value::from("hello"));
        }
        _ => panic!("Expected Array variant"),
    }
}

#[test]
fn test_from_json_array_nested() {
    let json = serde_json::json!([[1, 2], [3, 4], [[5]]]);

    // Direct conversion
    let direct: Value = json.clone().into();

    // Via serde
    let via_serde = from_serde_json_via_serde(&json);

    assert_eq!(direct, via_serde);
}

#[test]
fn test_from_json_object_empty() {
    let json = serde_json::json!({});

    // Direct conversion
    let direct: Value = json.clone().into();

    // Via serde
    let via_serde = from_serde_json_via_serde(&json);

    assert_eq!(direct, via_serde);

    match direct {
        Value::TextMap(map) => {
            assert_eq!(map.len(), 0);
        }
        _ => panic!("Expected TextMap variant"),
    }
}

#[test]
fn test_from_json_object_simple() {
    let json = serde_json::json!({
        "null": null,
        "bool": true,
        "number": 42,
        "string": "hello"
    });

    // Direct conversion
    let direct: Value = json.clone().into();

    // Via serde
    let via_serde = from_serde_json_via_serde(&json);

    assert_eq!(direct, via_serde);

    match direct {
        Value::TextMap(map) => {
            use jeb_values::Text;
            assert_eq!(map.len(), 4);
            assert_eq!(map.get(&Text::from("null")), Some(&Value::Null));
            assert_eq!(map.get(&Text::from("bool")), Some(&Value::Bool(true)));
            assert_eq!(map.get(&Text::from("number")), Some(&Value::Unsigned(42)));
            assert_eq!(map.get(&Text::from("string")), Some(&Value::from("hello")));
        }
        _ => panic!("Expected TextMap variant"),
    }
}

#[test]
fn test_from_json_object_nested() {
    let json = serde_json::json!({
        "outer": {
            "inner": {
                "value": 42
            }
        }
    });

    // Direct conversion
    let direct: Value = json.clone().into();

    // Via serde
    let via_serde = from_serde_json_via_serde(&json);

    assert_eq!(direct, via_serde);
}

#[test]
fn test_from_json_complex_nested() {
    let json = serde_json::json!({
        "users": [
            {
                "name": "Alice",
                "age": 30,
                "active": true,
                "balance": 123.45
            },
            {
                "name": "Bob",
                "age": 25,
                "active": false,
                "balance": -10.5
            }
        ],
        "metadata": {
            "version": "1.0",
            "count": 2
        }
    });

    // Direct conversion
    let direct: Value = json.clone().into();

    // Via serde
    let via_serde = from_serde_json_via_serde(&json);

    assert_eq!(direct, via_serde);
}

// ============================================================================
// Round-trip tests: Value -> JSON -> Value
// ============================================================================

#[test]
fn test_roundtrip_primitives() {
    let values = vec![
        Value::Null,
        Value::Bool(true),
        Value::Bool(false),
        Value::Unsigned(0),
        Value::Unsigned(42),
        Value::Unsigned(u64::MAX),
        Value::Signed(0),
        Value::Signed(-42),
        Value::Signed(i64::MIN),
        Value::Signed(i64::MAX),
        Value::Float(Float::try_from(0.0).unwrap()),
        Value::Float(Float::try_from(-0.0).unwrap()),
        Value::Float(Float::try_from(3.14).unwrap()),
        Value::Float(Float::try_from(-3.14).unwrap()),
        Value::from(""),
        Value::from("hello"),
        Value::from("unicode: 你好"),
    ];

    for original in values {
        // Convert to JSON
        let json = to_serde_json_via_serde(&original);

        // Convert back via direct From
        let via_direct: Value = json.clone().into();

        // Convert back via serde
        let via_serde = from_serde_json_via_serde(&json);

        assert_eq!(via_direct, via_serde, "Mismatch for {:?}", original);

        // JSON normalizes positive Signed integers to Unsigned because JSON doesn't
        // distinguish signed/unsigned for non-negative integers
        match original {
            Value::Signed(n) if n >= 0 => {
                assert_eq!(via_direct, Value::Unsigned(n as u64),
                    "Signed({}) should normalize to Unsigned({})", n, n);
            }
            _ => {
                assert_eq!(via_direct, original, "Round-trip failed for {:?}", original);
            }
        }
    }
}

#[test]
fn test_roundtrip_arrays() {
    let values = vec![
        Value::Array(vec![]),
        Value::from([Value::from(1u64), Value::from(2u64), Value::from(3u64)]),
        Value::from([Value::Null, Value::Bool(true), Value::from("test")]),
        Value::from([
            Value::from([Value::from(1u64)]),
            Value::from([Value::from(2u64)]),
        ]),
    ];

    for original in values {
        // Convert to JSON
        let json = to_serde_json_via_serde(&original);

        // Convert back via direct From
        let via_direct: Value = json.clone().into();

        // Convert back via serde
        let via_serde = from_serde_json_via_serde(&json);

        assert_eq!(via_direct, via_serde, "Mismatch for {:?}", original);
        assert_eq!(via_direct, original, "Round-trip failed for {:?}", original);
    }
}

#[test]
fn test_roundtrip_objects() {
    use jeb_values::Text;

    let values = vec![
        Value::TextMap(Default::default()),
        [(Text::from("a"), Value::from(1u64))]
            .into_iter()
            .collect::<Value>(),
        [
            (Text::from("null"), Value::Null),
            (Text::from("bool"), Value::Bool(true)),
            (Text::from("number"), Value::from(42u64)),
            (Text::from("string"), Value::from("hello")),
        ]
        .into_iter()
        .collect::<Value>(),
    ];

    for original in values {
        // Convert to JSON
        let json = to_serde_json_via_serde(&original);

        // Convert back via direct From
        let via_direct: Value = json.clone().into();

        // Convert back via serde
        let via_serde = from_serde_json_via_serde(&json);

        assert_eq!(via_direct, via_serde, "Mismatch for {:?}", original);
        assert_eq!(via_direct, original, "Round-trip failed for {:?}", original);
    }
}

// ============================================================================
// Edge cases and special scenarios
// ============================================================================

#[test]
fn test_from_json_number_special_values() {
    // Test values at integer boundaries
    let test_cases = vec![(0u64, "zero"), (1u64, "one"), (u64::MAX, "u64::MAX")];

    for (num, desc) in test_cases {
        let json = serde_json::json!(num);
        let direct: Value = json.clone().into();
        let via_serde = from_serde_json_via_serde(&json);

        assert_eq!(direct, via_serde, "Failed for {}", desc);
    }
}

#[test]
fn test_from_json_deeply_nested() {
    // Create a deeply nested structure
    let mut json = serde_json::json!({"value": 42});
    for _ in 0..10 {
        json = serde_json::json!({"nested": json});
    }

    // Direct conversion
    let direct: Value = json.clone().into();

    // Via serde
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
        3.14,
        "string",
        [1, 2, 3],
        {"key": "value"}
    ]);

    // Direct conversion
    let direct: Value = json.clone().into();

    // Via serde
    let via_serde = from_serde_json_via_serde(&json);

    assert_eq!(direct, via_serde);
}
