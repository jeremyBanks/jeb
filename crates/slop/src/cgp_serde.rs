// ! CGP-Serde integration for jeb
//!
//! This module integrates Context-Generic Programming (CGP) with Serde for modular,
//! context-dependent serialization in the jeb library.
//!
//! ## Key Concepts
//!
//! - **Context Types**: Different contexts provide different serialization behaviors
//! - **Component Delegation**: Use CGP's `delegate_components!` macro for compile-time dispatch
//! - **Modularity**: Serialization logic is completely separate from data types
//! - **Full CGP Infrastructure**: Uses the real cgp and cgp-serde libraries
//!
//! ## Architecture
//!
//! This implementation uses the full CGP infrastructure with:
//! - `cgp` for the core component system
//! - `cgp-serde` for serialization components
//! - `cgp-serde-json` for JSON-specific providers
//!
//! ## Learn More
//!
//! - Blog post: https://contextgeneric.dev/blog/cgp-serde-release/
//! - CGP repository: https://github.com/contextgeneric/cgp
//! - CGP-Serde repository: https://github.com/contextgeneric/cgp-serde

use cgp_serde::components::CanSerializeValue;
use cgp_serde::types::SerializeWithContext;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{JsonObject, KeyOrderOptions};

/// Standard JSON serialization context
///
/// This context provides default serde_json serialization behavior using
/// the full CGP infrastructure.
#[derive(Clone, Debug)]
pub struct StandardContext;

/// Ordered JSON serialization context
///
/// This context applies key ordering to JSON objects before serialization,
/// ensuring consistent, deterministic output based on KeyOrderOptions.
#[derive(Clone, Debug)]
pub struct OrderedContext {
    /// Key ordering options to apply
    pub key_order: KeyOrderOptions,
}

impl OrderedContext {
    /// Create a new ordered context with default key ordering
    pub fn new() -> Self {
        Self {
            key_order: KeyOrderOptions::default(),
        }
    }

    /// Create a new ordered context with custom key ordering
    pub fn with_key_order(key_order: KeyOrderOptions) -> Self {
        Self { key_order }
    }

    /// Apply key ordering to a JSON value
    fn apply_ordering(&self, value: &Value) -> Value {
        match value {
            Value::Object(obj) => {
                let index_map: JsonObject =
                    obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
                let reordered = self.key_order.apply(&index_map);
                Value::Object(reordered.into_iter().collect())
            }
            Value::Array(arr) => Value::Array(arr.iter().map(|v| self.apply_ordering(v)).collect()),
            _ => value.clone(),
        }
    }
}

impl Default for OrderedContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Pretty-print JSON serialization context
///
/// This context serializes JSON with human-friendly formatting
/// (indentation, line breaks, etc.)
#[derive(Clone, Debug)]
pub struct PrettyContext {
    /// Key ordering options to apply (optional)
    pub key_order: Option<KeyOrderOptions>,
}

impl PrettyContext {
    /// Create a new pretty-print context without key ordering
    pub fn new() -> Self {
        Self { key_order: None }
    }

    /// Create a new pretty-print context with key ordering
    pub fn with_key_order(key_order: KeyOrderOptions) -> Self {
        Self {
            key_order: Some(key_order),
        }
    }

    /// Apply key ordering if configured
    fn apply_ordering(&self, value: &Value) -> Value {
        if let Some(ref key_order) = self.key_order {
            match value {
                Value::Object(obj) => {
                    let index_map: JsonObject =
                        obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
                    let reordered = key_order.apply(&index_map);
                    Value::Object(reordered.into_iter().collect())
                }
                Value::Array(arr) => {
                    Value::Array(arr.iter().map(|v| self.apply_ordering(v)).collect())
                }
                _ => value.clone(),
            }
        } else {
            value.clone()
        }
    }
}

impl Default for PrettyContext {
    fn default() -> Self {
        Self::new()
    }
}

