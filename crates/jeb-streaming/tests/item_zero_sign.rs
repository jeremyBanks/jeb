/// Tests to verify that positive and negative zero are properly distinguished
/// in Item type for equality, ordering, and hashing (via contained Value).
use {
    jeb_streaming::Item,
    jeb_value::{
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
    let mut items = [
        Item::Value(Value::Float(Float::try_from(0.0f64).unwrap())),
        Item::Value(Value::Float(Float::try_from(-0.0f64).unwrap())),
        Item::Value(Value::from(0u64)),
        Item::Value(Value::from(0i64)),
    ];

    items.sort();

    println!("Sorted order:");
    for (i, item) in items.iter().enumerate() {
        println!("  {}: {:?}", i, item);
    }

    // Expected order based on Value ordering:
    // Integers sort before floats at same numeric value, so:
    // 1-2. Unsigned(0) and Signed(0) - equal, stable sort preserves original order
    // 3. Float(-0.0) - negative zero float (after integers, before positive zero)
    // 4. Float(0.0) - positive zero float

    // Items[0] and items[1] should be the integer zeros
    // Since Unsigned(0) == Signed(0), stable sort preserves their original order
    // Original order: Float(0.0), Float(-0.0), 0u64, 0i64
    // So 0u64 comes before 0i64 in the sorted result
    assert_eq!(items[0], Item::Value(Value::from(0u64)), "Unsigned(0) should be first");
    assert_eq!(items[1], Item::Value(Value::from(0i64)), "Signed(0) should be second");

    assert_eq!(
        items[2],
        Item::Value(Value::Float(Float::try_from(-0.0f64).unwrap())),
        "Float(-0.0) should be third"
    );

    assert_eq!(
        items[3],
        Item::Value(Value::Float(Float::try_from(0.0f64).unwrap())),
        "Float(0.0) should be last"
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
#[cfg(test)] mod tests { include!("/tmp/test_value_order.rs"); }
