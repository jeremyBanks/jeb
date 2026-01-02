use {
    jeb_value::{
        Float,
        Value,
    },
    std::{
        collections::hash_map::DefaultHasher,
        hash::{
            Hash,
            Hasher,
        },
    },
};
fn hash_value(v: &Value) -> u64 {
    let mut hasher = DefaultHasher::new();
    v.hash(&mut hasher);
    hasher.finish()
}
#[test]
// [verify jeb-value.value.cmp]
fn test_unsigned_signed_equality() {
    let u = Value::Unsigned(42);
    let s = Value::Signed(42);
    assert_eq!(u, s, "Unsigned(42) should equal Signed(42)");
    assert_eq!(s, u, "Signed(42) should equal Unsigned(42)");
}
// [verify jeb-value.value.cmp]
#[test]
fn test_unsigned_signed_hash_consistency() {
    let u = Value::Unsigned(42);
    let s = Value::Signed(42);
    assert_eq!(
        hash_value(&u),
        hash_value(&s),
        "Unsigned(42) and Signed(42) should have the same hash"
    );
// [verify jeb-value.value.cmp]
}
#[test]
fn test_unsigned_signed_ordering() {
    let u = Value::Unsigned(42);
    let s = Value::Signed(42);
    assert_eq!(u.cmp(&s), std::cmp::Ordering::Equal);
    assert_eq!(s.cmp(&u), std::cmp::Ordering::Equal);
}
#[test]
fn test_negative_signed_vs_unsigned() {
    let s_neg = Value::Signed(-5);
    let u = Value::Unsigned(0);
    assert_ne!(s_neg, u);
// [verify jeb-value.value.cmp]
    assert!(s_neg < u);
}
#[test]
fn test_integer_vs_float_distinct() {
    let u = Value::Unsigned(5);
    let s = Value::Signed(5);
    let f = Value::Float(5.0.try_into().unwrap());
// [verify jeb-value.value.cmp]
    assert_ne!(u, f, "Unsigned(5) should not equal Float(5.0)");
    assert_ne!(s, f, "Signed(5) should not equal Float(5.0)");
}
#[test]
fn test_integer_vs_float_ordering() {
    let u = Value::Unsigned(5);
    let s = Value::Signed(5);
// [verify jeb-value.value.cmp]
    let f = Value::Float(5.0.try_into().unwrap());
    assert!(u < f, "Unsigned(5) should be less than Float(5.0)");
    assert!(s < f, "Signed(5) should be less than Float(5.0)");
}
#[test]
fn test_integer_vs_float_hash_distinct() {
    let u = Value::Unsigned(5);
    let f = Value::Float(5.0.try_into().unwrap());
    assert_ne!(
        hash_value(&u),
        hash_value(&f),
        "Unsigned(5) and Float(5.0) should have different hashes"
    );
}
#[test]
fn test_mixed_integer_ordering() {
    let vals = vec![
        Value::Unsigned(10),
        Value::Signed(-5),
        Value::Signed(10),
        Value::Unsigned(0),
        Value::Signed(0),
        Value::Unsigned(100),
    ];
    let mut sorted = vals.clone();
    sorted.sort();
    assert_eq!(sorted[0], Value::Signed(-5));
    assert!(matches!(sorted[1], Value::Unsigned(0) | Value::Signed(0)));
    assert!(matches!(sorted[2], Value::Unsigned(0) | Value::Signed(0)));
    assert!(matches!(sorted[3], Value::Unsigned(10) | Value::Signed(10)));
    assert!(matches!(sorted[4], Value::Unsigned(10) | Value::Signed(10)));
    assert_eq!(sorted[5], Value::Unsigned(100));
}
#[test]
fn test_integer_float_mixed_ordering() {
    let mut vals = vec![
        Value::Float(5.0.try_into().unwrap()),
        Value::Unsigned(5),
        Value::Signed(5),
        Value::Float(4.0.try_into().unwrap()),
        Value::Unsigned(6),
    ];
    vals.sort();
    match &vals[0] {
        Value::Float(f) if **f == 4.0 => {}
        _ => panic!("Expected Float(4.0) at position 0"),
    }
    assert!(matches!(vals[1], Value::Unsigned(5) | Value::Signed(5)));
    assert!(matches!(vals[2], Value::Unsigned(5) | Value::Signed(5)));
    match &vals[3] {
        Value::Float(f) if **f == 5.0 => {}
// [verify jeb-value.value.cmp]
        _ => panic!("Expected Float(5.0) at position 3, got {:?}", vals[3]),
    }
    assert_eq!(vals[4], Value::Unsigned(6));
}
#[test]
fn test_hashmap_integer_equivalence() {
    use std::collections::HashMap;
    let mut map = HashMap::new();
    map.insert(Value::Unsigned(42), "first");
    assert_eq!(map.get(&Value::Signed(42)), Some(&"first"));
    map.insert(Value::Signed(42), "second");
    assert_eq!(map.get(&Value::Unsigned(42)), Some(&"second"));
    assert_eq!(map.len(), 1, "Should still only have one entry");
}
#[test]
fn test_edge_case_stable_sort() {
    let mut vals = vec![
        Value::Float(Float::new(0.0).unwrap()),
        Value::Signed(-1),
        Value::Unsigned(0),
        Value::Float(Float::new(-0.0).unwrap()),
        Value::Signed(0),
        Value::Float(Float::new(-1.0).unwrap()),
    ];
    vals.sort();
    println!("Sorted order:");
    for (i, v) in vals.iter().enumerate() {
        println!("  {}: {:?}", i, v);
    }
    assert_eq!(
        vals[0],
        Value::Signed(-1),
        "Expected Signed(-1) at position 0"
    );
    match &vals[1] {
        Value::Float(f) if **f == -1.0 => {}
        _ => panic!("Expected Float(-1.0) at position 1, got {:?}", vals[1]),
    }
    assert_eq!(
        vals[2],
        Value::Unsigned(0),
        "Expected Unsigned(0) at position 2"
    );
    assert_eq!(
        vals[3],
        Value::Signed(0),
        "Expected Signed(0) at position 3"
    );
    match &vals[4] {
        Value::Float(f) if f.is_sign_negative() && **f == 0.0 => {}
        _ => panic!("Expected Float(-0.0) at position 4, got {:?}", vals[4]),
// [verify jeb-value.number.cmp]
    }
    match &vals[5] {
        Value::Float(f) if f.is_sign_positive() && **f == 0.0 => {}
        _ => panic!("Expected Float(0.0) at position 5, got {:?}", vals[5]),
    }
}
#[test]
fn test_negative_zero_float_ordering() {
    let neg_zero = Value::Float(Float::new(-0.0).unwrap());
    let pos_zero = Value::Float(Float::new(0.0).unwrap());
    assert!(
        neg_zero < pos_zero,
        "-0.0 should be less than 0.0 in total ordering"
    );
}
#[test]
fn test_stable_sort_order_preservation() {
    let mut vals_a = vec![
        Value::Signed(-1),
        Value::Unsigned(0),
        Value::Signed(0),
        Value::Float(Float::new(5.0).unwrap()),
    ];
    let mut vals_b = vec![
        Value::Signed(-1),
        Value::Signed(0),
        Value::Unsigned(0),
        Value::Float(Float::new(5.0).unwrap()),
    ];
    vals_a.sort();
    vals_b.sort();
    println!("\nOrder A after sort:");
    for (i, v) in vals_a.iter().enumerate() {
        println!("  {}: {:?}", i, v);
    }
    println!("\nOrder B after sort:");
    for (i, v) in vals_b.iter().enumerate() {
        println!("  {}: {:?}", i, v);
    }
    assert_eq!(vals_a[0], Value::Signed(-1));
    assert_eq!(vals_b[0], Value::Signed(-1));
    assert_eq!(
        vals_a[1],
        Value::Unsigned(0),
        "Order A: Unsigned(0) should come first"
    );
    assert_eq!(
        vals_a[2],
        Value::Signed(0),
        "Order A: Signed(0) should come second"
    );
    assert_eq!(
        vals_b[1],
        Value::Signed(0),
        "Order B: Signed(0) should come first"
    );
    assert_eq!(
        vals_b[2],
        Value::Unsigned(0),
        "Order B: Unsigned(0) should come second"
    );
    match &vals_a[3] {
        Value::Float(f) if **f == 5.0 => {}
        _ => panic!("Order A: Expected Float(5.0) at position 3"),
    }
    match &vals_b[3] {
        Value::Float(f) if **f == 5.0 => {}
        _ => panic!("Order B: Expected Float(5.0) at position 3"),
    }
    assert!(
        !matches!(
            (&vals_a[1], &vals_b[1]),
            (Value::Unsigned(_), Value::Unsigned(_)) | (Value::Signed(_), Value::Signed(_))
        ),
        "Stable sort should preserve different initial orders - position 1 should have different \
         types"
    );
}
#[test]
fn test_stable_sort_with_floats() {
    let mut vals_a = vec![
        Value::Float(Float::new(0.0).unwrap()),
        Value::Float(Float::new(-0.0).unwrap()),
    ];
    let mut vals_b = vec![
        Value::Float(Float::new(-0.0).unwrap()),
        Value::Float(Float::new(0.0).unwrap()),
    ];
    vals_a.sort();
    vals_b.sort();
    match &vals_a[0] {
        Value::Float(f) if f.is_sign_negative() && **f == 0.0 => {}
        _ => panic!("Expected -0.0 at position 0 in order A"),
    }
    match &vals_a[1] {
        Value::Float(f) if f.is_sign_positive() && **f == 0.0 => {}
        _ => panic!("Expected 0.0 at position 1 in order A"),
    }
    match &vals_b[0] {
        Value::Float(f) if f.is_sign_negative() && **f == 0.0 => {}
        _ => panic!("Expected -0.0 at position 0 in order B"),
    }
    match &vals_b[1] {
        Value::Float(f) if f.is_sign_positive() && **f == 0.0 => {}
        _ => panic!("Expected 0.0 at position 1 in order B"),
    }
}