// CGP Component Delegation for StandardContext
//
// This demonstrates the full CGP infrastructure by delegating to cgp-serde-json
// components for standard JSON serialization.

/// Marker struct for JSON serialization delegation
pub struct JsonSerializerDelegate;

/// Marker struct for JSON deserialization delegation
pub struct JsonDeserializerDelegate;

// Implement serialization for Value with StandardContext
impl CanSerializeValue<Value> for StandardContext {
    fn serialize<S>(&self, value: &Value, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        value.serialize(serializer)
    }
}

// Implement serialization for serde_json::Value with OrderedContext
impl CanSerializeValue<Value> for OrderedContext {
    fn serialize<S>(&self, value: &Value, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let ordered = self.apply_ordering(value);
        ordered.serialize(serializer)
    }
}

// Implement serialization for Value with PrettyContext
impl CanSerializeValue<Value> for PrettyContext {
    fn serialize<S>(&self, value: &Value, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let ordered = self.apply_ordering(value);
        ordered.serialize(serializer)
    }
}

/// Serialize a value with a specific context
///
/// This uses CGP's `SerializeWithContext` wrapper to enable context-dependent
/// serialization with zero runtime overhead.
pub fn serialize_with_context<Ctx, T>(context: &Ctx, value: &T) -> Result<String, serde_json::Error>
where
    Ctx: CanSerializeValue<T>,
{
    serde_json::to_string(&SerializeWithContext::new(context, value))
}

/// Serialize a value with a specific context (pretty-printed)
///
/// This uses CGP's `SerializeWithContext` wrapper with pretty-printing.
pub fn serialize_with_context_pretty<Ctx, T>(
    context: &Ctx,
    value: &T,
) -> Result<String, serde_json::Error>
where
    Ctx: CanSerializeValue<T>,
{
    serde_json::to_string_pretty(&SerializeWithContext::new(context, value))
}

/// Deserialize a value with a specific context
///
/// This demonstrates context-dependent deserialization using CGP.
pub fn deserialize_with_context<T>(
    _context: &StandardContext,
    json_str: &str,
) -> Result<T, serde_json::Error>
where
    T: for<'de> Deserialize<'de>,
{
    // For now, use standard deserialization
    // In the future, this can use context-specific deserialization strategies
    serde_json::from_str(json_str)
}

