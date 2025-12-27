use {
    jeb_value::{Float, Value},
    std::collections::hash_map::DefaultHasher,
    std::hash::{Hash, Hasher},
};

fn hash_value(v: &Value) -> u64 {
    let mut hasher = DefaultHasher::new();
    v.hash(&mut hasher);
    hasher.finish()
}

#[test]
fn test_unsigned_signed_equality() {
    // Same positive values should be equal
    let u = Value::Unsigned(42);
    let s = Value::Signed(42);
    assert_eq!(u, s, "Unsigned(42) should equal Signed(42)");
    assert_eq!(s, u, "Signed(42) should equal Unsigned(42)");
}

#[test]
fn test_unsigned_signed_hash_consistency() {
    // Same values should have same hash
    let u = Value::Unsigned(42);
    let s = Value::Signed(42);
    assert_eq!(
        hash_value(&u),
        hash_value(&s),
        "Unsigned(42) and Signed(42) should have the same hash"
    );
}

#[test]
fn test_unsigned_signed_ordering() {
    // Same values should compare as Equal
    let u = Value::Unsigned(42);
    let s = Value::Signed(42);
    assert_eq!(u.cmp(&s), std::cmp::Ordering::Equal);
    assert_eq!(s.cmp(&u), std::cmp::Ordering::Equal);
}

#[test]
fn test_negative_signed_vs_unsigned() {
    // Negative signed should be less than any unsigned
    let s_neg = Value::Signed(-5);
    let u = Value::Unsigned(0);
    assert_ne!(s_neg, u);
    assert!(s_neg < u);
}

#[test]
fn test_integer_vs_float_distinct() {
    // Integer and float with same numeric value should NOT be equal
    let u = Value::Unsigned(5);
    let s = Value::Signed(5);
    let f = Value::Float(5.0.try_into().unwrap());

    assert_ne!(u, f, "Unsigned(5) should not equal Float(5.0)");
    assert_ne!(s, f, "Signed(5) should not equal Float(5.0)");
}

#[test]
fn test_integer_vs_float_ordering() {
    // Integers should sort before floats when numerically equal
    let u = Value::Unsigned(5);
    let s = Value::Signed(5);
    let f = Value::Float(5.0.try_into().unwrap());

    assert!(u < f, "Unsigned(5) should be less than Float(5.0)");
    assert!(s < f, "Signed(5) should be less than Float(5.0)");
}

#[test]
fn test_integer_vs_float_hash_distinct() {
    // Integers and floats with same numeric value should have different hashes
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
    // Test ordering with mix of unsigned and signed
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

    // Expected order: -5, 0, 0, 10, 10, 100
    assert_eq!(sorted[0], Value::Signed(-5));
    // 0s should be adjacent (either Signed or Unsigned can be first, they're equal)
    assert!(matches!(sorted[1], Value::Unsigned(0) | Value::Signed(0)));
    assert!(matches!(sorted[2], Value::Unsigned(0) | Value::Signed(0)));
    // 10s should be adjacent
    assert!(matches!(sorted[3], Value::Unsigned(10) | Value::Signed(10)));
    assert!(matches!(sorted[4], Value::Unsigned(10) | Value::Signed(10)));
    assert_eq!(sorted[5], Value::Unsigned(100));
}

#[test]
fn test_integer_float_mixed_ordering() {
    // Test that integers sort before floats at same numeric value
    let mut vals = vec![
        Value::Float(5.0.try_into().unwrap()),
        Value::Unsigned(5),
        Value::Signed(5),
        Value::Float(4.0.try_into().unwrap()),
        Value::Unsigned(6),
    ];

    vals.sort();

    // Expected order: 4.0, 5 (unsigned or signed), 5 (unsigned or signed), 5.0, 6
    match &vals[0] {
        Value::Float(f) if **f == 4.0 => {}
        _ => panic!("Expected Float(4.0) at position 0"),
    }

    // Positions 1 and 2 should be integers with value 5
    assert!(matches!(vals[1], Value::Unsigned(5) | Value::Signed(5)));
    assert!(matches!(vals[2], Value::Unsigned(5) | Value::Signed(5)));

    // Position 3 should be Float(5.0)
    match &vals[3] {
        Value::Float(f) if **f == 5.0 => {}
        _ => panic!("Expected Float(5.0) at position 3, got {:?}", vals[3]),
    }

    // Position 4 should be Unsigned(6)
    assert_eq!(vals[4], Value::Unsigned(6));
}

#[test]
fn test_hashmap_integer_equivalence() {
    use std::collections::HashMap;

    let mut map = HashMap::new();
    map.insert(Value::Unsigned(42), "first");

    // Signed(42) should retrieve the same entry as Unsigned(42)
    assert_eq!(map.get(&Value::Signed(42)), Some(&"first"));

    // Updating with Signed(42) should overwrite the Unsigned(42) entry
    map.insert(Value::Signed(42), "second");
    assert_eq!(map.get(&Value::Unsigned(42)), Some(&"second"));
    assert_eq!(map.len(), 1, "Should still only have one entry");
}

