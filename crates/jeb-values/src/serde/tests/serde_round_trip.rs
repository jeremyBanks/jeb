use {
    jeb_values::{
        Bytes,
        Value,
        from_value,
        to_value,
    },
    serde::{
        Deserialize,
        Serialize,
    },
    std::collections::HashMap,
};

#[test]
fn test_primitives() {
    // Booleans
    assert_round_trip(true);
    assert_round_trip(false);

    // Integers - signed
    assert_round_trip(42i8);
    assert_round_trip(-42i8);
    assert_round_trip(1000i16);
    assert_round_trip(-1000i16);
    assert_round_trip(100_000i32);
    assert_round_trip(-100_000i32);
    assert_round_trip(1_000_000_000i64);
    assert_round_trip(-1_000_000_000i64);

    // Integers - unsigned
    assert_round_trip(255u8);
    assert_round_trip(65535u16);
    assert_round_trip(4_000_000_000u32);
    assert_round_trip(18_000_000_000_000_000_000u64);

    // Floats
    assert_round_trip(core::f32::consts::PI);
    assert_round_trip(-core::f32::consts::PI);
    assert_round_trip(core::f64::consts::E);
    assert_round_trip(-core::f64::consts::E);
    assert_round_trip(0.0f64);
    assert_round_trip(-0.0f64);

    // Char
    assert_round_trip('A');
    assert_round_trip('€');
    assert_round_trip('🦀');

    // String
    assert_round_trip("hello".to_string());
    assert_round_trip("".to_string());
    assert_round_trip("with\nnewlines\tand\ttabs".to_string());
}

#[test]
fn test_i128_u128() {
    // Values that fit in i64/u64
    assert_round_trip(42i128);
    assert_round_trip(-42i128);
    assert_round_trip(42u128);

    // Values that overflow - require bytes encoding
    let large_i128 = i128::MAX;
    let value = to_value(large_i128).unwrap();
    assert!(matches!(value, Value::Bytes(_)));
    let recovered: i128 = from_value(value).unwrap();
    assert_eq!(recovered, large_i128);

    let large_u128 = u128::MAX;
    let value = to_value(large_u128).unwrap();
    assert!(matches!(value, Value::Bytes(_)));
    let recovered: u128 = from_value(value).unwrap();
    assert_eq!(recovered, large_u128);
}

#[test]
fn test_special_floats() {
    // NaN and infinity should serialize to bytes
    let nan_f32 = f32::NAN;
    let value = to_value(nan_f32).unwrap();
    assert!(matches!(value, Value::Bytes(_)));
    let recovered: f32 = from_value(value).unwrap();
    assert!(recovered.is_nan());

    let inf_f64 = f64::INFINITY;
    let value = to_value(inf_f64).unwrap();
    assert!(matches!(value, Value::Bytes(_)));
    let recovered: f64 = from_value(value).unwrap();
    assert!(recovered.is_infinite() && recovered.is_sign_positive());

    let neg_inf_f32 = f32::NEG_INFINITY;
    let value = to_value(neg_inf_f32).unwrap();
    assert!(matches!(value, Value::Bytes(_)));
    let recovered: f32 = from_value(value).unwrap();
    assert!(recovered.is_infinite() && recovered.is_sign_negative());
}

#[test]
fn test_option() {
    // None serializes to Null
    let none: Option<i32> = None;
    let value = to_value(none).unwrap();
    assert!(matches!(value, Value::Null));
    let recovered: Option<i32> = from_value(value).unwrap();
    assert_eq!(recovered, None);

    // Some serializes to {"Some": value}
    let some = Some(42);
    let value = to_value(some).unwrap();
    assert!(matches!(value, Value::TextMap(_)));
    let recovered: Option<i32> = from_value(value).unwrap();
    assert_eq!(recovered, Some(42));
}

#[test]
fn test_nested_option() {
    // None
    let none: Option<Option<i32>> = None;
    assert_round_trip(none);

    // Some(None)
    let some_none: Option<Option<i32>> = Some(None);
    assert_round_trip(some_none);

    // Some(Some(42))
    let some_some: Option<Option<i32>> = Some(Some(42));
    assert_round_trip(some_some);
}

