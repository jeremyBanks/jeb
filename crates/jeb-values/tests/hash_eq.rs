use {
    jeb_values::{Bytes, Float, Text, Value},
    std::collections::{BTreeMap, HashMap, HashSet},
};

#[test]
fn test_value_as_hashmap_key() {
    let mut map = HashMap::new();

    map.insert(Value::from(42u64), "unsigned");
    map.insert(Value::from(-42i64), "signed");
    map.insert(Value::from(true), "bool");
    map.insert(Value::Null, "null");
    map.insert(Value::from("hello"), "text");
    map.insert(Value::from(vec![1u8, 2, 3]), "bytes");

    assert_eq!(map.get(&Value::from(42u64)), Some(&"unsigned"));
    assert_eq!(map.get(&Value::from(-42i64)), Some(&"signed"));
    assert_eq!(map.get(&Value::from(true)), Some(&"bool"));
    assert_eq!(map.get(&Value::Null), Some(&"null"));
    assert_eq!(map.get(&Value::from("hello")), Some(&"text"));
    assert_eq!(map.get(&Value::from(vec![1u8, 2, 3])), Some(&"bytes"));
}

#[test]
fn test_value_as_hashset_member() {
    let mut set = HashSet::new();

    set.insert(Value::from(42u64));
    set.insert(Value::from(-42i64));
    set.insert(Value::from(true));
    set.insert(Value::Null);
    set.insert(Value::from("hello"));

    assert!(set.contains(&Value::from(42u64)));
    assert!(set.contains(&Value::from(-42i64)));
    assert!(set.contains(&Value::from(true)));
    assert!(set.contains(&Value::Null));
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
    use jeb_values::Text;
    let mut outer_map = HashMap::new();

    let inner1: Value = [
        (Text::from("a"), Value::from(1u64)),
        (Text::from("b"), Value::from(2u64)),
    ]
    .into_iter()
    .collect();

    let inner2: Value = [
        (Text::from("a"), Value::from(1u64)),
        (Text::from("b"), Value::from(2u64)),
    ]
    .into_iter()
    .collect();

    outer_map.insert(inner1, "map");

    assert_eq!(outer_map.get(&inner2), Some(&"map"));
}

#[test]
fn test_eq_different_numeric_types() {
    // These should NOT be equal - different variants
    assert_ne!(Value::from(42u64), Value::from(42i64));

    // These SHOULD be equal - same variant, same value
    assert_eq!(Value::from(42u64), Value::from(42u64));
    assert_eq!(Value::from(-42i64), Value::from(-42i64));
}

#[test]
fn test_hash_consistency() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let v1 = Value::from(42u64);
    let v2 = Value::from(42u64);

    let mut hasher1 = DefaultHasher::new();
    let mut hasher2 = DefaultHasher::new();

    v1.hash(&mut hasher1);
    v2.hash(&mut hasher2);

    assert_eq!(hasher1.finish(), hasher2.finish());
}

// ============================================================================
// Ordering tests
// ============================================================================

#[test]
fn test_ordering_type_hierarchy() {
    use std::cmp::Ordering;

    // Type hierarchy (aligned with JSON serialization lexicographic order):
    // Bytes < Text < Number < Array < Bool(false) < Null < Bool(true) < BytesMap < TextMap

    // Bytes < Text
    assert_eq!(
        Value::from(vec![1u8, 2, 3]).cmp(&Value::from("hello")),
        Ordering::Less
    );

    // Text < Number
    assert_eq!(
        Value::from("hello").cmp(&Value::from(42u64)),
        Ordering::Less
    );

    // Number < Array
    assert_eq!(
        Value::from(42u64).cmp(&Value::from([Value::from(1u64), Value::from(2u64)])),
        Ordering::Less
    );

    // Array < Bool(false)
    assert_eq!(
        Value::from([Value::from(1u64)]).cmp(&Value::from(false)),
        Ordering::Less
    );

    // Bool(false) < Null
    assert_eq!(Value::from(false).cmp(&Value::Null), Ordering::Less);

    // Null < Bool(true)
    assert_eq!(Value::Null.cmp(&Value::from(true)), Ordering::Less);

    // Bool(true) < BytesMap
    let bytes_map: Value = [(Bytes::from(vec![1u8]), Value::from(1u64))]
        .into_iter()
        .collect();
    assert_eq!(Value::from(true).cmp(&bytes_map), Ordering::Less);

    // BytesMap < TextMap
    let text_map: Value = [(Text::from("a"), Value::from(1u64))]
        .into_iter()
        .collect();
    assert_eq!(bytes_map.cmp(&text_map), Ordering::Less);
}

