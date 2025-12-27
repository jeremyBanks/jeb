/// Tests to verify that positive and negative zero are properly distinguished
/// in Float and Value types for equality, ordering, and hashing.
use {
    jeb_values::{
        Float,
        Value,
    },
    std::collections::{
        HashMap,
        hash_map::DefaultHasher,
    },
    std::hash::{
        Hash,
        Hasher,
    },
};

// ============================================================================
// Float tests
// ============================================================================

#[test]
fn test_float_zero_equality() {
    let pos_zero = Float::try_from(0.0f64).unwrap();
    let neg_zero = Float::try_from(-0.0f64).unwrap();

    // Positive and negative zero should NOT be equal
    assert_ne!(pos_zero, neg_zero);
    assert_ne!(neg_zero, pos_zero);
}

#[test]
fn test_float_zero_ordering() {
    let pos_zero = Float::try_from(0.0f64).unwrap();
    let neg_zero = Float::try_from(-0.0f64).unwrap();

    // Negative zero should be less than positive zero
    assert!(neg_zero < pos_zero);
    assert!(pos_zero > neg_zero);

    // Verify via cmp
    use std::cmp::Ordering;
    assert_eq!(neg_zero.cmp(&pos_zero), Ordering::Less);
    assert_eq!(pos_zero.cmp(&neg_zero), Ordering::Greater);
}

#[test]
fn test_float_zero_hashing() {
    let pos_zero = Float::try_from(0.0f64).unwrap();
    let neg_zero = Float::try_from(-0.0f64).unwrap();

    let mut hasher1 = DefaultHasher::new();
    let mut hasher2 = DefaultHasher::new();

    pos_zero.hash(&mut hasher1);
    neg_zero.hash(&mut hasher2);

    // Positive and negative zero should have different hashes
    assert_ne!(hasher1.finish(), hasher2.finish());
}

#[test]
fn test_float_zero_as_hashmap_key() {
    let mut map = HashMap::new();

    let pos_zero = Float::try_from(0.0f64).unwrap();
    let neg_zero = Float::try_from(-0.0f64).unwrap();

    map.insert(pos_zero, "positive zero");
    map.insert(neg_zero, "negative zero");

    // Should have two distinct entries
    assert_eq!(map.len(), 2);
    assert_eq!(map.get(&pos_zero), Some(&"positive zero"));
    assert_eq!(map.get(&neg_zero), Some(&"negative zero"));
}

#[test]
fn test_float_zero_bit_representation() {
    let pos_zero = Float::try_from(0.0f64).unwrap();
    let neg_zero = Float::try_from(-0.0f64).unwrap();

    // Verify the underlying f64 values have different bit patterns
    let pos_bits = (*pos_zero).to_bits();
    let neg_bits = (*neg_zero).to_bits();

    assert_ne!(pos_bits, neg_bits);
    assert_eq!(pos_bits, 0x0000_0000_0000_0000u64); // +0.0
    assert_eq!(neg_bits, 0x8000_0000_0000_0000u64); // -0.0
}

// ============================================================================
// Value tests
// ============================================================================

#[test]
fn test_value_zero_equality() {
    let pos_zero = Value::Float(Float::try_from(0.0f64).unwrap());
    let neg_zero = Value::Float(Float::try_from(-0.0f64).unwrap());

    // Positive and negative zero should NOT be equal
    assert_ne!(pos_zero, neg_zero);
    assert_ne!(neg_zero, pos_zero);
}

#[test]
fn test_value_zero_ordering() {
    let pos_zero = Value::Float(Float::try_from(0.0f64).unwrap());
    let neg_zero = Value::Float(Float::try_from(-0.0f64).unwrap());

    // Negative zero should be less than positive zero
    assert!(neg_zero < pos_zero);
    assert!(pos_zero > neg_zero);

    // Verify via cmp
    use std::cmp::Ordering;
    assert_eq!(neg_zero.cmp(&pos_zero), Ordering::Less);
    assert_eq!(pos_zero.cmp(&neg_zero), Ordering::Greater);
}

#[test]
fn test_value_zero_hashing() {
    let pos_zero = Value::Float(Float::try_from(0.0f64).unwrap());
    let neg_zero = Value::Float(Float::try_from(-0.0f64).unwrap());

    let mut hasher1 = DefaultHasher::new();
    let mut hasher2 = DefaultHasher::new();

    pos_zero.hash(&mut hasher1);
    neg_zero.hash(&mut hasher2);

    // Positive and negative zero should have different hashes
    assert_ne!(hasher1.finish(), hasher2.finish());
}