#[test]
fn test_sequences() {
    assert_round_trip(vec![1, 2, 3, 4, 5]);
    assert_round_trip(Vec::<i32>::new());
    assert_round_trip(vec!["a".to_string(), "b".to_string(), "c".to_string()]);
}

#[test]
fn test_tuples() {
    assert_round_trip((1, 2));
    assert_round_trip((1, "hello".to_string(), core::f64::consts::PI));
    assert_round_trip((true, false, true, false));
}

#[test]
fn test_maps() {
    let mut map = HashMap::new();
    map.insert("key1".to_string(), 1);
    map.insert("key2".to_string(), 2);

    let value = to_value(&map).unwrap();
    assert!(matches!(value, Value::TextMap(_)));

    let recovered: HashMap<String, i32> = from_value(value).unwrap();
    assert_eq!(recovered.get("key1"), Some(&1));
    assert_eq!(recovered.get("key2"), Some(&2));

    // Empty map
    let empty: HashMap<String, i32> = HashMap::new();
    let value = to_value(&empty).unwrap();
    assert!(matches!(value, Value::Array(_))); // Empty maps -> []
    let recovered: HashMap<String, i32> = from_value(value).unwrap();
    assert!(recovered.is_empty());
}

#[test]
fn test_structs() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Person {
        name: String,
        age: u32,
    }

    let person = Person {
        name: "Alice".to_string(),
        age: 30,
    };

    assert_round_trip(person);
}

#[test]
fn test_unit_struct() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Unit;

    assert_round_trip(Unit);
}

#[test]
fn test_newtype_struct() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Meters(u64);

    assert_round_trip(Meters(42));
}

#[test]
fn test_tuple_struct() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Rgb(u8, u8, u8);

    assert_round_trip(Rgb(255, 128, 0));
}

#[test]
fn test_enums() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    enum Message {
        Quit,
        Move { x: i32, y: i32 },
        Write(String),
        ChangeColor(u8, u8, u8),
    }

    assert_round_trip(Message::Quit);
    assert_round_trip(Message::Move { x: 10, y: 20 });
    assert_round_trip(Message::Write("hello".to_string()));
    assert_round_trip(Message::ChangeColor(255, 0, 128));
}

#[test]
fn test_bytes() {
    // Vec<u8> serializes as a sequence by default in serde, not bytes
    let vec_bytes = vec![0u8, 1, 2, 3, 255];
    let value = to_value(&vec_bytes).unwrap();
    // Vec<u8> becomes Array of Unsigned
    assert!(matches!(value, Value::Array(_)));
    let recovered: Vec<u8> = from_value(value).unwrap();
    assert_eq!(recovered, vec_bytes);

    // But our Bytes type uses serialize_bytes, so it becomes Value::Bytes!
    let bytes = Bytes::from(vec![0u8, 1, 2, 3, 255]);
    let value = to_value(&bytes).unwrap();
    assert!(matches!(value, Value::Bytes(_)));
    let recovered: Bytes = from_value(value).unwrap();
    assert_eq!(recovered, bytes);

    // Our deserializer accepts arrays as bytes too
    let arr_value = Value::Array(vec![
        Value::Unsigned(0),
        Value::Unsigned(1),
        Value::Unsigned(255),
    ]);
    let as_vec: Vec<u8> = from_value(arr_value.clone()).unwrap();
    assert_eq!(as_vec, vec![0, 1, 255]);
    let as_bytes: Bytes = from_value(arr_value).unwrap();
    assert_eq!(as_bytes, Bytes::from(vec![0, 1, 255]));
}

#[test]
fn test_complex_nested() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Complex {
        number: i32,
        text: String,
        optional: Option<Vec<i32>>,
        nested: Option<Option<bool>>,
    }

    let complex = Complex {
        number: 42,
        text: "test".to_string(),
        optional: Some(vec![1, 2, 3]),
        nested: Some(None),
    };

    assert_round_trip(complex);
}

fn assert_round_trip<T>(original: T)
where
    T: Serialize + for<'de> Deserialize<'de> + PartialEq + std::fmt::Debug,
{
    let value = to_value(&original).unwrap();
    let recovered: T = from_value(value).unwrap();
    assert_eq!(original, recovered);
}
