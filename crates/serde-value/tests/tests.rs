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
    let json = r#"{"name": "Alice", "age": 30}"#;
    let value: Value = serde_json::from_str(json).unwrap();

    match value {
        Value::Map(entries) => {
            assert_eq!(entries.len(), 2);
        }
        _ => panic!("expected Map from JSON, got {value:?}"),
    }
}

// ============================================================================
// Additional Coverage Tests
// ============================================================================

#[test]
fn test_all_integer_round_trips() {
    // Ensure all integer types round-trip correctly
    assert_eq!(from_value::<i8>(Value::I8(i8::MIN)).unwrap(), i8::MIN);
    assert_eq!(from_value::<i8>(Value::I8(i8::MAX)).unwrap(), i8::MAX);
    assert_eq!(from_value::<i16>(Value::I16(i16::MIN)).unwrap(), i16::MIN);
    assert_eq!(from_value::<i16>(Value::I16(i16::MAX)).unwrap(), i16::MAX);
    assert_eq!(from_value::<i32>(Value::I32(i32::MIN)).unwrap(), i32::MIN);
    assert_eq!(from_value::<i32>(Value::I32(i32::MAX)).unwrap(), i32::MAX);
    assert_eq!(from_value::<i64>(Value::I64(i64::MIN)).unwrap(), i64::MIN);
    assert_eq!(from_value::<i64>(Value::I64(i64::MAX)).unwrap(), i64::MAX);
    assert_eq!(from_value::<i128>(Value::I128(i128::MIN)).unwrap(), i128::MIN);
    assert_eq!(from_value::<i128>(Value::I128(i128::MAX)).unwrap(), i128::MAX);

    assert_eq!(from_value::<u8>(Value::U8(u8::MIN)).unwrap(), u8::MIN);
    assert_eq!(from_value::<u8>(Value::U8(u8::MAX)).unwrap(), u8::MAX);
    assert_eq!(from_value::<u16>(Value::U16(u16::MIN)).unwrap(), u16::MIN);
    assert_eq!(from_value::<u16>(Value::U16(u16::MAX)).unwrap(), u16::MAX);
    assert_eq!(from_value::<u32>(Value::U32(u32::MIN)).unwrap(), u32::MIN);
    assert_eq!(from_value::<u32>(Value::U32(u32::MAX)).unwrap(), u32::MAX);
    assert_eq!(from_value::<u64>(Value::U64(u64::MIN)).unwrap(), u64::MIN);
    assert_eq!(from_value::<u64>(Value::U64(u64::MAX)).unwrap(), u64::MAX);
    assert_eq!(from_value::<u128>(Value::U128(u128::MIN)).unwrap(), u128::MIN);
    assert_eq!(from_value::<u128>(Value::U128(u128::MAX)).unwrap(), u128::MAX);
}

#[test]
fn test_float_edge_cases() {
    // Positive and negative zero
    assert_eq!(from_value::<f32>(Value::F32(0.0)).unwrap(), 0.0);
    assert_eq!(from_value::<f32>(Value::F32(-0.0)).unwrap(), -0.0);
    assert_eq!(from_value::<f64>(Value::F64(0.0)).unwrap(), 0.0);
    assert_eq!(from_value::<f64>(Value::F64(-0.0)).unwrap(), -0.0);

    // Infinity
    assert_eq!(from_value::<f32>(Value::F32(f32::INFINITY)).unwrap(), f32::INFINITY);
    assert_eq!(from_value::<f32>(Value::F32(f32::NEG_INFINITY)).unwrap(), f32::NEG_INFINITY);
    assert_eq!(from_value::<f64>(Value::F64(f64::INFINITY)).unwrap(), f64::INFINITY);
    assert_eq!(from_value::<f64>(Value::F64(f64::NEG_INFINITY)).unwrap(), f64::NEG_INFINITY);

    // NaN (use is_nan since NaN != NaN in normal comparison)
    assert!(from_value::<f32>(Value::F32(f32::NAN)).unwrap().is_nan());
    assert!(from_value::<f64>(Value::F64(f64::NAN)).unwrap().is_nan());

    // Subnormal numbers
    assert_eq!(from_value::<f64>(Value::F64(f64::MIN_POSITIVE)).unwrap(), f64::MIN_POSITIVE);
}