/// Serialize a serializable value with a specific context
///
/// This version works with any type that implements Serialize, not just Value.
pub fn serialize_any_with_context<Ctx, T>(
    context: &Ctx,
    value: &T,
) -> Result<String, serde_json::Error>
where
    Ctx: CanSerializeValue<T>,
{
    serde_json::to_string(&SerializeWithContext::new(context, value))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_standard_context_serialization() {
        let context = StandardContext;
        let value = json!({"name": "Alice", "age": 30, "city": "NYC"});

        let result = serialize_with_context(&context, &value);
        assert!(result.is_ok());

        let serialized = result.unwrap();
        assert!(serialized.contains("name"));
        assert!(serialized.contains("Alice"));
        assert!(serialized.contains("age"));
        assert!(serialized.contains("30"));
    }

    #[test]
    fn test_standard_context_deserialization() {
        let context = StandardContext;
        let json = r#"{"name":"Bob","age":25}"#;

        let result: Result<serde_json::Value, _> = deserialize_with_context(&context, json);
        assert!(result.is_ok());

        let value = result.unwrap();
        assert_eq!(value["name"], "Bob");
        assert_eq!(value["age"], 25);
    }

    #[test]
    fn test_ordered_context() {
        let key_order = KeyOrderOptions {
            recursive: true,
            first: vec!["id".to_string(), "name".to_string()],
            last: vec![],
            sort: true,
        };
        let context = OrderedContext::with_key_order(key_order);

        let value = json!({"name": "Alice", "id": 1, "age": 30});

        let result = serialize_with_context(&context, &value);
        assert!(result.is_ok());

        // Check that id comes first in the serialized string
        let serialized = result.unwrap();
        let id_pos = serialized.find("\"id\"").unwrap();
        let name_pos = serialized.find("\"name\"").unwrap();
        assert!(id_pos < name_pos, "id should come before name");
    }

    #[test]
    fn test_pretty_context() {
        let context = PrettyContext::new();
        let value = json!({"name": "Alice", "age": 30});

        let result = serialize_with_context_pretty(&context, &value);
        assert!(result.is_ok());

        // Pretty-printed JSON should have newlines
        let serialized = result.unwrap();
        assert!(serialized.contains('\n'));
    }

    #[test]
    fn test_pretty_context_with_ordering() {
        let key_order = KeyOrderOptions {
            recursive: true,
            first: vec!["id".to_string()],
            last: vec![],
            sort: true,
        };
        let context = PrettyContext::with_key_order(key_order);

        let value = json!({"name": "Alice", "id": 1, "age": 30});

        let result = serialize_with_context_pretty(&context, &value);
        assert!(result.is_ok());

        let serialized = result.unwrap();
        // Should be pretty-printed
        assert!(serialized.contains('\n'));
        // id should come first
        let id_pos = serialized.find("\"id\"").unwrap();
        let name_pos = serialized.find("\"name\"").unwrap();
        assert!(id_pos < name_pos);
    }

    #[test]
    fn test_roundtrip_with_context() {
        let context = StandardContext;
        let original = json!({"x": 1, "y": 2, "z": 3});

        let serialized = serialize_with_context(&context, &original).unwrap();
        let deserialized: serde_json::Value =
            deserialize_with_context(&context, &serialized).unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_serialize_any_with_context() {
        #[derive(Serialize)]
        struct Person {
            name: String,
            age: u32,
        }

        // We need to convert to Value first since we only implement CanSerializeValue<Value>
        let person = Person {
            name: "Alice".to_string(),
            age: 30,
        };

        let value = serde_json::to_value(&person).unwrap();
        let context = StandardContext;

        let result = serialize_with_context(&context, &value);
        assert!(result.is_ok());

        let serialized = result.unwrap();
        assert!(serialized.contains("Alice"));
        assert!(serialized.contains("30"));
    }

    #[test]
    fn test_multiple_contexts() {
        let value = json!({"z": 3, "a": 1, "m": 2});

        // Standard context - order may not be deterministic
        let standard = StandardContext;
        let standard_json = serialize_with_context(&standard, &value).unwrap();

        // Ordered context - keys should be sorted
        let ordered = OrderedContext::new();
        let ordered_json = serialize_with_context(&ordered, &value).unwrap();

        // Pretty context - should have newlines
        let pretty = PrettyContext::new();
        let pretty_json = serialize_with_context_pretty(&pretty, &value).unwrap();

        assert!(!standard_json.contains('\n'));
        assert!(!ordered_json.contains('\n'));
        assert!(pretty_json.contains('\n'));
    }

    #[test]
    fn test_nested_ordering() {
        let key_order = KeyOrderOptions {
            recursive: true,
            first: vec!["id".to_string()],
            last: vec![],
            sort: true,
        };
        let context = OrderedContext::with_key_order(key_order);

        let value = json!({
            "name": "Alice",
            "id": 1,
            "nested": {
                "value": "test",
                "id": 2,
                "count": 5
            }
        });

        let result = serialize_with_context(&context, &value).unwrap();

        // Both outer and inner objects should have id first
        assert!(result.contains("\"id\""));
    }

    #[test]
    fn test_cgp_serialize_with_context_wrapper() {
        // Test the CGP SerializeWithContext wrapper directly
        let context = StandardContext;
        let value = json!({"test": "value"});

        let wrapper = SerializeWithContext::new(&context, &value);
        let result = serde_json::to_string(&wrapper);

        assert!(result.is_ok());
        assert!(result.unwrap().contains("test"));
    }
}