#[test]
fn test_value_zero_as_hashmap_key() {
    let mut map = HashMap::new();

    let pos_zero = Value::Float(Float::try_from(0.0f64).unwrap());
    let neg_zero = Value::Float(Float::try_from(-0.0f64).unwrap());

    map.insert(pos_zero.clone(), "positive zero");
    map.insert(neg_zero.clone(), "negative zero");

    // Should have two distinct entries
    assert_eq!(map.len(), 2);
    assert_eq!(map.get(&pos_zero), Some(&"positive zero"));
    assert_eq!(map.get(&neg_zero), Some(&"negative zero"));
}

#[test]
fn test_value_zero_in_arrays() {
    let arr_pos = Value::from([Value::Float(Float::try_from(0.0f64).unwrap())]);
    let arr_neg = Value::from([Value::Float(Float::try_from(-0.0f64).unwrap())]);

    // Arrays containing different zeros should not be equal
    assert_ne!(arr_pos, arr_neg);
}

#[test]
fn test_value_zero_in_maps() {
    use jeb_values::Text;

    let map_pos: Value = [(
        Text::from("zero"),
        Value::Float(Float::try_from(0.0f64).unwrap()),
    )]
    .into_iter()
    .collect();

    let map_neg: Value = [(
        Text::from("zero"),
        Value::Float(Float::try_from(-0.0f64).unwrap()),
    )]
    .into_iter()
    .collect();

    // Maps containing different zeros should not be equal
    assert_ne!(map_pos, map_neg);
}

#[test]
fn test_value_zero_cross_type_comparison() {
    use std::cmp::Ordering;

    let pos_zero_float = Value::Float(Float::try_from(0.0f64).unwrap());
    let neg_zero_float = Value::Float(Float::try_from(-0.0f64).unwrap());
    let unsigned_zero = Value::from(0u64);
    let signed_zero = Value::from(0i64);

    // Float zeros vs integer zeros
    // When comparing integer 0 to float -0.0 or +0.0, we convert integer to f64
    // (which gives +0.0) Then use total_cmp: -0.0 < +0.0, so integer 0 (+0.0) >
    // float -0.0

    // unsigned_zero (0u64 -> 0.0) vs neg_zero_float (-0.0)
    // Using total_cmp: 0.0 > -0.0, so unsigned is Greater
    assert_eq!(unsigned_zero.cmp(&neg_zero_float), Ordering::Greater);

    // unsigned_zero (0u64 -> 0.0) vs pos_zero_float (0.0)
    // Using total_cmp: 0.0 == 0.0, then use type tiebreaker: Unsigned < Float
    assert_eq!(unsigned_zero.cmp(&pos_zero_float), Ordering::Less);

    // neg_zero_float (-0.0) and pos_zero_float (0.0) themselves are ordered
    assert_eq!(neg_zero_float.cmp(&pos_zero_float), Ordering::Less);

    // signed_zero (0i64 -> 0.0) vs neg_zero_float (-0.0)
    // Using total_cmp: 0.0 > -0.0, so signed is Greater
    assert_eq!(signed_zero.cmp(&neg_zero_float), Ordering::Greater);

    // signed_zero (0i64 -> 0.0) vs pos_zero_float (0.0)
    // Using total_cmp: 0.0 == 0.0, then use type tiebreaker: Signed < Float
    assert_eq!(signed_zero.cmp(&pos_zero_float), Ordering::Less);
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

    // Expected order: -0.0, 0u64, 0i64, 0.0
    // Reasoning:
    // - All numbers are in the same type rank (2)
    // - -0.0 < 0u64 because when comparing, 0u64 converts to +0.0, and -0.0 < +0.0
    // - 0u64 < 0i64 because when both compare as +0.0, we use type tiebreaker:
    //   Unsigned < Signed
    // - 0i64 < +0.0 because when both compare as +0.0, we use type tiebreaker:
    //   Signed < Float
    assert_eq!(values[0], Value::Float(Float::try_from(-0.0f64).unwrap()));
    assert_eq!(values[1], Value::from(0u64));
    assert_eq!(values[2], Value::from(0i64));
    assert_eq!(values[3], Value::Float(Float::try_from(0.0f64).unwrap()));
}
