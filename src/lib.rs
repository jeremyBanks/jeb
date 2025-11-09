use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, info, instrument};
use wasm_bindgen::prelude::*;

/// Type alias for JSON objects
pub type JsonObject = serde_json::Map<String, Value>;

/// A simple data structure to demonstrate serde serialization.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Entity {
    pub id: i64,
    pub name: String,
    pub metadata: Option<String>,
}

impl Entity {
    pub fn new(id: i64, name: String) -> Self {
        Self {
            id,
            name,
            metadata: None,
        }
    }

    pub fn with_metadata(mut self, metadata: String) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// A simple greeting function that returns a formatted message.
#[wasm_bindgen]
#[instrument]
pub fn greet(name: &str) -> String {
    info!("Greeting user: {}", name);
    format!("Hello, {}!", name)
}

/// Process some data - placeholder implementation.
#[instrument(skip(data))]
pub fn process_data(data: &[u8]) -> Vec<u8> {
    debug!("Processing {} bytes of data", data.len());
    // Placeholder: just returns a copy of the input
    data.to_vec()
}

/// Calculate something - placeholder implementation.
#[instrument]
pub fn calculate(x: i32, y: i32) -> i32 {
    debug!("Calculating: {} + {}", x, y);
    // Placeholder: simple addition
    x + y
}

/// Serialize an Entity to JSON.
#[instrument(skip(entity))]
pub fn entity_to_json(entity: &Entity) -> Result<String, serde_json::Error> {
    info!("Serializing entity with id: {}", entity.id);
    serde_json::to_string(entity)
}

/// Deserialize an Entity from JSON.
#[instrument(skip(json))]
pub fn entity_from_json(json: &str) -> Result<Entity, serde_json::Error> {
    debug!("Deserializing entity from JSON");
    serde_json::from_str(json)
}

/// Parse a stream of JSON objects from a string.
///
/// This function handles multiple input formats:
/// - JSON lines (newline-delimited JSON objects)
/// - JSON arrays
/// - Concatenated JSON objects (no delimiter)
///
/// It scans for '{' characters and parses JSON objects from those positions,
/// ignoring any content that isn't part of a JSON object.
#[instrument(skip(input))]
pub fn parse_json_stream(input: &str) -> Result<Vec<JsonObject>, Box<dyn std::error::Error>> {
    let mut objects = Vec::new();
    let mut chars = input.char_indices().peekable();

    while let Some((start_idx, ch)) = chars.next() {
        // Skip until we find an opening brace
        if ch != '{' {
            continue;
        }

        // Found a potential JSON object, try to parse it
        // We need to find the matching closing brace
        let json_str = &input[start_idx..];

        match extract_json_object(json_str) {
            Some((obj_str, consumed_len)) => {
                // Try to parse as a JSON object
                match serde_json::from_str::<Value>(obj_str) {
                    Ok(Value::Object(obj)) => {
                        debug!("Parsed JSON object with {} keys", obj.len());
                        objects.push(obj);

                        // Skip ahead by the consumed length minus 1
                        // (minus 1 because we already consumed the first char)
                        for _ in 1..consumed_len {
                            chars.next();
                        }
                    }
                    Ok(_) => {
                        debug!("Parsed JSON value but not an object, skipping");
                    }
                    Err(e) => {
                        debug!("Failed to parse JSON at position {}: {}", start_idx, e);
                    }
                }
            }
            None => {
                debug!("Could not extract JSON object at position {}", start_idx);
            }
        }
    }

    info!("Parsed {} JSON objects from stream", objects.len());
    Ok(objects)
}

/// Extract a JSON object from the beginning of a string.
/// Returns the JSON string and the number of characters consumed.
fn extract_json_object(input: &str) -> Option<(&str, usize)> {
    let mut depth = 0;
    let mut in_string = false;
    let mut escape_next = false;

    for (i, ch) in input.char_indices() {
        if escape_next {
            escape_next = false;
            continue;
        }

        match ch {
            '\\' if in_string => {
                escape_next = true;
            }
            '"' => {
                in_string = !in_string;
            }
            '{' if !in_string => {
                depth += 1;
            }
            '}' if !in_string => {
                depth -= 1;
                if depth == 0 {
                    // Found the closing brace
                    return Some((&input[..=i], i + 1));
                }
            }
            _ => {}
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greet() {
        let result = greet("World");
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_greet_empty() {
        let result = greet("");
        assert_eq!(result, "Hello, !");
    }

    #[test]
    fn test_process_data() {
        let data = vec![1, 2, 3, 4, 5];
        let result = process_data(&data);
        assert_eq!(result, data);
    }

    #[test]
    fn test_process_data_empty() {
        let data: Vec<u8> = vec![];
        let result = process_data(&data);
        assert_eq!(result, data);
    }

    #[test]
    fn test_calculate() {
        assert_eq!(calculate(2, 3), 5);
        assert_eq!(calculate(-1, 1), 0);
        assert_eq!(calculate(0, 0), 0);
    }

    #[test]
    fn test_entity_creation() {
        let entity = Entity::new(1, "Test Entity".to_string());
        assert_eq!(entity.id, 1);
        assert_eq!(entity.name, "Test Entity");
        assert_eq!(entity.metadata, None);
    }

    #[test]
    fn test_entity_with_metadata() {
        let entity = Entity::new(1, "Test".to_string())
            .with_metadata("Some metadata".to_string());
        assert_eq!(entity.metadata, Some("Some metadata".to_string()));
    }

    #[test]
    fn test_entity_serialization() {
        let entity = Entity::new(42, "John Doe".to_string());
        let json = entity_to_json(&entity).unwrap();
        assert!(json.contains("\"id\":42"));
        assert!(json.contains("\"name\":\"John Doe\""));
    }

    #[test]
    fn test_entity_deserialization() {
        let json = r#"{"id":123,"name":"Jane Doe","metadata":null}"#;
        let entity = entity_from_json(json).unwrap();
        assert_eq!(entity.id, 123);
        assert_eq!(entity.name, "Jane Doe");
        assert_eq!(entity.metadata, None);
    }

    #[test]
    fn test_entity_round_trip() {
        let original = Entity::new(999, "Round Trip".to_string())
            .with_metadata("test metadata".to_string());
        let json = entity_to_json(&original).unwrap();
        let deserialized = entity_from_json(&json).unwrap();
        assert_eq!(original, deserialized);
    }

    // JSON stream parsing tests

    #[test]
    fn test_parse_json_lines() {
        let input = r#"{"id": 1, "name": "Alice"}
{"id": 2, "name": "Bob"}
{"id": 3, "name": "Charlie"}"#;

        let objects = parse_json_stream(input).unwrap();
        assert_eq!(objects.len(), 3);
        assert_eq!(objects[0].get("id").unwrap(), &Value::from(1));
        assert_eq!(objects[0].get("name").unwrap(), &Value::from("Alice"));
        assert_eq!(objects[1].get("id").unwrap(), &Value::from(2));
        assert_eq!(objects[2].get("id").unwrap(), &Value::from(3));
    }

    #[test]
    fn test_parse_json_array() {
        let input = r#"[
            {"id": 1, "name": "Alice"},
            {"id": 2, "name": "Bob"},
            {"id": 3, "name": "Charlie"}
        ]"#;

        let objects = parse_json_stream(input).unwrap();
        assert_eq!(objects.len(), 3);
        assert_eq!(objects[0].get("id").unwrap(), &Value::from(1));
        assert_eq!(objects[1].get("id").unwrap(), &Value::from(2));
        assert_eq!(objects[2].get("id").unwrap(), &Value::from(3));
    }

    #[test]
    fn test_parse_concatenated_json() {
        let input = r#"{"id": 1, "name": "Alice"}{"id": 2, "name": "Bob"}{"id": 3, "name": "Charlie"}"#;

        let objects = parse_json_stream(input).unwrap();
        assert_eq!(objects.len(), 3);
        assert_eq!(objects[0].get("id").unwrap(), &Value::from(1));
        assert_eq!(objects[1].get("id").unwrap(), &Value::from(2));
        assert_eq!(objects[2].get("id").unwrap(), &Value::from(3));
    }

    #[test]
    fn test_parse_mixed_format() {
        let input = r#"Some preamble text
{"id": 1, "name": "Alice"}
Random text in between
{"id": 2, "name": "Bob"}
[{"id": 3, "name": "Charlie"}]"#;

        let objects = parse_json_stream(input).unwrap();
        assert_eq!(objects.len(), 3);
        assert_eq!(objects[0].get("id").unwrap(), &Value::from(1));
        assert_eq!(objects[1].get("id").unwrap(), &Value::from(2));
        assert_eq!(objects[2].get("id").unwrap(), &Value::from(3));
    }

    #[test]
    fn test_parse_nested_objects() {
        let input = r#"{"id": 1, "data": {"nested": "value"}}
{"id": 2, "data": {"nested": {"deep": "value"}}}"#;

        let objects = parse_json_stream(input).unwrap();
        assert_eq!(objects.len(), 2);
        assert_eq!(objects[0].get("id").unwrap(), &Value::from(1));
        assert!(objects[0].get("data").unwrap().is_object());
        assert_eq!(objects[1].get("id").unwrap(), &Value::from(2));
    }

    #[test]
    fn test_parse_with_escaped_braces() {
        let input = r#"{"id": 1, "text": "This has a } in it"}
{"id": 2, "text": "And this has a { in it"}"#;

        let objects = parse_json_stream(input).unwrap();
        assert_eq!(objects.len(), 2);
        assert_eq!(
            objects[0].get("text").unwrap(),
            &Value::from("This has a } in it")
        );
        assert_eq!(
            objects[1].get("text").unwrap(),
            &Value::from("And this has a { in it")
        );
    }

    #[test]
    fn test_parse_empty_input() {
        let input = "";
        let objects = parse_json_stream(input).unwrap();
        assert_eq!(objects.len(), 0);
    }

    #[test]
    fn test_parse_no_json_objects() {
        let input = "This is just plain text with no JSON objects";
        let objects = parse_json_stream(input).unwrap();
        assert_eq!(objects.len(), 0);
    }

    #[test]
    fn test_extract_json_object() {
        let input = r#"{"id": 1, "name": "test"}"#;
        let result = extract_json_object(input);
        assert!(result.is_some());
        let (extracted, len) = result.unwrap();
        assert_eq!(extracted, r#"{"id": 1, "name": "test"}"#);
        assert_eq!(len, input.len());
    }

    #[test]
    fn test_extract_json_object_with_trailing() {
        let input = r#"{"id": 1}trailing data"#;
        let result = extract_json_object(input);
        assert!(result.is_some());
        let (extracted, len) = result.unwrap();
        assert_eq!(extracted, r#"{"id": 1}"#);
        assert_eq!(len, 9);
    }
}
