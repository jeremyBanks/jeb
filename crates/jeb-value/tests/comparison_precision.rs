use {jeb_value::Value, std::cmp::Ordering};
/// Test the precision fix for comparing large integers with floats.
/// Previously, casting u64 to f64 lost precision for values > 2^53.
#[test]
fn test_unsigned_float_precision() {
    let max_safe = 1u64 << 53;
    let u1 = Value::Unsigned(max_safe + 1);
    let f1 = Value::Float((max_safe as f64).try_into().unwrap());
    assert_eq!(u1.cmp(& f1), Ordering::Greater);
    assert_eq!(f1.cmp(& u1), Ordering::Less);
}
#[test]
fn test_unsigned_float_fractional() {
    let u = Value::Unsigned(100);
    let f = Value::Float(99.5.try_into().unwrap());
    assert_eq!(u.cmp(& f), Ordering::Greater);
    assert_eq!(f.cmp(& u), Ordering::Less);
    let u2 = Value::Unsigned(100);
    let f2 = Value::Float(100.5.try_into().unwrap());
    assert_eq!(u2.cmp(& f2), Ordering::Less);
    assert_eq!(f2.cmp(& u2), Ordering::Greater);
}
#[test]
fn test_unsigned_float_negative() {
    let u = Value::Unsigned(0);
    let f = Value::Float((-1.0).try_into().unwrap());
    assert_eq!(u.cmp(& f), Ordering::Greater);
    assert_eq!(f.cmp(& u), Ordering::Less);
}
#[test]
fn test_unsigned_float_large() {
    let u = Value::Unsigned(u64::MAX);
    let f = Value::Float(1e20.try_into().unwrap());
    assert_eq!(u.cmp(& f), Ordering::Less);
    assert_eq!(f.cmp(& u), Ordering::Greater);
}
#[test]
fn test_signed_float_precision() {
    let max_safe = 1i64 << 53;
    let i1 = Value::Signed(max_safe + 1);
    let f1 = Value::Float((max_safe as f64).try_into().unwrap());
    assert_eq!(i1.cmp(& f1), Ordering::Greater);
    assert_eq!(f1.cmp(& i1), Ordering::Less);
}
#[test]
fn test_signed_float_negative() {
    let max_safe = 1i64 << 53;
    let i1 = Value::Signed(-(max_safe + 1));
    let f1 = Value::Float((-(max_safe as f64)).try_into().unwrap());
    assert_eq!(i1.cmp(& f1), Ordering::Less);
    assert_eq!(f1.cmp(& i1), Ordering::Greater);
}
#[test]
fn test_transitivity() {
    let max_safe = 1u64 << 53;
    let a = Value::Unsigned(max_safe + 1);
    let b = Value::Float((max_safe as f64).try_into().unwrap());
    let c = Value::Unsigned(max_safe);
    assert_eq!(a.cmp(& b), Ordering::Greater);
    assert_eq!(b.cmp(& c), Ordering::Greater);
    assert_eq!(a.cmp(& c), Ordering::Greater);
}
#[test]
fn test_exact_equality() {
    let u = Value::Unsigned(42);
    let f = Value::Float(42.0.try_into().unwrap());
    assert_eq!(u.cmp(& f), Ordering::Less);
    assert_eq!(f.cmp(& u), Ordering::Greater);
}
#[test]
fn test_signed_float_negative_fractional() {
    let i1 = Value::Signed(-5);
    let f1 = Value::Float((-4.5).try_into().unwrap());
    assert_eq!(i1.cmp(& f1), Ordering::Less);
    assert_eq!(f1.cmp(& i1), Ordering::Greater);
    let i2 = Value::Signed(-5);
    let f2 = Value::Float((-5.5).try_into().unwrap());
    assert_eq!(i2.cmp(& f2), Ordering::Greater);
    assert_eq!(f2.cmp(& i2), Ordering::Less);
    let i3 = Value::Signed(-6);
    let f3 = Value::Float((-5.5).try_into().unwrap());
    assert_eq!(i3.cmp(& f3), Ordering::Less);
    assert_eq!(f3.cmp(& i3), Ordering::Greater);
}
