# CGP-Serde Concepts Demonstration in jeb

This document explains how **Context-Generic Programming (CGP)** concepts are demonstrated in the jeb library.

> **Note**: This is a conceptual demonstration using standard Rust traits. The actual cgp-serde library (v0.1.0) is still in early development and has some API instability. When it stabilizes, this implementation can be migrated to use the full CGP infrastructure.

## What is CGP-Serde?

CGP-Serde is a modular serialization library that extends Serde with Context-Generic Programming, enabling:

1. **Overlapping Implementations** - Write multiple serialization strategies for the same type
2. **Context-Dependent Behavior** - Different contexts provide different serialization behavior
3. **Modularity Without Derive Bloat** - Separate serialization logic from data types
4. **Bypass Coherence Restrictions** - Implement orphaned or overlapping traits

For more information, see: https://contextgeneric.dev/blog/cgp-serde-release/

## Integration Overview

The `jeb::cgp_serde` module provides three context types that demonstrate different serialization strategies:

### 1. StandardContext

Provides default serde_json serialization behavior.

```rust
use jeb::cgp_serde::{StandardContext, serialize_with_context};
use serde_json::json;

let context = StandardContext;
let value = json!({"name": "Alice", "age": 30});

let serialized = serialize_with_context(&context, &value).unwrap();
// Output: {"name":"Alice","age":30}
```

### 2. OrderedContext

Applies key ordering to JSON objects during serialization.

```rust
use jeb::cgp_serde::{OrderedContext, serialize_with_context};
use jeb::KeyOrderOptions;
use serde_json::json;

let key_order = KeyOrderOptions {
    recursive: true,
    first: vec!["id".to_string(), "name".to_string()],
    last: vec!["timestamp".to_string()],
    sort: true,
};

let context = OrderedContext::with_key_order(key_order);
let value = json!({
    "timestamp": "2025-01-01",
    "name": "Alice",
    "id": 1,
    "age": 30
});

let serialized = serialize_with_context(&context, &value).unwrap();
// Keys will be ordered: id, name, age (sorted), timestamp
```

### 3. PrettyContext

Serializes JSON with human-friendly formatting.

```rust
use jeb::cgp_serde::{PrettyContext, serialize_with_context_pretty};
use serde_json::json;

let context = PrettyContext::new();
let value = json!({"name": "Alice", "age": 30});

let serialized = serialize_with_context_pretty(&context, &value).unwrap();
/* Output:
{
  "name": "Alice",
  "age": 30
}
*/
```

## Deserialization

All contexts also support deserialization:

```rust
use jeb::cgp_serde::{StandardContext, deserialize_with_context};
use serde_json::Value;

let context = StandardContext;
let json = r#"{"name":"Bob","age":25}"#;

let value: Value = deserialize_with_context(&context, json).unwrap();
assert_eq!(value["name"], "Bob");
```

## Key Ordering Transformations

The module provides a helper function to apply key ordering transformations:

```rust
use jeb::cgp_serde::apply_context_ordering;
use jeb::KeyOrderOptions;
use serde_json::json;

let value = json!({
    "name": "Alice",
    "id": 1,
    "age": 30,
    "city": "NYC"
});

let key_order = KeyOrderOptions {
    recursive: true,
    first: vec!["id".to_string(), "name".to_string()],
    last: vec!["city".to_string()],
    sort: true,
};

let ordered = apply_context_ordering(&value, &key_order);
// Keys will be: id, name, age (sorted), city
```

## How It Works

### Context Types

Each context type is a zero-sized struct that implements specific CGP traits:

```rust
#[derive(Clone)]
pub struct StandardContext;

#[derive(Clone)]
pub struct OrderedContext {
    pub key_order: KeyOrderOptions,
}
```

### Component Delegation

The `delegate_components!` macro creates compile-time dispatch tables:

```rust
delegate_components! {
    StandardContext {
        ValueSerializerComponent: UseDelegate<JsonSerializerComponents>,
        ValueDeserializerComponent: UseDelegate<JsonDeserializerComponents>,
    }
}
```

### Serialization Providers

Providers implement the actual serialization logic:

```rust
pub struct OrderedSerializerComponents;

delegate_components! {
    OrderedSerializerComponents {
        [
            serde_json::Value,
            IndexMap<String, Value>,
        ]: SerializeToJson,
    }
}
```

## Benefits for jeb

### 1. Multiple Serialization Strategies

Before CGP-serde, we had one serialization path. Now we can have:

- Compact JSON (minimal whitespace)
- Pretty JSON (formatted with indentation)
- Ordered JSON (keys sorted according to specifications)
- Binary-sortable JSON (custom encoding for database indexes)

### 2. Extensibility

Users of the jeb library can define their own contexts and providers:

```rust
// User-defined context
#[derive(Clone)]
pub struct CustomContext;

delegate_components! {
    CustomContext {
        ValueSerializerComponent: UseDelegate<MyCustomSerializers>,
    }
}
```

### 3. Composability

Different contexts can be composed and mixed:

```rust
let standard = StandardContext;
let ordered = OrderedContext::new();
let pretty = PrettyContext::new();

// Use different contexts for different purposes
let compact_json = serialize_with_context(&standard, &value)?;
let sorted_json = serialize_with_context(&ordered, &value)?;
let readable_json = serialize_with_context_pretty(&pretty, &value)?;
```

## Integration with Existing Features

CGP-serde complements jeb's existing features:

### JSON Total Ordering

The `json_total_order` function remains independent and can be used alongside CGP contexts:

```rust
use jeb::{json_total_order, cgp_serde::StandardContext};
use std::cmp::Ordering;

let v1 = json!("string");
let v2 = json!(42);

assert_eq!(json_total_order(&v1, &v2), Ordering::Less);
```

### Key Ordering

The `KeyOrderOptions` type is used by `OrderedContext` to control key ordering:

```rust
let key_order = KeyOrderOptions::parse(r#"["id", true, "metadata"]"#).unwrap();
let context = OrderedContext::with_key_order(key_order);
```

### Stream Processing

CGP-serde can be integrated into stream processing pipelines (future work):

```rust
// Future: Apply context-dependent serialization to streams
stream
    .map(|obj| apply_context_ordering(&obj, &key_order))
    .map(|obj| serialize_with_context(&context, &obj))
```

## Future Enhancements

Potential future improvements include:

1. **Custom Binary Encodings** - Context-specific binary serialization for database indexes
2. **Streaming Serialization** - Apply contexts to async streams of JSON objects
3. **Performance Optimization** - Zero-copy serialization with arena allocators
4. **Format Conversions** - Contexts for converting between JSON, CBOR, MessagePack, etc.

## Testing

The module includes comprehensive tests:

```bash
# Run all tests including cgp_serde tests
cargo test

# Run only cgp_serde tests
cargo test cgp_serde
```

Test coverage includes:

- Standard context serialization/deserialization
- Ordered context with key ordering
- Pretty-print context
- Context ordering transformations
- Roundtrip serialization/deserialization
- Integration with json_total_order

## Performance Considerations

CGP-serde uses compile-time dispatch, so there is **zero runtime overhead** compared to direct serde usage. All context resolution happens at compile time through the type system.

## References

- [CGP-Serde Release Announcement](https://contextgeneric.dev/blog/cgp-serde-release/)
- [Context-Generic Programming](https://contextgeneric.dev/)
- [CGP Core Library](https://github.com/contextgeneric/cgp)
- [CGP-Serde Library](https://github.com/contextgeneric/cgp-serde)

## License

The CGP-serde integration in jeb is licensed under the same terms as the jeb project (MIT OR Apache-2.0).