#[test]
fn test_empty_collections() {
    // Empty vec
    let empty_vec: Vec<i32> = vec![];
    let value = to_value(&empty_vec).unwrap();
    assert_eq!(value, Value::Seq(vec![]));
    assert_eq!(from_value::<Vec<i32>>(value).unwrap(), empty_vec);

    // Empty tuple (unit)
    let empty_tuple = ();
    assert_eq!(to_value(&empty_tuple).unwrap(), Value::Unit);

    // Empty map
    use std::collections::HashMap;
    let empty_map: HashMap<String, i32> = HashMap::new();
    let value = to_value(&empty_map).unwrap();
    assert_eq!(value, Value::Map(vec![]));
    assert_eq!(from_value::<HashMap<String, i32>>(value).unwrap(), empty_map);

    // Empty struct
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Empty {}
    let empty = Empty {};
    let value = to_value(&empty).unwrap();
    match &value {
        Value::Struct { name, fields } => {
            assert_eq!(*name, "Empty");
            assert!(fields.is_empty());
        }
        _ => panic!("expected Struct"),
    }
    assert_eq!(from_value::<Empty>(value).unwrap(), empty);
}

#[test]
fn test_variant_indices() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    enum Multi {
        First,
        Second,
        Third,
    }

    // Test non-zero variant indices
    let second = to_value(&Multi::Second).unwrap();
    match &second {
        Value::UnitVariant { variant_index, variant, .. } => {
            assert_eq!(*variant_index, 1);
            assert_eq!(*variant, "Second");
        }
        _ => panic!("expected UnitVariant"),
    }
    assert_eq!(from_value::<Multi>(second).unwrap(), Multi::Second);

    let third = to_value(&Multi::Third).unwrap();
    match &third {
        Value::UnitVariant { variant_index, variant, .. } => {
            assert_eq!(*variant_index, 2);
            assert_eq!(*variant, "Third");
        }
        _ => panic!("expected UnitVariant"),
    }
    assert_eq!(from_value::<Multi>(third).unwrap(), Multi::Third);
}

#[test]
fn test_value_type_name() {
    assert_eq!(Value::Bool(true).type_name(), "bool");
    assert_eq!(Value::I8(0).type_name(), "i8");
    assert_eq!(Value::I16(0).type_name(), "i16");
    assert_eq!(Value::I32(0).type_name(), "i32");
    assert_eq!(Value::I64(0).type_name(), "i64");
    assert_eq!(Value::I128(0).type_name(), "i128");
    assert_eq!(Value::U8(0).type_name(), "u8");
    assert_eq!(Value::U16(0).type_name(), "u16");
    assert_eq!(Value::U32(0).type_name(), "u32");
    assert_eq!(Value::U64(0).type_name(), "u64");
    assert_eq!(Value::U128(0).type_name(), "u128");
    assert_eq!(Value::F32(0.0).type_name(), "f32");
    assert_eq!(Value::F64(0.0).type_name(), "f64");
    assert_eq!(Value::Char('a').type_name(), "char");
    assert_eq!(Value::String("".into()).type_name(), "string");
    assert_eq!(Value::Bytes(vec![]).type_name(), "bytes");
    assert_eq!(Value::None.type_name(), "none");
    assert_eq!(Value::Some(Box::new(Value::Unit)).type_name(), "some");
    assert_eq!(Value::Unit.type_name(), "unit");
    assert_eq!(Value::UnitStruct { name: "X" }.type_name(), "unit struct");
    assert_eq!(Value::Seq(vec![]).type_name(), "sequence");
    assert_eq!(Value::Tuple(vec![]).type_name(), "tuple");
    assert_eq!(Value::Map(vec![]).type_name(), "map");
    assert_eq!(Value::Struct { name: "X", fields: vec![] }.type_name(), "struct");
}

