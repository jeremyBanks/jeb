/// Tests to verify that positive and negative zero are properly distinguished
/// in Item type for equality, ordering, and hashing (via contained Value).
use {
    jeb_node::Item,
    jeb_values::{Float, Value},
    std::collections::{hash_map::DefaultHasher, HashMap},
    std::hash::{Hash, Hasher},
};

#[test]
fn test_item_zero_equality() {
    let pos_zero = Item::Value(Value::Float(Float::try_from(0.0f64).unwrap()));
    let neg_zero = Item::Value(Value::Float(Float::try_from(-0.0f64).unwrap()));

    // Positive and negative zero should NOT be equal
    assert_ne!(pos_zero, neg_zero);
    assert_ne!(neg_zero, pos_zero);
}

#[test]
fn test_item_zero_ordering() {
    let pos_zero = Item::Value(Value::Float(Float::try_from(0.0f64).unwrap()));
    let neg_zero = Item::Value(Value::Float(Float::try_from(-0.0f64).unwrap()));

    // Negative zero should be less than positive zero
    assert!(neg_zero < pos_zero);
    assert!(pos_zero > neg_zero);

    // Verify via cmp
    use std::cmp::Ordering;
    assert_eq!(neg_zero.cmp(&pos_zero), Ordering::Less);
    assert_eq!(pos_zero.cmp(&neg_zero), Ordering::Greater);
}

#[test]
fn test_item_zero_hashing() {
    let pos_zero = Item::Value(Value::Float(Float::try_from(0.0f64).unwrap()));
    let neg_zero = Item::Value(Value::Float(Float::try_from(-0.0f64).unwrap()));

    let mut hasher1 = DefaultHasher::new();
    let mut hasher2 = DefaultHasher::new();

    pos_zero.hash(&mut hasher1);
    neg_zero.hash(&mut hasher2);

    // Positive and negative zero should have different hashes
    assert_ne!(hasher1.finish(), hasher2.finish());
}

#[test]
fn test_item_zero_as_hashmap_key() {
    let mut map = HashMap::new();

    let pos_zero = Item::Value(Value::Float(Float::try_from(0.0f64).unwrap()));
    let neg_zero = Item::Value(Value::Float(Float::try_from(-0.0f64).unwrap()));

    map.insert(pos_zero.clone(), "positive zero");
    map.insert(neg_zero.clone(), "negative zero");

    // Should have two distinct entries
    assert_eq!(map.len(), 2);
    assert_eq!(map.get(&pos_zero), Some(&"positive zero"));
    assert_eq!(map.get(&neg_zero), Some(&"negative zero"));
}

#[test]
fn test_item_zero_sorted() {
    let mut items = vec![
        Item::Value(Value::Float(Float::try_from(0.0f64).unwrap())),
        Item::Value(Value::Float(Float::try_from(-0.0f64).unwrap())),
        Item::Value(Value::from(0u64)),
        Item::Value(Value::from(0i64)),
    ];

    items.sort();

    // Expected order based on Value ordering: -0.0, 0u64, 0i64, 0.0
    assert_eq!(
        items[0],
        Item::Value(Value::Float(Float::try_from(-0.0f64).unwrap()))
    );
    assert_eq!(items[1], Item::Value(Value::from(0u64)));
    assert_eq!(items[2], Item::Value(Value::from(0i64)));
    assert_eq!(
        items[3],
        Item::Value(Value::Float(Float::try_from(0.0f64).unwrap()))
    );
}

#[test]
fn test_item_type_ordering() {
    use std::cmp::Ordering;

    // Item ordering: Bytes < Text < Value
    let bytes_item = Item::Bytes(vec![0u8].into());
    let text_item = Item::Text("0".into());
    let value_item = Item::Value(Value::Float(Float::try_from(0.0f64).unwrap()));

    assert_eq!(bytes_item.cmp(&text_item), Ordering::Less);
    assert_eq!(text_item.cmp(&value_item), Ordering::Less);
    assert_eq!(bytes_item.cmp(&value_item), Ordering::Less);
}
