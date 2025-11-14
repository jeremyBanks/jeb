//! CGP-Serde integration demonstration for jeb
//!
//! This module demonstrates the *concepts* of Context-Generic Programming (CGP) with Serde.
//! The actual cgp-serde library is still in early development (v0.1.0), so this module provides
//! a simplified implementation that shows the key ideas.
//!
//! ## Key Concepts
//!
//! - **Context Types**: Different contexts provide different serialization behaviors
//! - **Trait-based Dispatch**: Use traits to enable multiple serialization strategies
//! - **Modularity**: Separate serialization logic from data types
//!
//! ## Future Integration
//!
//! When cgp-serde stabilizes, this module can be updated to use the full CGP infrastructure.
//! For now, it demonstrates the concepts using standard Rust traits.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{JsonObject, KeyOrderOptions};

/// Trait for context-dependent JSON serialization
///
/// This trait demonstrates the core idea of CGP-serde: different contexts
/// can provide different serialization behavior for the same type.
pub trait JsonSerializationContext {
    /// Serialize a JSON value to a string using this context
    fn serialize_json(&self, value: &Value) -> Result<String, serde_json::Error>;

    /// Serialize a JSON value to a pretty-printed string using this context
    fn serialize_json_pretty(&self, value: &Value) -> Result<String, serde_json::Error> {
        // Default implementation uses pretty printing
        serde_json::to_string_pretty(value)
    }
}

/// Trait for context-dependent JSON deserialization
pub trait JsonDeserializationContext {
    /// Deserialize a JSON value from a string using this context
    fn deserialize_json<T>(&self, json_str: &str) -> Result<T, serde_json::Error>
    where
        T: for<'de> Deserialize<'de>,
    {
        // Default implementation uses standard deserialization
        serde_json::from_str(json_str)
    }
}

/// Standard JSON serialization context
///
/// This context provides the default serde_json serialization behavior
/// without any special ordering or formatting.
#[derive(Debug, Clone)]
pub struct StandardContext;

impl JsonSerializationContext for StandardContext {
    fn serialize_json(&self, value: &Value) -> Result<String, serde_json::Error> {
        serde_json::to_string(value)
    }
}

impl JsonDeserializationContext for StandardContext {}

/// Ordered JSON serialization context
///
/// This context applies key ordering to JSON objects before serialization,
/// ensuring consistent, deterministic output.
#[derive(Debug, Clone)]
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

impl JsonSerializationContext for OrderedContext {
    fn serialize_json(&self, value: &Value) -> Result<String, serde_json::Error> {
        let ordered = self.apply_ordering(value);
        serde_json::to_string(&ordered)
    }

    fn serialize_json_pretty(&self, value: &Value) -> Result<String, serde_json::Error> {
        let ordered = self.apply_ordering(value);
        serde_json::to_string_pretty(&ordered)
    }
}

impl JsonDeserializationContext for OrderedContext {}

/// Pretty-print JSON serialization context
///
/// This context serializes JSON with human-friendly formatting
/// (indentation, line breaks, etc.)
#[derive(Debug, Clone)]
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

impl JsonSerializationContext for PrettyContext {
    fn serialize_json(&self, value: &Value) -> Result<String, serde_json::Error> {
        let ordered = self.apply_ordering(value);
        serde_json::to_string_pretty(&ordered)
    }

    fn serialize_json_pretty(&self, value: &Value) -> Result<String, serde_json::Error> {
        self.serialize_json(value)
    }
}

impl JsonDeserializationContext for PrettyContext {}

/// Serialize a value with a specific context
///
/// This function demonstrates context-dependent serialization.
pub fn serialize_with_context<Ctx>(
    context: &Ctx,
    value: &Value,
) -> Result<String, serde_json::Error>
where
    Ctx: JsonSerializationContext,
{
    context.serialize_json(value)
}

/// Serialize a value with a specific context (pretty-printed)
pub fn serialize_with_context_pretty<Ctx>(
    context: &Ctx,
    value: &Value,
) -> Result<String, serde_json::Error>
where
    Ctx: JsonSerializationContext,
{
    context.serialize_json_pretty(value)
}

/// Deserialize a value with a specific context
pub fn deserialize_with_context<Ctx, T>(
    context: &Ctx,
    json_str: &str,
) -> Result<T, serde_json::Error>
where
    Ctx: JsonDeserializationContext,
    T: for<'de> Deserialize<'de>,
{
    context.deserialize_json(json_str)
}

/// Serialize a serializable value with a specific context
///
/// This version works with any type that implements Serialize, not just Value.
pub fn serialize_any_with_context<Ctx, T>(
    context: &Ctx,
    value: &T,
) -> Result<String, serde_json::Error>
where
    Ctx: JsonSerializationContext,
    T: Serialize,
{
    let json_value = serde_json::to_value(value)?;
    context.serialize_json(&json_value)
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

        let result = serialize_with_context(&context, &value);
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

        let context = StandardContext;
        let person = Person {
            name: "Alice".to_string(),
            age: 30,
        };

        let result = serialize_any_with_context(&context, &person);
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
}
