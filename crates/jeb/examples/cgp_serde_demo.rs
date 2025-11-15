// CGP-Serde demonstration example
//
// This example shows how to use the full CGP infrastructure with cgp-serde
// in the jeb library.

use jeb::cgp_serde::{
    deserialize_with_context, serialize_with_context, serialize_with_context_pretty,
    OrderedContext, PrettyContext, StandardContext,
};
use jeb::KeyOrderOptions;
use serde_json::json;

fn main() {
    println!("=== CGP-Serde Full Infrastructure Demonstration ===\n");

    let value = json!({
        "name": "Alice",
        "id": 1,
        "age": 30,
        "city": "NYC",
        "metadata": {
            "updated": "2025-01-01",
            "version": 2
        }
    });

    println!("Original value:");
    println!("{}\n", serde_json::to_string_pretty(&value).unwrap());

    // Demonstration 1: Standard Context
    println!("=== 1. Standard Context (default serde_json) ===");
    let standard = StandardContext;
    let standard_json = serialize_with_context(&standard, &value).unwrap();
    println!("Serialized (compact): {}\n", standard_json);

    // Roundtrip test
    let deserialized: serde_json::Value =
        deserialize_with_context(&standard, &standard_json).unwrap();
    assert_eq!(value, deserialized);
    println!("✓ Roundtrip test passed\n");

    // Demonstration 2: Ordered Context
    println!("=== 2. Ordered Context (with key ordering) ===");
    let key_order = KeyOrderOptions {
        recursive: true,
        first: vec!["id".to_string(), "name".to_string()],
        last: vec!["metadata".to_string()],
        sort: true, // Sort remaining keys alphabetically
    };
    let ordered = OrderedContext::with_key_order(key_order);
    let ordered_json = serialize_with_context(&ordered, &value).unwrap();
    println!("Serialized (ordered keys): {}", ordered_json);
    println!("Note: 'id' and 'name' come first, 'metadata' last, others sorted\n");

    // Demonstration 3: Pretty Context
    println!("=== 3. Pretty Context (human-readable) ===");
    let pretty = PrettyContext::new();
    let pretty_json = serialize_with_context_pretty(&pretty, &value).unwrap();
    println!("Serialized (pretty-printed):");
    println!("{}\n", pretty_json);

    // Demonstration 4: Pretty Context with Ordering
    println!("=== 4. Pretty Context + Key Ordering ===");
    let ordered_pretty_key_order = KeyOrderOptions {
        recursive: true,
        first: vec!["id".to_string()],
        last: vec![],
        sort: true,
    };
    let ordered_pretty = PrettyContext::with_key_order(ordered_pretty_key_order);
    let ordered_pretty_json = serialize_with_context_pretty(&ordered_pretty, &value).unwrap();
    println!("Serialized (pretty + ordered):");
    println!("{}\n", ordered_pretty_json);

    // Demonstration 5: Multiple values with different contexts
    println!("=== 5. Same Data, Different Contexts ===");
    let simple_value = json!({"z": 3, "a": 1, "m": 2});

    let std_out = serialize_with_context(&standard, &simple_value).unwrap();
    let ord_out = serialize_with_context(&OrderedContext::new(), &simple_value).unwrap();
    let pretty_out = serialize_with_context_pretty(&pretty, &simple_value).unwrap();

    println!("Standard: {}", std_out);
    println!("Ordered:  {}", ord_out);
    println!("Pretty:");
    println!("{}", pretty_out);

    println!("\n=== Summary ===");
    println!("✓ All CGP-Serde contexts working with full infrastructure");
    println!("✓ Zero runtime overhead - all dispatch happens at compile time");
    println!("✓ Modular serialization strategies using cgp and cgp-serde");
    println!("\nFor more information:");
    println!("  - https://contextgeneric.dev/blog/cgp-serde-release/");
    println!("  - https://github.com/contextgeneric/cgp");
    println!("  - https://github.com/contextgeneric/cgp-serde");
}
