/// Tests to verify that positive and negative zero are properly distinguished
/// in Float and Value types for equality, ordering, and hashing.
use {
    jeb_value::{Float, Value},
    std::collections::{HashMap, hash_map::DefaultHasher},
    std::hash::{Hash, Hasher},
};
#[test]
fn test_float_zero_equality() {
    let pos_zero = Float::try_from(0.0f64).unwrap();
    let neg_zero = Float::try_from(-0.0f64).unwrap();
    assert_ne!(pos_zero, neg_zero);
    assert_ne!(neg_zero, pos_zero);
}
#[test]
fn test_float_zero_ordering() {
    let pos_zero = Float::try_from(0.0f64).unwrap();
    let neg_zero = Float::try_from(-0.0f64).unwrap();
    assert!(neg_zero < pos_zero);
    assert!(pos_zero > neg_zero);
    use std::cmp::Ordering;
    assert_eq!(neg_zero.cmp(& pos_zero), Ordering::Less);
    assert_eq!(pos_zero.cmp(& neg_zero), Ordering::Greater);
}
#[test]
fn test_float_zero_hashing() {
    let pos_zero = Float::try_from(0.0f64).unwrap();
    let neg_zero = Float::try_from(-0.0f64).unwrap();
    let mut hasher1 = DefaultHasher::new();
    let mut hasher2 = DefaultHasher::new();
    pos_zero.hash(&mut hasher1);
    neg_zero.hash(&mut hasher2);
    assert_ne!(hasher1.finish(), hasher2.finish());
}
#[test]
fn test_float_zero_as_hashmap_key() {
    let mut map = HashMap::new();
    let pos_zero = Float::try_from(0.0f64).unwrap();
    let neg_zero = Float::try_from(-0.0f64).unwrap();
    map.insert(pos_zero, "positive zero");
    map.insert(neg_zero, "negative zero");
    assert_eq!(map.len(), 2);
    assert_eq!(map.get(& pos_zero), Some(& "positive zero"));
    assert_eq!(map.get(& neg_zero), Some(& "negative zero"));
}
#[test]
fn test_float_zero_bit_representation() {
    let pos_zero = Float::try_from(0.0f64).unwrap();
    let neg_zero = Float::try_from(-0.0f64).unwrap();
    let pos_bits = (*pos_zero).to_bits();
    let neg_bits = (*neg_zero).to_bits();
    assert_ne!(pos_bits, neg_bits);
    assert_eq!(pos_bits, 0x0000_0000_0000_0000u64);
    assert_eq!(neg_bits, 0x8000_0000_0000_0000u64);
}
#[test]
fn test_value_zero_equality() {
    let pos_zero = Value::Float(Float::try_from(0.0f64).unwrap());
    let neg_zero = Value::Float(Float::try_from(-0.0f64).unwrap());
    assert_ne!(pos_zero, neg_zero);
    assert_ne!(neg_zero, pos_zero);
}
#[test]
fn test_value_zero_ordering() {
    let pos_zero = Value::Float(Float::try_from(0.0f64).unwrap());
    let neg_zero = Value::Float(Float::try_from(-0.0f64).unwrap());
    assert!(neg_zero < pos_zero);
    assert!(pos_zero > neg_zero);
    use std::cmp::Ordering;
    assert_eq!(neg_zero.cmp(& pos_zero), Ordering::Less);
    assert_eq!(pos_zero.cmp(& neg_zero), Ordering::Greater);
}
#[test]
fn test_value_zero_hashing() {
    let pos_zero = Value::Float(Float::try_from(0.0f64).unwrap());
    let neg_zero = Value::Float(Float::try_from(-0.0f64).unwrap());
    let mut hasher1 = DefaultHasher::new();
    let mut hasher2 = DefaultHasher::new();
    pos_zero.hash(&mut hasher1);
    neg_zero.hash(&mut hasher2);
    assert_ne!(hasher1.finish(), hasher2.finish());
}
#[test]
fn test_value_zero_as_hashmap_key() {
    let mut map = HashMap::new();
    let pos_zero = Value::Float(Float::try_from(0.0f64).unwrap());
    let neg_zero = Value::Float(Float::try_from(-0.0f64).unwrap());
    map.insert(pos_zero.clone(), "positive zero");
    map.insert(neg_zero.clone(), "negative zero");
    assert_eq!(map.len(), 2);
    assert_eq!(map.get(& pos_zero), Some(& "positive zero"));
    assert_eq!(map.get(& neg_zero), Some(& "negative zero"));
}
#[test]
fn test_value_zero_in_arrays() {
    let arr_pos = Value::from([Value::Float(Float::try_from(0.0f64).unwrap())]);
    let arr_neg = Value::from([Value::Float(Float::try_from(-0.0f64).unwrap())]);
    assert_ne!(arr_pos, arr_neg);
}
#[test]
fn test_value_zero_in_maps() {
    use jeb_value::Text;
    let map_pos: Value = [
        (Text::from("zero"), Value::Float(Float::try_from(0.0f64).unwrap())),
    ]
        .into_iter()
        .collect();
    let map_neg: Value = [
        (Text::from("zero"), Value::Float(Float::try_from(-0.0f64).unwrap())),
    ]
        .into_iter()
        .collect();
    assert_ne!(map_pos, map_neg);
}
#[test]
fn test_value_zero_cross_type_comparison() {
    use std::cmp::Ordering;
    let pos_zero_float = Value::Float(Float::try_from(0.0f64).unwrap());
    let neg_zero_float = Value::Float(Float::try_from(-0.0f64).unwrap());
    let unsigned_zero = Value::from(0u64);
    let signed_zero = Value::from(0i64);
    assert_eq!(unsigned_zero.cmp(& neg_zero_float), Ordering::Greater);
    assert_eq!(unsigned_zero.cmp(& pos_zero_float), Ordering::Less);
    assert_eq!(neg_zero_float.cmp(& pos_zero_float), Ordering::Less);
    assert_eq!(signed_zero.cmp(& neg_zero_float), Ordering::Greater);
    assert_eq!(signed_zero.cmp(& pos_zero_float), Ordering::Less);
}
#[test]
fn test_value_zero_sorted() {
    let mut values = [
        Value::Float(Float::try_from(0.0f64).unwrap()),
        Value::Float(Float::try_from(-0.0f64).unwrap()),
        Value::from(0u64),
        Value::from(0i64),
    ];
    values.sort();
    assert_eq!(values[0], Value::Float(Float::try_from(- 0.0f64).unwrap()));
    assert_eq!(values[1], Value::from(0u64));
    assert_eq!(values[2], Value::from(0i64));
    assert_eq!(values[3], Value::Float(Float::try_from(0.0f64).unwrap()));
}
