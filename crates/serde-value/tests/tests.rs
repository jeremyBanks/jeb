//! Tests for serde-value.

use serde::{Deserialize, Serialize};
use serde_value::{from_value, to_value, Value};

// ============================================================================
// Primitive Tests
// ============================================================================

#[test]
fn test_bool() {
    assert_eq!(to_value(&true).unwrap(), Value::Bool(true));
    assert_eq!(to_value(&false).unwrap(), Value::Bool(false));

    let v = Value::Bool(true);
    assert_eq!(from_value::<bool>(v).unwrap(), true);
}

#[test]
fn test_integers() {
    assert_eq!(to_value(&42i8).unwrap(), Value::I8(42));
    assert_eq!(to_value(&42i16).unwrap(), Value::I16(42));
    assert_eq!(to_value(&42i32).unwrap(), Value::I32(42));
    assert_eq!(to_value(&42i64).unwrap(), Value::I64(42));
    assert_eq!(to_value(&42i128).unwrap(), Value::I128(42));

    assert_eq!(to_value(&42u8).unwrap(), Value::U8(42));
    assert_eq!(to_value(&42u16).unwrap(), Value::U16(42));
    assert_eq!(to_value(&42u32).unwrap(), Value::U32(42));
    assert_eq!(to_value(&42u64).unwrap(), Value::U64(42));
    assert_eq!(to_value(&42u128).unwrap(), Value::U128(42));

    // Round-trip
    assert_eq!(from_value::<i32>(Value::I32(42)).unwrap(), 42);
    assert_eq!(from_value::<u64>(Value::U64(123)).unwrap(), 123);
}

#[test]
fn test_floats() {
    assert_eq!(to_value(&3.14f32).unwrap(), Value::F32(3.14));
    assert_eq!(to_value(&3.14f64).unwrap(), Value::F64(3.14));

    // Round-trip
    assert_eq!(from_value::<f32>(Value::F32(3.14)).unwrap(), 3.14);
    assert_eq!(from_value::<f64>(Value::F64(2.718)).unwrap(), 2.718);
}

#[test]
fn test_char() {
    assert_eq!(to_value(&'a').unwrap(), Value::Char('a'));
    assert_eq!(from_value::<char>(Value::Char('z')).unwrap(), 'z');
}

#[test]
fn test_string() {
    assert_eq!(
        to_value(&"hello".to_string()).unwrap(),
        Value::String("hello".to_string())
    );
    assert_eq!(to_value(&"world").unwrap(), Value::String("world".to_string()));

    assert_eq!(
        from_value::<String>(Value::String("test".to_string())).unwrap(),
        "test"
    );
}

#[test]
fn test_bytes() {
    // serde_bytes is needed for proper byte serialization
    // For now, test that bytes variant exists
    let v = Value::Bytes(vec![1, 2, 3]);
    assert_eq!(v, Value::Bytes(vec![1, 2, 3]));
}

// ============================================================================
// Option Tests
// ============================================================================

#[test]
fn test_option_none() {
    let none: Option<i32> = None;
    assert_eq!(to_value(&none).unwrap(), Value::None);

    assert_eq!(from_value::<Option<i32>>(Value::None).unwrap(), None);
}

#[test]
fn test_option_some() {
    let some: Option<i32> = Some(42);
    let value = to_value(&some).unwrap();
    assert_eq!(value, Value::Some(Box::new(Value::I32(42))));

    assert_eq!(
        from_value::<Option<i32>>(Value::Some(Box::new(Value::I32(42)))).unwrap(),
        Some(42)
    );
}

// ============================================================================
// Unit Tests
// ============================================================================

#[test]
fn test_unit() {
    assert_eq!(to_value(&()).unwrap(), Value::Unit);
    assert_eq!(from_value::<()>(Value::Unit).unwrap(), ());
}

#[test]
fn test_unit_struct() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Marker;

    let value = to_value(&Marker).unwrap();
    assert_eq!(value, Value::UnitStruct { name: "Marker" });

    // Deserializing works because UnitStruct deserializes as unit
    assert_eq!(from_value::<Marker>(value).unwrap(), Marker);
}

// ============================================================================
// Newtype Tests
// ============================================================================

#[test]
fn test_newtype_struct() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Meters(u32);

    let value = to_value(&Meters(100)).unwrap();
    match &value {
        Value::NewtypeStruct { name, value } => {
            assert_eq!(*name, "Meters");
            assert_eq!(**value, Value::U32(100));
        }
        _ => panic!("expected NewtypeStruct, got {value:?}"),
    }

    assert_eq!(from_value::<Meters>(value).unwrap(), Meters(100));
}

// ============================================================================
// Sequence Tests
// ============================================================================