#[test]
fn test_ordering_same_type() {
    use std::cmp::Ordering;

    // Booleans: false < true
    assert_eq!(Value::from(false).cmp(&Value::from(true)), Ordering::Less);
    assert_eq!(
        Value::from(true).cmp(&Value::from(false)),
        Ordering::Greater
    );
    assert_eq!(Value::from(true).cmp(&Value::from(true)), Ordering::Equal);

    // Unsigned integers
    assert_eq!(Value::from(10u64).cmp(&Value::from(20u64)), Ordering::Less);
    assert_eq!(
        Value::from(20u64).cmp(&Value::from(10u64)),
        Ordering::Greater
    );

    // Signed integers
    assert_eq!(
        Value::from(-10i64).cmp(&Value::from(-5i64)),
        Ordering::Less
    );
    assert_eq!(Value::from(-5i64).cmp(&Value::from(5i64)), Ordering::Less);

    // Bytes
    assert_eq!(
        Value::from(vec![1u8, 2]).cmp(&Value::from(vec![1u8, 3])),
        Ordering::Less
    );

    // Text
    assert_eq!(
        Value::from("apple").cmp(&Value::from("banana")),
        Ordering::Less
    );
}

#[test]
fn test_ordering_numeric_cross_type() {
    use std::cmp::Ordering;

    // Unsigned vs Signed: numeric comparison first
    assert_eq!(
        Value::from(10u64).cmp(&Value::from(-5i64)),
        Ordering::Greater
    ); // 10 > -5
    assert_eq!(Value::from(10u64).cmp(&Value::from(5i64)), Ordering::Greater); // 10 > 5
    assert_eq!(Value::from(10u64).cmp(&Value::from(10i64)), Ordering::Less); // Equal numerically, but Unsigned < Signed

    // Unsigned vs Float
    assert_eq!(
        Value::from(10u64).cmp(&Value::Float(Float::try_from(9.5).unwrap())),
        Ordering::Greater
    ); // 10.0 > 9.5
    assert_eq!(
        Value::from(10u64).cmp(&Value::Float(Float::try_from(10.5).unwrap())),
        Ordering::Less
    ); // 10.0 < 10.5
    assert_eq!(
        Value::from(10u64).cmp(&Value::Float(Float::try_from(10.0).unwrap())),
        Ordering::Less
    ); // Equal numerically, but Unsigned < Float

    // Signed vs Float
    assert_eq!(
        Value::from(-10i64).cmp(&Value::Float(Float::try_from(-9.5).unwrap())),
        Ordering::Less
    ); // -10.0 < -9.5
    assert_eq!(
        Value::from(10i64).cmp(&Value::Float(Float::try_from(10.0).unwrap())),
        Ordering::Less
    ); // Equal numerically, but Signed < Float
}

#[test]
fn test_ordering_arrays_lexicographic() {
    use std::cmp::Ordering;

    let arr1 = Value::from([Value::from(1u64), Value::from(2u64)]);
    let arr2 = Value::from([Value::from(1u64), Value::from(3u64)]);
    let arr3 = Value::from([Value::from(1u64), Value::from(2u64), Value::from(3u64)]);

    assert_eq!(arr1.cmp(&arr2), Ordering::Less); // [1, 2] < [1, 3]
    assert_eq!(arr1.cmp(&arr3), Ordering::Less); // [1, 2] < [1, 2, 3]
    assert_eq!(arr2.cmp(&arr3), Ordering::Greater); // [1, 3] > [1, 2, 3]
}

