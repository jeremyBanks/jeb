use {
    crate::{Bytes, Null, Number, String, Value},
    std::collections::{BTreeMap, HashMap, HashSet},
};
#[test]
fn test_value_as_hashmap_key() {
    let mut map = HashMap::new();
    map.insert(Value::from(42u64), "number");
    map.insert(Value::from(true), "bool");
    map.insert(Value::Null(Null::new()), "null");
    map.insert(Value::from("hello"), "string");
    map.insert(Value::from(vec![1u8, 2, 3]), "bytes");
    assert_eq!(map.get(&Value::from(42u64)), Some(&"number"));
    assert_eq!(map.get(&Value::from(42i64)), Some(&"number")); // Same value
    assert_eq!(map.get(&Value::from(true)), Some(&"bool"));
    assert_eq!(map.get(&Value::Null(Null::new())), Some(&"null"));
    assert_eq!(map.get(&Value::from("hello")), Some(&"string"));
    assert_eq!(map.get(&Value::from(vec![1u8, 2, 3])), Some(&"bytes"));
}
#[test]
fn test_value_as_hashset_member() {
    let mut set = HashSet::new();
    set.insert(Value::from(42u64));
    set.insert(Value::from(-42i64));
    set.insert(Value::from(true));
    set.insert(Value::Null(Null::new()));
    set.insert(Value::from("hello"));
    assert!(set.contains(&Value::from(42u64)));
    assert!(set.contains(&Value::from(42i64))); // Same as 42u64
    assert!(set.contains(&Value::from(-42i64)));
    assert!(set.contains(&Value::from(true)));
    assert!(set.contains(&Value::Null(Null::new())));
    assert!(set.contains(&Value::from("hello")));
    assert!(!set.contains(&Value::from(999u64)));
}
#[test]
fn test_array_value_hash() {
    let mut map = HashMap::new();
    let arr1 = Value::from([Value::from(1u64), Value::from(2u64), Value::from(3u64)]);
    let arr2 = Value::from([Value::from(1u64), Value::from(2u64), Value::from(3u64)]);
    map.insert(arr1, "array");
    assert_eq!(map.get(&arr2), Some(&"array"));
}
#[test]
fn test_map_value_hash() {
    let mut outer_map = HashMap::new();
    let inner1: Value = [
        (String::from("a"), Value::from(1u64)),
        (String::from("b"), Value::from(2u64)),
    ]
    .into_iter()
    .collect();
    let inner2: Value = [
        (String::from("a"), Value::from(1u64)),
        (String::from("b"), Value::from(2u64)),
    ]
    .into_iter()
    .collect();
    outer_map.insert(inner1, "map");
    assert_eq!(outer_map.get(&inner2), Some(&"map"));
}
#[test]
fn test_eq_same_numeric_value() {
    // In the new model, u64 and i64 with the same value are equal
    assert_eq!(Value::from(42u64), Value::from(42i64));
    assert_eq!(Value::from(42u64), Value::from(42u64));
    assert_eq!(Value::from(-42i64), Value::from(-42i64));
}
#[test]
fn test_hash_consistency() {
    use std::{
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
    };
    let v1 = Value::from(42u64);
    let v2 = Value::from(42u64);
    let mut hasher1 = DefaultHasher::new();
    let mut hasher2 = DefaultHasher::new();
    v1.hash(&mut hasher1);
    v2.hash(&mut hasher2);
    assert_eq!(hasher1.finish(), hasher2.finish());
}
#[test]
fn test_ordering_type_hierarchy() {
    use std::cmp::Ordering;
    // Null < Boolean < Number < Bytes < String < Array < BytesMap < StringMap
    assert_eq!(Value::Null(Null::new()).cmp(&Value::from(false)), Ordering::Less);
    assert_eq!(Value::from(false).cmp(&Value::from(42u64)), Ordering::Less);
    assert_eq!(
        Value::from(42u64).cmp(&Value::from(vec![1u8, 2, 3])),
        Ordering::Less
    );
    assert_eq!(
        Value::from(vec![1u8, 2, 3]).cmp(&Value::from("hello")),
        Ordering::Less
    );
    assert_eq!(
        Value::from("hello").cmp(&Value::from([Value::from(1u64), Value::from(2u64)])),
        Ordering::Less
    );
    let bytes_map: Value = [(Bytes::from(vec![1u8]), Value::from(1u64))]
        .into_iter()
        .collect();
    assert_eq!(
        Value::from([Value::from(1u64)]).cmp(&bytes_map),
        Ordering::Less
    );
    let string_map: Value = [(String::from("a"), Value::from(1u64))]
        .into_iter()
        .collect();
    assert_eq!(bytes_map.cmp(&string_map), Ordering::Less);
}
#[test]
fn test_ordering_same_type() {
    use std::cmp::Ordering;
    // Boolean ordering
    assert_eq!(Value::from(false).cmp(&Value::from(true)), Ordering::Less);
    assert_eq!(
        Value::from(true).cmp(&Value::from(false)),
        Ordering::Greater
    );
    assert_eq!(Value::from(true).cmp(&Value::from(true)), Ordering::Equal);
    // Number ordering
    assert_eq!(Value::from(10u64).cmp(&Value::from(20u64)), Ordering::Less);
    assert_eq!(
        Value::from(20u64).cmp(&Value::from(10u64)),
        Ordering::Greater
    );
    assert_eq!(Value::from(-10i64).cmp(&Value::from(-5i64)), Ordering::Less);
    assert_eq!(Value::from(-5i64).cmp(&Value::from(5i64)), Ordering::Less);
    // Bytes ordering
    assert_eq!(
        Value::from(vec![1u8, 2]).cmp(&Value::from(vec![1u8, 3])),
        Ordering::Less
    );
    // String ordering
    assert_eq!(
        Value::from("apple").cmp(&Value::from("banana")),
        Ordering::Less
    );
}
#[test]
fn test_ordering_arrays_lexicographic() {
    use std::cmp::Ordering;
    let arr1 = Value::from([Value::from(1u64), Value::from(2u64)]);
    let arr2 = Value::from([Value::from(1u64), Value::from(3u64)]);
    let arr3 = Value::from([Value::from(1u64), Value::from(2u64), Value::from(3u64)]);
    assert_eq!(arr1.cmp(&arr2), Ordering::Less);
    assert_eq!(arr1.cmp(&arr3), Ordering::Less);
    assert_eq!(arr2.cmp(&arr3), Ordering::Greater);
}
#[test]
fn test_ordering_maps_lexicographic() {
    use std::cmp::Ordering;
    let map1: Value = [
        (String::from("a"), Value::from(1u64)),
        (String::from("b"), Value::from(2u64)),
    ]
    .into_iter()
    .collect();
    let map2: Value = [
        (String::from("a"), Value::from(1u64)),
        (String::from("b"), Value::from(3u64)),
    ]
    .into_iter()
    .collect();
    let map3: Value = [
        (String::from("a"), Value::from(1u64)),
        (String::from("b"), Value::from(2u64)),
        (String::from("c"), Value::from(3u64)),
    ]
    .into_iter()
    .collect();
    assert_eq!(map1.cmp(&map2), Ordering::Less);
    assert_eq!(map1.cmp(&map3), Ordering::Less);
}
#[test]
fn test_value_in_btreemap() {
    let mut map = BTreeMap::new();
    map.insert(Value::Null(Null::new()), "null");
    map.insert(Value::from(false), "false");
    map.insert(Value::from(true), "true");
    map.insert(Value::from(10u64), "ten");
    map.insert(Value::from(-5i64), "neg five");
    map.insert(Value::from("hello"), "string");
    map.insert(Value::from(vec![1u8, 2, 3]), "bytes");
    let keys: Vec<_> = map.keys().cloned().collect();
    // Order: Null < Boolean < Number < Bytes < String < Array < BytesMap < StringMap
    assert_eq!(keys[0], Value::Null(Null::new())); // Null
    assert_eq!(keys[1], Value::from(false)); // Boolean
    assert_eq!(keys[2], Value::from(true)); // Boolean (true > false)
    assert_eq!(keys[3], Value::from(-5i64)); // Number
    assert_eq!(keys[4], Value::from(10u64)); // Number
    assert_eq!(keys[5], Value::from(vec![1u8, 2, 3])); // Bytes
    assert_eq!(keys[6], Value::from("hello")); // String
}
#[test]
fn test_cmp() {
    use std::cmp::Ordering;
    let v1 = Value::from(42u64);
    let v2 = Value::from(100u64);
    assert_eq!(v1.cmp(&v2), Ordering::Less);
    assert_eq!(v2.cmp(&v1), Ordering::Greater);
    assert_eq!(v1.cmp(&v1), Ordering::Equal);
}
#[test]
fn test_sorted_values() {
    let mut values = vec![
        Value::from("zebra"),
        Value::from(42u64),
        Value::Null(Null::new()),
        Value::from(true),
        Value::from(-10i64),
        Value::from([Value::from(1u64)]),
        Value::from("apple"),
        Value::from(vec![1u8, 2, 3]),
        Value::from(false),
    ];
    values.sort();
    // Order: Null < Boolean < Number < Bytes < String < Array
    assert_eq!(values[0], Value::Null(Null::new())); // Null
    assert_eq!(values[1], Value::from(false)); // Boolean
    assert_eq!(values[2], Value::from(true)); // Boolean
    assert_eq!(values[3], Value::from(-10i64)); // Number
    assert_eq!(values[4], Value::from(42u64)); // Number
    assert_eq!(values[5], Value::from(vec![1u8, 2, 3])); // Bytes
    assert_eq!(values[6], Value::from("apple")); // String
    assert_eq!(values[7], Value::from("zebra")); // String
    assert_eq!(values[8], Value::from([Value::from(1u64)])); // Array
}