#[test]
fn test_vec() {
    let vec = vec![1i32, 2, 3];
    let value = to_value(&vec).unwrap();
    assert_eq!(
        value,
        Value::Seq(vec![Value::I32(1), Value::I32(2), Value::I32(3)])
    );

    assert_eq!(from_value::<Vec<i32>>(value).unwrap(), vec![1, 2, 3]);
}

#[test]
fn test_tuple() {
    let tuple = (1i32, "hello", true);
    let value = to_value(&tuple).unwrap();
    assert_eq!(
        value,
        Value::Tuple(vec![
            Value::I32(1),
            Value::String("hello".to_string()),
            Value::Bool(true),
        ])
    );

    assert_eq!(
        from_value::<(i32, String, bool)>(value).unwrap(),
        (1, "hello".to_string(), true)
    );
}

#[test]
fn test_tuple_struct() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Point(i32, i32);

    let value = to_value(&Point(10, 20)).unwrap();
    match &value {
        Value::TupleStruct { name, fields } => {
            assert_eq!(*name, "Point");
            assert_eq!(fields, &vec![Value::I32(10), Value::I32(20)]);
        }
        _ => panic!("expected TupleStruct, got {value:?}"),
    }

    assert_eq!(from_value::<Point>(value).unwrap(), Point(10, 20));
}

// ============================================================================
// Map Tests
// ============================================================================

#[test]
fn test_map() {
    use std::collections::HashMap;

    let mut map = HashMap::new();
    map.insert("a".to_string(), 1i32);
    map.insert("b".to_string(), 2);

    let value = to_value(&map).unwrap();
    match &value {
        Value::Map(entries) => {
            assert_eq!(entries.len(), 2);
            // Check that both entries exist (order may vary)
            let has_a = entries
                .iter()
                .any(|(k, v)| *k == Value::String("a".to_string()) && *v == Value::I32(1));
            let has_b = entries
                .iter()
                .any(|(k, v)| *k == Value::String("b".to_string()) && *v == Value::I32(2));
            assert!(has_a && has_b);
        }
        _ => panic!("expected Map, got {value:?}"),
    }

    let result: HashMap<String, i32> = from_value(value).unwrap();
    assert_eq!(result.get("a"), Some(&1));
    assert_eq!(result.get("b"), Some(&2));
}

// ============================================================================
// Struct Tests
// ============================================================================

#[test]
fn test_struct() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct User {
        name: String,
        age: u32,
    }

    let user = User {
        name: "Alice".to_string(),
        age: 30,
    };
    let value = to_value(&user).unwrap();

    match &value {
        Value::Struct { name, fields } => {
            assert_eq!(*name, "User");
            assert_eq!(fields.len(), 2);
            assert_eq!(fields[0], ("name", Value::String("Alice".to_string())));
            assert_eq!(fields[1], ("age", Value::U32(30)));
        }
        _ => panic!("expected Struct, got {value:?}"),
    }

    assert_eq!(from_value::<User>(value).unwrap(), user);
}

#[test]
fn test_nested_struct() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Address {
        city: String,
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Person {
        name: String,
        address: Address,
    }

    let person = Person {
        name: "Bob".to_string(),
        address: Address {
            city: "NYC".to_string(),
        },
    };

    let value = to_value(&person).unwrap();
    match &value {
        Value::Struct { name, fields } => {
            assert_eq!(*name, "Person");
            assert_eq!(fields.len(), 2);

            // Check nested struct
            match &fields[1].1 {
                Value::Struct {
                    name: inner_name,
                    fields: inner_fields,
                } => {
                    assert_eq!(*inner_name, "Address");
                    assert_eq!(inner_fields[0], ("city", Value::String("NYC".to_string())));
                }
                other => panic!("expected nested Struct, got {other:?}"),
            }
        }
        _ => panic!("expected Struct, got {value:?}"),
    }

    assert_eq!(from_value::<Person>(value).unwrap(), person);
}

// ============================================================================
// Enum Tests
// ============================================================================

#[test]
fn test_unit_variant() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    enum Status {
        Active,
        Inactive,
    }

    let value = to_value(&Status::Active).unwrap();
    match &value {
        Value::UnitVariant {
            enum_name,
            variant_index,
            variant,
        } => {
            assert_eq!(*enum_name, "Status");
            assert_eq!(*variant_index, 0);
            assert_eq!(*variant, "Active");
        }
        _ => panic!("expected UnitVariant, got {value:?}"),
    }

    assert_eq!(from_value::<Status>(value).unwrap(), Status::Active);
}

#[test]
fn test_newtype_variant() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    enum Message {
        Text(String),
        Number(i32),
    }

    let value = to_value(&Message::Text("hello".to_string())).unwrap();
    match &value {
        Value::NewtypeVariant {
            enum_name,
            variant_index,
            variant,
            value: inner,
        } => {
            assert_eq!(*enum_name, "Message");
            assert_eq!(*variant_index, 0);
            assert_eq!(*variant, "Text");
            assert_eq!(**inner, Value::String("hello".to_string()));
        }
        _ => panic!("expected NewtypeVariant, got {value:?}"),
    }

    assert_eq!(
        from_value::<Message>(value).unwrap(),
        Message::Text("hello".to_string())
    );
}