#[test]
fn test_edge_case_stable_sort() {
    // Test with edge cases: -1, -0.0, 0, 0.0 in a non-trivial order
    // Initial order is intentionally scrambled (not sorted, not reversed)
    let mut vals = vec![
        Value::Float(Float::new(0.0).unwrap()),      // 0: Float(0.0)
        Value::Signed(-1),                            // 1: Signed(-1)
        Value::Unsigned(0),                           // 2: Unsigned(0)
        Value::Float(Float::new(-0.0).unwrap()),     // 3: Float(-0.0)
        Value::Signed(0),                             // 4: Signed(0)
        Value::Float(Float::new(-1.0).unwrap()),     // 5: Float(-1.0)
    ];

    vals.sort();

    // Print sorted order for debugging
    println!("Sorted order:");
    for (i, v) in vals.iter().enumerate() {
        println!("  {}: {:?}", i, v);
    }

    // Expected order after stable sort:
    // Based on type_rank and comparison logic:
    // 1. Signed(-1) and Float(-1.0) - need to check which comes first
    // 2-3. Unsigned(0) and Signed(0) in original order (2, 4) - they're equal
    // 4-5. Float(-0.0) and Float(0.0) - by total_cmp, -0.0 < 0.0

    // Let's verify the actual behavior
    // Signed(-1) vs Float(-1.0): both have type_rank 2, so goes to cross-type comparison
    // Signed(-1) vs Float(-1.0) should make Signed < Float (integer before float at same numeric value)
    assert_eq!(vals[0], Value::Signed(-1), "Expected Signed(-1) at position 0");

    match &vals[1] {
        Value::Float(f) if **f == -1.0 => {}
        _ => panic!("Expected Float(-1.0) at position 1, got {:?}", vals[1]),
    }

    // Positions 2-3 should be the integer zeros in their original relative order
    // Original order: Unsigned(0) was at index 2, Signed(0) was at index 4
    // Since stable sort preserves order of equal elements, Unsigned(0) should come first
    assert_eq!(vals[2], Value::Unsigned(0), "Expected Unsigned(0) at position 2");
    assert_eq!(vals[3], Value::Signed(0), "Expected Signed(0) at position 3");

    // Positions 4-5 should be the float zeros
    // By total_cmp, -0.0 < 0.0, so Float(-0.0) should come before Float(0.0)
    match &vals[4] {
        Value::Float(f) if f.is_sign_negative() && **f == 0.0 => {} // -0.0
        _ => panic!("Expected Float(-0.0) at position 4, got {:?}", vals[4]),
    }
    match &vals[5] {
        Value::Float(f) if f.is_sign_positive() && **f == 0.0 => {} // +0.0
        _ => panic!("Expected Float(0.0) at position 5, got {:?}", vals[5]),
    }
}

#[test]
fn test_negative_zero_float_ordering() {
    // Test that -0.0 and 0.0 are ordered correctly by total_cmp
    let neg_zero = Value::Float(Float::new(-0.0).unwrap());
    let pos_zero = Value::Float(Float::new(0.0).unwrap());

    // According to IEEE 754 total_cmp, -0.0 < 0.0
    assert!(
        neg_zero < pos_zero,
        "-0.0 should be less than 0.0 in total ordering"
    );
}

#[test]
fn test_stable_sort_order_preservation() {
    // Test that stable sort preserves relative order of equal elements
    // We'll use two different initial orders and verify they produce different results
    // for equal elements while maintaining the same overall ordering

    // Order A: Unsigned before Signed for zeros
    let mut vals_a = vec![
        Value::Signed(-1),
        Value::Unsigned(0),  // First zero
        Value::Signed(0),    // Second zero
        Value::Float(Float::new(5.0).unwrap()),
    ];

    // Order B: Signed before Unsigned for zeros (reversed)
    let mut vals_b = vec![
        Value::Signed(-1),
        Value::Signed(0),    // First zero
        Value::Unsigned(0),  // Second zero
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

    // Both should have same overall structure
    assert_eq!(vals_a[0], Value::Signed(-1));
    assert_eq!(vals_b[0], Value::Signed(-1));

    // Order A should have Unsigned(0) before Signed(0) (original order preserved)
    assert_eq!(vals_a[1], Value::Unsigned(0), "Order A: Unsigned(0) should come first");
    assert_eq!(vals_a[2], Value::Signed(0), "Order A: Signed(0) should come second");

    // Order B should have Signed(0) before Unsigned(0) (original order preserved)
    assert_eq!(vals_b[1], Value::Signed(0), "Order B: Signed(0) should come first");
    assert_eq!(vals_b[2], Value::Unsigned(0), "Order B: Unsigned(0) should come second");

    // Both should have Float(5.0) at the end
    match &vals_a[3] {
        Value::Float(f) if **f == 5.0 => {}
        _ => panic!("Order A: Expected Float(5.0) at position 3"),
    }
    match &vals_b[3] {
        Value::Float(f) if **f == 5.0 => {}
        _ => panic!("Order B: Expected Float(5.0) at position 3"),
    }

    // The two sorted arrays should be different due to stable sort
    // However, since Unsigned(0) == Signed(0) by our PartialEq implementation,
    // the vectors will compare as equal even though they have different types at positions 1-2
    // Let's verify they're structurally different by comparing discriminants
    assert!(
        !matches!((&vals_a[1], &vals_b[1]),
                  (Value::Unsigned(_), Value::Unsigned(_)) | (Value::Signed(_), Value::Signed(_))),
        "Stable sort should preserve different initial orders - position 1 should have different types"
    );
}

#[test]
fn test_stable_sort_with_floats() {
    // Test stable sort with -0.0 and 0.0 in different orders

    // Order A: 0.0 before -0.0
    let mut vals_a = vec![
        Value::Float(Float::new(0.0).unwrap()),
        Value::Float(Float::new(-0.0).unwrap()),
    ];

    // Order B: -0.0 before 0.0
    let mut vals_b = vec![
        Value::Float(Float::new(-0.0).unwrap()),
        Value::Float(Float::new(0.0).unwrap()),
    ];

    vals_a.sort();
    vals_b.sort();

    // Both should have -0.0 before 0.0 after sort (by total_cmp, -0.0 < 0.0)
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

    // Both should end up with the same order since -0.0 < 0.0 by total_cmp
    // (not equal, so stable sort doesn't matter here)
}