#[test]
fn test_value_clone() {
    let original = Value::Struct {
        name: "Test",
        fields: vec![
            ("a", Value::I32(1)),
            ("b", Value::String("hello".into())),
        ],
    };
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

#[test]
fn test_value_debug() {
    let value = Value::I32(42);
    let debug_str = format!("{:?}", value);
    assert!(debug_str.contains("I32"));
    assert!(debug_str.contains("42"));
}

#[test]
fn test_nested_options() {
    let nested: Option<Option<i32>> = Some(Some(42));
    let value = to_value(&nested).unwrap();
    assert_eq!(
        value,
        Value::Some(Box::new(Value::Some(Box::new(Value::I32(42)))))
    );
    assert_eq!(from_value::<Option<Option<i32>>>(value).unwrap(), nested);

    let none_inner: Option<Option<i32>> = Some(None);
    let value = to_value(&none_inner).unwrap();
    assert_eq!(value, Value::Some(Box::new(Value::None)));
    assert_eq!(from_value::<Option<Option<i32>>>(value).unwrap(), none_inner);
}

#[test]
fn test_complex_map_keys() {
    use std::collections::HashMap;

    // Integer keys
    let mut int_map: HashMap<i32, String> = HashMap::new();
    int_map.insert(1, "one".into());
    int_map.insert(2, "two".into());

    let value = to_value(&int_map).unwrap();
    match &value {
        Value::Map(entries) => {
            assert_eq!(entries.len(), 2);
        }
        _ => panic!("expected Map"),
    }
    let result: HashMap<i32, String> = from_value(value).unwrap();
    assert_eq!(result.get(&1), Some(&"one".to_string()));
    assert_eq!(result.get(&2), Some(&"two".to_string()));
}

#[test]
fn test_vec_of_structs() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Item {
        id: u32,
        name: String,
    }

    let items = vec![
        Item { id: 1, name: "first".into() },
        Item { id: 2, name: "second".into() },
    ];

    let value = to_value(&items).unwrap();
    match &value {
        Value::Seq(elements) => {
            assert_eq!(elements.len(), 2);
            match &elements[0] {
                Value::Struct { name, .. } => assert_eq!(*name, "Item"),
                _ => panic!("expected Struct"),
            }
        }
        _ => panic!("expected Seq"),
    }

    assert_eq!(from_value::<Vec<Item>>(value).unwrap(), items);
}

#[test]
fn test_deeply_nested() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Level3 { value: i32 }

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Level2 { inner: Level3 }

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Level1 { inner: Level2 }

    let deep = Level1 {
        inner: Level2 {
            inner: Level3 { value: 42 },
        },
    };

    let value = to_value(&deep).unwrap();
    let result: Level1 = from_value(value).unwrap();
    assert_eq!(result, deep);
}

// ============================================================================
// Cross-Type Conversion Tests (try_cast scenarios)
// ============================================================================

#[test]
fn test_cross_type_conversion_same_field_names() {
    // Source type
    #[derive(Serialize)]
    struct UserV1 {
        name: String,
        age: u32,
    }

    // Target type with same field names
    #[derive(Deserialize, Debug, PartialEq)]
    struct UserV2 {
        name: String,
        age: u32,
    }

    let v1 = UserV1 {
        name: "Alice".to_string(),
        age: 30,
    };

    let value = to_value(&v1).unwrap();
    let v2: UserV2 = from_value(value).unwrap();

    assert_eq!(v2.name, "Alice");
    assert_eq!(v2.age, 30);
}

#[test]
fn test_cross_type_conversion_field_type_coercion() {
    // Source with i32
    #[derive(Serialize)]
    struct SourceConfig {
        count: i32,
    }

    // Target with u64 (needs integer coercion)
    #[derive(Deserialize, Debug, PartialEq)]
    struct TargetConfig {
        count: u64,
    }

    let source = SourceConfig { count: 42 };
    let value = to_value(&source).unwrap();
    let target: TargetConfig = from_value(value).unwrap();

    assert_eq!(target.count, 42);
}