#[test]
fn test_tuple_variant() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    enum Event {
        Click(i32, i32),
    }

    let value = to_value(&Event::Click(100, 200)).unwrap();
    match &value {
        Value::TupleVariant {
            enum_name,
            variant_index,
            variant,
            fields,
        } => {
            assert_eq!(*enum_name, "Event");
            assert_eq!(*variant_index, 0);
            assert_eq!(*variant, "Click");
            assert_eq!(fields, &vec![Value::I32(100), Value::I32(200)]);
        }
        _ => panic!("expected TupleVariant, got {value:?}"),
    }

    assert_eq!(from_value::<Event>(value).unwrap(), Event::Click(100, 200));
}

#[test]
fn test_struct_variant() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    enum Shape {
        Rectangle { width: u32, height: u32 },
    }

    let value = to_value(&Shape::Rectangle {
        width: 10,
        height: 20,
    })
    .unwrap();
    match &value {
        Value::StructVariant {
            enum_name,
            variant_index,
            variant,
            fields,
        } => {
            assert_eq!(*enum_name, "Shape");
            assert_eq!(*variant_index, 0);
            assert_eq!(*variant, "Rectangle");
            assert_eq!(fields.len(), 2);
            assert_eq!(fields[0], ("width", Value::U32(10)));
            assert_eq!(fields[1], ("height", Value::U32(20)));
        }
        _ => panic!("expected StructVariant, got {value:?}"),
    }

    assert_eq!(
        from_value::<Shape>(value).unwrap(),
        Shape::Rectangle {
            width: 10,
            height: 20
        }
    );
}

// ============================================================================
// Round-Trip Tests
// ============================================================================

#[test]
fn test_complex_round_trip() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Config {
        name: String,
        values: Vec<i32>,
        enabled: bool,
        nested: Option<Box<Config>>,
    }

    let config = Config {
        name: "root".to_string(),
        values: vec![1, 2, 3],
        enabled: true,
        nested: Some(Box::new(Config {
            name: "child".to_string(),
            values: vec![],
            enabled: false,
            nested: None,
        })),
    };

    let value = to_value(&config).unwrap();
    let result: Config = from_value(value).unwrap();
    assert_eq!(result, config);
}

// ============================================================================
// Value Equality Tests
// ============================================================================

#[test]
fn test_value_equality() {
    assert_eq!(Value::Bool(true), Value::Bool(true));
    assert_ne!(Value::Bool(true), Value::Bool(false));
    assert_ne!(Value::Bool(true), Value::I32(1));

    assert_eq!(Value::I32(42), Value::I32(42));
    assert_ne!(Value::I32(42), Value::I64(42)); // Different types

    // Float equality uses total_cmp
    assert_eq!(Value::F64(0.0), Value::F64(0.0));
    assert_ne!(Value::F64(0.0), Value::F64(-0.0)); // -0.0 != +0.0 with total_cmp

    // NaN equality (total_cmp makes NaN == NaN)
    assert_eq!(Value::F64(f64::NAN), Value::F64(f64::NAN));
}

#[test]
fn test_value_hash() {
    use std::collections::HashSet;

    let mut set = HashSet::new();
    set.insert(Value::I32(1));
    set.insert(Value::I32(2));
    set.insert(Value::I32(1)); // Duplicate

    assert_eq!(set.len(), 2);

    // Floats can be hashed
    let mut float_set = HashSet::new();
    float_set.insert(Value::F64(1.0));
    float_set.insert(Value::F64(2.0));
    assert_eq!(float_set.len(), 2);
}

// ============================================================================
// Serialize Value to JSON Tests
// ============================================================================

#[test]
fn test_serialize_value_to_json() {
    // Value -> JSON (via serde_json)
    let value = Value::Struct {
        name: "Test",
        fields: vec![
            ("x", Value::I32(10)),
            ("y", Value::String("hello".to_string())),
        ],
    };

    let json = serde_json::to_string(&value).unwrap();
    // Structs serialize as JSON objects
    assert!(json.contains("\"x\":10"));
    assert!(json.contains("\"y\":\"hello\""));
}

#[test]
fn test_deserialize_value_from_json() {
    // JSON -> Value (note: we lose struct names)
    let json = r#"{"name": "Alice", "age": 30}"#;
    let value: Value = serde_json::from_str(json).unwrap();

    // JSON objects become Maps, not Structs (no struct name in JSON)
    match value {
        Value::Map(entries) => {
            assert_eq!(entries.len(), 2);
        }
        _ => panic!("expected Map from JSON, got {value:?}"),
    }
}
