use jeb_values::{from_value, to_value, Value};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Test complex map keys (tuples, structs)
/// This is useful for composite indexing in data structures like BTrees

#[test]
fn test_tuple_map_keys() {
    // Tuple keys for composite indexing
    let mut map = HashMap::new();
    map.insert((1, "a".to_string()), "value1".to_string());
    map.insert((2, "b".to_string()), "value2".to_string());
    map.insert((1, "b".to_string()), "value3".to_string());

    let value = to_value(&map).unwrap();

    // Maps with complex keys serialize as Array of [key, value] pairs
    assert!(matches!(value, Value::Array(_)));

    // Round-trip
    let recovered: HashMap<(i32, String), String> = from_value(value).unwrap();
    assert_eq!(recovered.get(&(1, "a".to_string())), Some(&"value1".to_string()));
    assert_eq!(recovered.get(&(2, "b".to_string())), Some(&"value2".to_string()));
    assert_eq!(recovered.get(&(1, "b".to_string())), Some(&"value3".to_string()));
}

#[test]
fn test_struct_map_keys() {
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    struct CompositeKey {
        user_id: u32,
        category: String,
    }

    let mut map = HashMap::new();
    map.insert(
        CompositeKey {
            user_id: 1,
            category: "books".to_string(),
        },
        vec!["item1".to_string(), "item2".to_string()],
    );
    map.insert(
        CompositeKey {
            user_id: 2,
            category: "electronics".to_string(),
        },
        vec!["item3".to_string()],
    );

    let value = to_value(&map).unwrap();

    // Maps with struct keys serialize as Array of [key, value] pairs
    assert!(matches!(value, Value::Array(_)));

    // Round-trip
    let recovered: HashMap<CompositeKey, Vec<String>> = from_value(value).unwrap();
    assert_eq!(
        recovered.get(&CompositeKey {
            user_id: 1,
            category: "books".to_string()
        }),
        Some(&vec!["item1".to_string(), "item2".to_string()])
    );
}

#[test]
fn test_nested_tuple_keys() {
    // More complex: nested tuples
    let mut map = HashMap::new();
    map.insert(((1, 2), (3, 4)), "nested".to_string());
    map.insert(((5, 6), (7, 8)), "another".to_string());

    let value = to_value(&map).unwrap();
    let recovered: HashMap<((i32, i32), (i32, i32)), String> = from_value(value).unwrap();
    assert_eq!(recovered.get(&((1, 2), (3, 4))), Some(&"nested".to_string()));
    assert_eq!(recovered.get(&((5, 6), (7, 8))), Some(&"another".to_string()));
}

#[test]
fn test_mixed_tuple_keys() {
    // Tuple with different types
    let mut map = HashMap::new();
    map.insert((1, "key".to_string(), true), 100);
    map.insert((2, "other".to_string(), false), 200);

    let value = to_value(&map).unwrap();
    let recovered: HashMap<(i32, String, bool), i32> = from_value(value).unwrap();
    assert_eq!(recovered.get(&(1, "key".to_string(), true)), Some(&100));
    assert_eq!(recovered.get(&(2, "other".to_string(), false)), Some(&200));
}

#[test]
fn test_tuple_struct_keys() {
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    struct Point(i32, i32);

    let mut map = HashMap::new();
    map.insert(Point(0, 0), "origin".to_string());
    map.insert(Point(1, 1), "diagonal".to_string());

    let value = to_value(&map).unwrap();
    let recovered: HashMap<Point, String> = from_value(value).unwrap();
    assert_eq!(recovered.get(&Point(0, 0)), Some(&"origin".to_string()));
    assert_eq!(recovered.get(&Point(1, 1)), Some(&"diagonal".to_string()));
}

#[test]
fn test_seq_keys() {
    // Vec as key (serialized as seq)
    let mut map = HashMap::new();
    map.insert(vec![1, 2, 3], "list1".to_string());
    map.insert(vec![4, 5], "list2".to_string());

    let value = to_value(&map).unwrap();

    // Maps with Vec keys serialize as Array of [key, value] pairs
    assert!(matches!(value, Value::Array(_)));

    let recovered: HashMap<Vec<i32>, String> = from_value(value).unwrap();
    assert_eq!(recovered.get(&vec![1, 2, 3]), Some(&"list1".to_string()));
    assert_eq!(recovered.get(&vec![4, 5]), Some(&"list2".to_string()));
}