#[test]
fn test_ordering_maps_lexicographic() {
    use std::cmp::Ordering;

    let map1: Value = [(Text::from("a"), Value::from(1u64)), (Text::from("b"), Value::from(2u64))]
        .into_iter()
        .collect();

    let map2: Value = [(Text::from("a"), Value::from(1u64)), (Text::from("b"), Value::from(3u64))]
        .into_iter()
        .collect();

    let map3: Value = [
        (Text::from("a"), Value::from(1u64)),
        (Text::from("b"), Value::from(2u64)),
        (Text::from("c"), Value::from(3u64)),
    ]
    .into_iter()
    .collect();

    assert_eq!(map1.cmp(&map2), Ordering::Less); // {"a": 1, "b": 2} < {"a": 1, "b": 3}
    assert_eq!(map1.cmp(&map3), Ordering::Less); // {"a": 1, "b": 2} < {"a": 1, "b": 2, "c": 3}
}

#[test]
fn test_value_in_btreemap() {
    let mut map = BTreeMap::new();

    map.insert(Value::Null, "null");
    map.insert(Value::from(false), "false");
    map.insert(Value::from(true), "true");
    map.insert(Value::from(10u64), "ten");
    map.insert(Value::from(-5i64), "neg five");
    map.insert(Value::from("hello"), "text");
    map.insert(Value::from(vec![1u8, 2, 3]), "bytes");

    // Verify ordering: Bytes < Text < Numbers < Array < Bool(false) < Null < Bool(true)
    let keys: Vec<_> = map.keys().cloned().collect();
    assert_eq!(keys[0], Value::from(vec![1u8, 2, 3])); // Bytes
    assert_eq!(keys[1], Value::from("hello")); // Text
    assert_eq!(keys[2], Value::from(-5i64)); // Number (negative)
    assert_eq!(keys[3], Value::from(10u64)); // Number (positive unsigned)
    assert_eq!(keys[4], Value::from(false)); // Bool(false)
    assert_eq!(keys[5], Value::Null); // Null
    assert_eq!(keys[6], Value::from(true)); // Bool(true)
}

#[test]
fn test_cmp_by_complexity_public_api() {
    use std::cmp::Ordering;

    // Test that the public method works correctly
    let v1 = Value::from(42u64);
    let v2 = Value::from(100u64);

    assert_eq!(v1.cmp_by_complexity(&v2), Ordering::Less);
    assert_eq!(v2.cmp_by_complexity(&v1), Ordering::Greater);
    assert_eq!(v1.cmp_by_complexity(&v1), Ordering::Equal);
}

#[test]
fn test_sorted_values() {
    let mut values = vec![
        Value::from("zebra"),
        Value::from(42u64),
        Value::Null,
        Value::from(true),
        Value::from(-10i64),
        Value::from([Value::from(1u64)]),
        Value::from("apple"),
        Value::from(vec![1u8, 2, 3]),
        Value::from(false),
    ];

    values.sort();

    // Expected order: Bytes < Text < Numbers < Array < Bool(false) < Null < Bool(true)
    assert_eq!(values[0], Value::from(vec![1u8, 2, 3])); // Bytes
    assert_eq!(values[1], Value::from("apple")); // Text (sorted)
    assert_eq!(values[2], Value::from("zebra")); // Text (sorted)
    assert_eq!(values[3], Value::from(-10i64)); // Numbers
    assert_eq!(values[4], Value::from(42u64));
    assert_eq!(values[5], Value::from([Value::from(1u64)])); // Array
    assert_eq!(values[6], Value::from(false)); // Bool(false)
    assert_eq!(values[7], Value::Null); // Null
    assert_eq!(values[8], Value::from(true)); // Bool(true)
}
