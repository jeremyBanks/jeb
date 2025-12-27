use jeb_value::Value;
use std::cmp::Ordering;

/// Test the precision fix for comparing large integers with floats.
/// Previously, casting u64 to f64 lost precision for values > 2^53.
#[test]
fn test_unsigned_float_precision() {
    let max_safe = 1u64 << 53; // 9007199254740992 (2^53)

    // The critical bug case: u64 value beyond f64's exact integer range
    let u1 = Value::Unsigned(max_safe + 1);
    let f1 = Value::Float((max_safe as f64).try_into().unwrap());

    // Should be Greater (9007199254740993 > 9007199254740992.0)
    assert_eq!(u1.cmp(&f1), Ordering::Greater);
    assert_eq!(f1.cmp(&u1), Ordering::Less);
}

#[test]
fn test_unsigned_float_fractional() {
    // Integer vs fractional float
    let u = Value::Unsigned(100);
    let f = Value::Float(99.5.try_into().unwrap());

    assert_eq!(u.cmp(&f), Ordering::Greater);
    assert_eq!(f.cmp(&u), Ordering::Less);

    // Equal integer parts, but float has fraction
    let u2 = Value::Unsigned(100);
    let f2 = Value::Float(100.5.try_into().unwrap());

    assert_eq!(u2.cmp(&f2), Ordering::Less);
    assert_eq!(f2.cmp(&u2), Ordering::Greater);
}

#[test]
fn test_unsigned_float_negative() {
    let u = Value::Unsigned(0);
    let f = Value::Float((-1.0).try_into().unwrap());

    // u64 >= 0, so always greater than negative float
    assert_eq!(u.cmp(&f), Ordering::Greater);
    assert_eq!(f.cmp(&u), Ordering::Less);
}

#[test]
fn test_unsigned_float_large() {
    let u = Value::Unsigned(u64::MAX);
    let f = Value::Float(1e20.try_into().unwrap()); // Much larger than u64::MAX

    assert_eq!(u.cmp(&f), Ordering::Less);
    assert_eq!(f.cmp(&u), Ordering::Greater);
}

#[test]
fn test_signed_float_precision() {
    let max_safe = 1i64 << 53;

    let i1 = Value::Signed(max_safe + 1);
    let f1 = Value::Float((max_safe as f64).try_into().unwrap());

    assert_eq!(i1.cmp(&f1), Ordering::Greater);
    assert_eq!(f1.cmp(&i1), Ordering::Less);
}

#[test]
fn test_signed_float_negative() {
    let max_safe = 1i64 << 53;

    let i1 = Value::Signed(-(max_safe + 1));
    let f1 = Value::Float((-(max_safe as f64)).try_into().unwrap());

    assert_eq!(i1.cmp(&f1), Ordering::Less);
    assert_eq!(f1.cmp(&i1), Ordering::Greater);
}

#[test]
fn test_transitivity() {
    // Test that a > b and b > c implies a > c
    let max_safe = 1u64 << 53;

    let a = Value::Unsigned(max_safe + 1); // 2^53 + 1
    let b = Value::Float((max_safe as f64).try_into().unwrap()); // 2^53
    let c = Value::Unsigned(max_safe); // 2^53

    assert_eq!(a.cmp(&b), Ordering::Greater); // a > b
    assert_eq!(b.cmp(&c), Ordering::Greater); // b > c (tiebreaker)
    assert_eq!(a.cmp(&c), Ordering::Greater); // a > c (transitivity holds)
}

#[test]
fn test_exact_equality() {
    let u = Value::Unsigned(42);
    let f = Value::Float(42.0.try_into().unwrap());

    // Tiebreaker: Unsigned < Float when mathematically equal
    assert_eq!(u.cmp(&f), Ordering::Less);
    assert_eq!(f.cmp(&u), Ordering::Greater);
}

#[test]
fn test_signed_float_negative_fractional() {
    // Test negative float with fractional part
    // -5 vs -4.5: -5 < -4.5
    let i1 = Value::Signed(-5);
    let f1 = Value::Float((-4.5).try_into().unwrap());
    assert_eq!(i1.cmp(&f1), Ordering::Less);
    assert_eq!(f1.cmp(&i1), Ordering::Greater);

    // Test negative float with fractional part, integer parts equal
    // -5 vs -5.5: -5 > -5.5
    let i2 = Value::Signed(-5);
    let f2 = Value::Float((-5.5).try_into().unwrap());
    assert_eq!(i2.cmp(&f2), Ordering::Greater);
    assert_eq!(f2.cmp(&i2), Ordering::Less);

    // Test negative float with positive fractional (closer to zero)
    // -6 vs -5.5: -6 < -5.5
    let i3 = Value::Signed(-6);
    let f3 = Value::Float((-5.5).try_into().unwrap());
    assert_eq!(i3.cmp(&f3), Ordering::Less);
    assert_eq!(f3.cmp(&i3), Ordering::Greater);
}