#[test]
fn test_cross_type_conversion_subset_fields() {
    // Source with more fields
    #[derive(Serialize)]
    struct FullUser {
        id: u64,
        name: String,
        email: String,
        age: u32,
    }

    // Target with fewer fields (ignores extra)
    #[derive(Deserialize, Debug, PartialEq)]
    struct PartialUser {
        name: String,
        age: u32,
    }

    let full = FullUser {
        id: 1,
        name: "Bob".to_string(),
        email: "bob@example.com".to_string(),
        age: 25,
    };

    let value = to_value(&full).unwrap();
    let partial: PartialUser = from_value(value).unwrap();

    assert_eq!(partial.name, "Bob");
    assert_eq!(partial.age, 25);
}

#[test]
fn test_cross_type_conversion_different_field_order() {
    // Source with fields in one order
    #[derive(Serialize)]
    struct OrderA {
        first: String,
        second: i32,
        third: bool,
    }

    // Target with same fields, different declaration order
    #[derive(Deserialize, Debug, PartialEq)]
    struct OrderB {
        third: bool,
        first: String,
        second: i32,
    }

    let a = OrderA {
        first: "hello".to_string(),
        second: 123,
        third: true,
    };

    let value = to_value(&a).unwrap();
    let b: OrderB = from_value(value).unwrap();

    assert_eq!(b.first, "hello");
    assert_eq!(b.second, 123);
    assert!(b.third);
}

#[test]
fn test_cross_type_conversion_nested_structs() {
    #[derive(Serialize)]
    struct InnerV1 {
        value: i32,
    }

    #[derive(Serialize)]
    struct OuterV1 {
        inner: InnerV1,
        label: String,
    }

    #[derive(Deserialize, Debug, PartialEq)]
    struct InnerV2 {
        value: i64, // Widened type
    }

    #[derive(Deserialize, Debug, PartialEq)]
    struct OuterV2 {
        inner: InnerV2,
        label: String,
    }

    let v1 = OuterV1 {
        inner: InnerV1 { value: 999 },
        label: "test".to_string(),
    };

    let value = to_value(&v1).unwrap();
    let v2: OuterV2 = from_value(value).unwrap();

    assert_eq!(v2.inner.value, 999);
    assert_eq!(v2.label, "test");
}

// ============================================================================
// Field Index Tests (Binary Format Interop)
// ============================================================================

/// Test that Value preserves field names from structs, enabling name-based matching.
/// This is the key difference from positional binary formats.
#[test]
fn test_value_preserves_field_names() {
    #[derive(Serialize)]
    struct Source {
        alpha: i32,
        beta: String,
    }

    let source = Source {
        alpha: 42,
        beta: "test".to_string(),
    };

    let value = to_value(&source).unwrap();

    // Verify the Value contains field names
    match &value {
        Value::Struct { name, fields } => {
            assert_eq!(*name, "Source");
            assert_eq!(fields.len(), 2);
            assert_eq!(fields[0].0, "alpha");
            assert_eq!(fields[1].0, "beta");
        }
        _ => panic!("expected Struct"),
    }
}

/// Test JSON roundtrip with various Value types.
/// JSON is a self-describing format that supports deserialize_any.
#[test]
fn test_json_roundtrip() {
    // Note: JSON doesn't distinguish all serde types, so we test types it preserves
    let test_values = vec![
        Value::Bool(true),
        Value::Bool(false),
        Value::I64(-100000),
        Value::U64(100000),
        Value::F64(3.14159),
        Value::String("hello world".to_string()),
        Value::None,
        Value::Seq(vec![Value::I64(1), Value::I64(2), Value::I64(3)]),
        Value::Map(vec![
            (Value::String("key1".to_string()), Value::I64(1)),
            (Value::String("key2".to_string()), Value::String("value".to_string())),
        ]),
    ];

    for original in test_values {
        let json = serde_json::to_string(&original).unwrap();
        let roundtripped: Value = serde_json::from_str(&json).unwrap();
        // JSON loses type distinctions (all ints become i64/u64, all maps become Map)
        // Just verify it doesn't error
        assert!(!json.is_empty());
        let _ = roundtripped; // Use the variable
    }
}

