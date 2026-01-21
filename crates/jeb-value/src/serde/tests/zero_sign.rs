/// Tests to verify that positive and negative zero are properly distinguished
/// in Number and Value types for equality, ordering, and hashing.
use {
    jeb_value::{Number, Value},
    std::collections::{hash_map::DefaultHasher, HashMap},
    std::hash::{Hash, Hasher},
};
#[test]
// [verify jeb-value.variant.common.eq]
// [verify jeb-value.variant.common.partial-eq]
// [verify jeb-value.number.eq-total-cmp]
// [verify jeb-value.number.partial-eq-total-cmp]
fn test_number_zero_equality() {
    let pos_zero = Number::new(0.0f64).unwrap();
    let neg_zero = Number::new(-0.0f64).unwrap();
    assert_ne!(pos_zero, neg_zero);
    assert_ne!(neg_zero, pos_zero);
}
#[test]
// [verify jeb-value.variant.common.ord]
// [verify jeb-value.variant.common.partial-ord]
// [verify jeb-value.number.ord-total-cmp]
// [verify jeb-value.number.partial-ord-total-cmp]
fn test_number_zero_ordering() {
    let pos_zero = Number::new(0.0f64).unwrap();
    let neg_zero = Number::new(-0.0f64).unwrap();
    assert!(neg_zero < pos_zero);
    assert!(pos_zero > neg_zero);
    use std::cmp::Ordering;
    assert_eq!(neg_zero.cmp(&pos_zero), Ordering::Less);
    assert_eq!(pos_zero.cmp(&neg_zero), Ordering::Greater);
}
#[test]
// [verify jeb-value.variant.common.hash]
// [verify jeb-value.number.hash-to-be-bytes]
fn test_number_zero_hashing() {
    let pos_zero = Number::new(0.0f64).unwrap();
    let neg_zero = Number::new(-0.0f64).unwrap();
    let mut hasher1 = DefaultHasher::new();
    let mut hasher2 = DefaultHasher::new();
    pos_zero.hash(&mut hasher1);
    neg_zero.hash(&mut hasher2);
    assert_ne!(hasher1.finish(), hasher2.finish());
}
#[test]
// [verify jeb-value.variant.common.eq]
// [verify jeb-value.variant.common.hash]
// [verify jeb-value.number.eq-total-cmp]
// [verify jeb-value.number.hash-to-be-bytes]
fn test_number_zero_as_hashmap_key() {
    let mut map = HashMap::new();
    let pos_zero = Number::new(0.0f64).unwrap();
    let neg_zero = Number::new(-0.0f64).unwrap();
    map.insert(pos_zero, "positive zero");
    map.insert(neg_zero, "negative zero");
    assert_eq!(map.len(), 2);
    assert_eq!(map.get(&pos_zero), Some(&"positive zero"));
    assert_eq!(map.get(&neg_zero), Some(&"negative zero"));
}
#[test]
// [verify jeb-value.number.hash-to-be-bytes]
fn test_number_zero_bit_representation() {
    let pos_zero = Number::new(0.0f64).unwrap();
    let neg_zero = Number::new(-0.0f64).unwrap();
    let pos_bits = (*pos_zero).to_bits();
    let neg_bits = (*neg_zero).to_bits();
    assert_ne!(pos_bits, neg_bits);
    assert_eq!(pos_bits, 0x0000_0000_0000_0000u64);
    assert_eq!(neg_bits, 0x8000_0000_0000_0000u64);
}
#[test]
// [verify jeb-value.value.traits.eq]
// [verify jeb-value.value.traits.partial-eq]
fn test_value_zero_equality() {
    let pos_zero = Value::Number(Number::new(0.0f64).unwrap());
    let neg_zero = Value::Number(Number::new(-0.0f64).unwrap());
    assert_ne!(pos_zero, neg_zero);
    assert_ne!(neg_zero, pos_zero);
}
#[test]
// [verify jeb-value.value.traits.ord]
// [verify jeb-value.value.traits.partial-ord]
fn test_value_zero_ordering() {
    let pos_zero = Value::Number(Number::new(0.0f64).unwrap());
    let neg_zero = Value::Number(Number::new(-0.0f64).unwrap());
    assert!(neg_zero < pos_zero);
    assert!(pos_zero > neg_zero);
    use std::cmp::Ordering;
    assert_eq!(neg_zero.cmp(&pos_zero), Ordering::Less);
    assert_eq!(pos_zero.cmp(&neg_zero), Ordering::Greater);
}
#[test]
// [verify jeb-value.value.traits.hash]
fn test_value_zero_hashing() {
    let pos_zero = Value::Number(Number::new(0.0f64).unwrap());
    let neg_zero = Value::Number(Number::new(-0.0f64).unwrap());
    let mut hasher1 = DefaultHasher::new();
    let mut hasher2 = DefaultHasher::new();
    pos_zero.hash(&mut hasher1);
    neg_zero.hash(&mut hasher2);
    assert_ne!(hasher1.finish(), hasher2.finish());
}
#[test]
fn test_value_zero_as_hashmap_key() {
    let mut map = HashMap::new();
    let pos_zero = Value::Number(Number::new(0.0f64).unwrap());
    let neg_zero = Value::Number(Number::new(-0.0f64).unwrap());
    map.insert(pos_zero.clone(), "positive zero");
    map.insert(neg_zero.clone(), "negative zero");
    assert_eq!(map.len(), 2);
    assert_eq!(map.get(&pos_zero), Some(&"positive zero"));
    assert_eq!(map.get(&neg_zero), Some(&"negative zero"));
}
#[test]
fn test_value_zero_in_arrays() {
    let arr_pos = Value::from([Value::Number(Number::new(0.0f64).unwrap())]);
    let arr_neg = Value::from([Value::Number(Number::new(-0.0f64).unwrap())]);
    assert_ne!(arr_pos, arr_neg);
}
#[test]
fn test_value_zero_in_maps() {
    use jeb_value::String;
    let map_pos: Value = [(
        String::from("zero"),
        Value::Number(Number::new(0.0f64).unwrap()),
    )]
    .into_iter()
    .collect();
    let map_neg: Value = [(
        String::from("zero"),
        Value::Number(Number::new(-0.0f64).unwrap()),
    )]
    .into_iter()
    .collect();
    assert_ne!(map_pos, map_neg);
}
#[test]
// [verify jeb-value.value.traits.ord]
fn test_value_zero_sorted() {
    let mut values = [
        Value::Number(Number::new(0.0f64).unwrap()),
        Value::Number(Number::new(-0.0f64).unwrap()),
    ];
    values.sort();
    // -0.0 < 0.0 in total ordering
    assert_eq!(values[0], Value::Number(Number::new(-0.0f64).unwrap()));
    assert_eq!(values[1], Value::Number(Number::new(0.0f64).unwrap()));
}