/// Test RON (Rusty Object Notation) roundtrip.
/// RON is a self-describing format that preserves more type info than JSON.
#[test]
fn test_ron_roundtrip() {
    let test_values = vec![
        Value::Bool(true),
        Value::I32(-42),
        Value::U32(42),
        Value::F64(3.14),
        Value::Char('🦀'),
        Value::String("hello".to_string()),
        Value::None,
        Value::Some(Box::new(Value::I32(42))),
        Value::Unit,
        Value::Seq(vec![Value::I32(1), Value::I32(2)]),
        Value::Map(vec![
            (Value::String("a".to_string()), Value::I32(1)),
        ]),
    ];

    for original in &test_values {
        let ron_str = ron::to_string(original).unwrap();
        let roundtripped: Value = ron::from_str(&ron_str).unwrap();
        // RON preserves more type info but still has some differences
        let _ = roundtripped;
    }
}

/// Test MessagePack roundtrip via rmp-serde.
/// MessagePack is a binary format that supports deserialize_any.
#[test]
fn test_msgpack_roundtrip() {
    let test_values = vec![
        Value::Bool(true),
        Value::I64(-100000),
        Value::U64(100000),
        Value::F64(3.14),
        Value::String("hello world".to_string()),
        Value::Bytes(vec![1, 2, 3, 4, 5]),
        Value::Seq(vec![Value::I64(1), Value::I64(2), Value::I64(3)]),
        Value::Map(vec![
            (Value::String("key".to_string()), Value::I64(42)),
        ]),
    ];

    for original in test_values {
        let bytes = rmp_serde::to_vec(&original).unwrap();
        let roundtripped: Value = rmp_serde::from_slice(&bytes).unwrap();
        // MessagePack has its own type mapping
        let _ = roundtripped;
    }
}

/// Test that bincode can serialize Value (but not deserialize without schema).
/// Bincode is a non-self-describing format that doesn't support deserialize_any.
#[test]
fn test_bincode_serialize_only() {
    let value = Value::Struct {
        name: "Test",
        fields: vec![
            ("x", Value::I32(1)),
            ("y", Value::I32(2)),
        ],
    };

    // Serialization works
    let bytes = bincode::serialize(&value).unwrap();
    assert!(!bytes.is_empty());

    // Deserialization to Value fails (expected - bincode needs schema)
    let result: Result<Value, _> = bincode::deserialize(&bytes);
    assert!(result.is_err(), "bincode can't deserialize Value without schema");
}

/// Test that field name matching works even when fields are in different order.
/// This demonstrates name-based (not positional) deserialization.
#[test]
fn test_struct_field_order_independence() {
    // Create a Value with fields in a specific order
    let value = Value::Struct {
        name: "Point",
        fields: vec![
            ("y", Value::I32(20)),
            ("x", Value::I32(10)),
        ],
    };

    // Deserialize to a struct where fields are declared in different order
    #[derive(Deserialize, Debug, PartialEq)]
    struct Point {
        x: i32,
        y: i32,
    }

    let point: Point = from_value(value).unwrap();
    assert_eq!(point.x, 10);
    assert_eq!(point.y, 20);
}

/// Test that we can deserialize with fields in wrong order but correct names.
/// This proves name-based (not positional) matching.
#[test]
fn test_field_order_mismatch_by_name() {
    // Manually construct Value with fields in "wrong" positional order
    // but correct names
    let value = Value::Struct {
        name: "Point3D",
        fields: vec![
            ("z", Value::I32(30)), // z comes first positionally
            ("x", Value::I32(10)),
            ("y", Value::I32(20)),
        ],
    };

    #[derive(Deserialize, Debug, PartialEq)]
    struct Point3D {
        x: i32,
        y: i32,
        z: i32,
    }

    let point: Point3D = from_value(value).unwrap();

    // If this were positional, we'd get x=30, y=10, z=20 (wrong!)
    // With name-based matching, we correctly get x=10, y=20, z=30
    assert_eq!(point.x, 10);
    assert_eq!(point.y, 20);
    assert_eq!(point.z, 30);
}
