//! Module for converting between streams of text and streams of serde_json::Value
//!
//! This module provides functionality to parse JSON objects from various text formats
//! and convert them to/from async streams.

use async_stream::stream;
use futures::stream::Stream;
use indexmap::IndexMap;
use serde_json::Value;
use tokio::io::{AsyncBufRead, AsyncReadExt, AsyncWriteExt, BufReader};
use tracing::{debug, info, instrument};

/// Type alias for JSON objects using IndexMap to preserve insertion order
pub type JsonObject = IndexMap<String, Value>;

/// Type alias for JSON parsing errors
pub type JsonError = Box<dyn std::error::Error + Send + Sync>;

/// Parse a stream of JSON objects from an async reader
///
/// This function handles multiple input formats:
/// - JSON lines (newline-delimited JSON objects)
/// - JSON arrays
/// - Concatenated JSON objects (no delimiter)
///
/// It scans for '{' characters and parses JSON objects from those positions,
/// ignoring any content that isn't part of a JSON object.
pub fn parse_json_stream<R: AsyncBufRead + Unpin + Send + 'static>(
    reader: R,
) -> impl Stream<Item = Result<JsonObject, JsonError>> {
    stream! {
        let mut reader = BufReader::new(reader);
        let mut buffer = String::new();

        // Read entire content (for now - can optimize later for true streaming)
        match reader.read_to_string(&mut buffer).await {
            Ok(_) => {
                debug!("Read {} bytes from input", buffer.len());

                // Parse using the synchronous parser
                match parse_json_string(&buffer) {
                    Ok(objects) => {
                        for obj in objects {
                            yield Ok(obj);
                        }
                    }
                    Err(e) => {
                        yield Err(e);
                    }
                }
            }
            Err(e) => {
                yield Err(Box::new(e) as JsonError);
            }
        }
    }
}

/// Parse JSON objects from a string (synchronous helper)
///
/// This is the core parsing logic, used by both sync and async variants
#[instrument(skip(input))]
pub fn parse_json_string(input: &str) -> Result<Vec<JsonObject>, JsonError> {
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
                        // Convert serde_json::Map to IndexMap to preserve order
                        let index_map: IndexMap<String, Value> = obj.into_iter().collect();
                        objects.push(index_map);

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

/// Write JSON objects to an async writer in line-by-line friendly format
///
/// Writes objects as a JSON array with special formatting:
/// - First line: `[` followed by first object
/// - Middle lines: `,` followed by each object
/// - Last line: `]`
///
/// This format is both valid JSON and line-by-line processable (skip first character).
///
/// # Example Output
/// ```text
/// [{"id":1,"name":"Alice"}
/// ,{"id":2,"name":"Bob"}
/// ]
/// ```
pub async fn write_json_array<W: tokio::io::AsyncWrite + Unpin>(
    writer: &mut W,
    objects: &[JsonObject],
) -> Result<(), std::io::Error> {
    for (i, obj) in objects.iter().enumerate() {
        let json_str = serde_json::to_string(obj)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        if i == 0 {
            writer
                .write_all(format!("[{}\n", json_str).as_bytes())
                .await?;
        } else {
            writer
                .write_all(format!(",{}\n", json_str).as_bytes())
                .await?;
        }
    }

    // Write closing bracket (or just [] if empty)
    if objects.is_empty() {
        writer.write_all(b"[]\n").await?;
    } else {
        writer.write_all(b"]\n").await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_json_lines() {
        let input = r#"{"id": 1, "name": "Alice"}
{"id": 2, "name": "Bob"}
{"id": 3, "name": "Charlie"}"#;

        let objects = parse_json_string(input).unwrap();
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

        let objects = parse_json_string(input).unwrap();
        assert_eq!(objects.len(), 3);
        assert_eq!(objects[0].get("id").unwrap(), &Value::from(1));
        assert_eq!(objects[1].get("id").unwrap(), &Value::from(2));
        assert_eq!(objects[2].get("id").unwrap(), &Value::from(3));
    }

    #[test]
    fn test_parse_concatenated_json() {
        let input =
            r#"{"id": 1, "name": "Alice"}{"id": 2, "name": "Bob"}{"id": 3, "name": "Charlie"}"#;

        let objects = parse_json_string(input).unwrap();
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

        let objects = parse_json_string(input).unwrap();
        assert_eq!(objects.len(), 3);
        assert_eq!(objects[0].get("id").unwrap(), &Value::from(1));
        assert_eq!(objects[1].get("id").unwrap(), &Value::from(2));
        assert_eq!(objects[2].get("id").unwrap(), &Value::from(3));
    }

    #[test]
    fn test_parse_nested_objects() {
        let input = r#"{"id": 1, "data": {"nested": "value"}}
{"id": 2, "data": {"nested": {"deep": "value"}}}"#;

        let objects = parse_json_string(input).unwrap();
        assert_eq!(objects.len(), 2);
        assert_eq!(objects[0].get("id").unwrap(), &Value::from(1));
        assert!(objects[0].get("data").unwrap().is_object());
        assert_eq!(objects[1].get("id").unwrap(), &Value::from(2));
    }

    #[test]
    fn test_parse_with_escaped_braces() {
        let input = r#"{"id": 1, "text": "This has a } in it"}
{"id": 2, "text": "And this has a { in it"}"#;

        let objects = parse_json_string(input).unwrap();
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
        let objects = parse_json_string(input).unwrap();
        assert_eq!(objects.len(), 0);
    }

    #[test]
    fn test_parse_no_json_objects() {
        let input = "This is just plain text with no JSON objects";
        let objects = parse_json_string(input).unwrap();
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

    #[tokio::test]
    async fn test_write_json_array_empty() {
        let objects: Vec<JsonObject> = vec![];
        let mut buffer = Vec::new();

        write_json_array(&mut buffer, &objects).await.unwrap();

        let output = String::from_utf8(buffer).unwrap();
        assert_eq!(output, "[]\n");
    }

    #[tokio::test]
    async fn test_write_json_array_single() {
        let mut obj = IndexMap::new();
        obj.insert("id".to_string(), Value::from(1));
        obj.insert("name".to_string(), Value::from("Alice"));
        let objects = vec![obj];

        let mut buffer = Vec::new();
        write_json_array(&mut buffer, &objects).await.unwrap();

        let output = String::from_utf8(buffer).unwrap();
        assert_eq!(
            output,
            r#"[{"id":1,"name":"Alice"}
]
"#
        );
    }

    #[tokio::test]
    async fn test_write_json_array_multiple() {
        let mut obj1 = IndexMap::new();
        obj1.insert("id".to_string(), Value::from(1));
        obj1.insert("name".to_string(), Value::from("Alice"));

        let mut obj2 = IndexMap::new();
        obj2.insert("id".to_string(), Value::from(2));
        obj2.insert("name".to_string(), Value::from("Bob"));

        let objects = vec![obj1, obj2];

        let mut buffer = Vec::new();
        write_json_array(&mut buffer, &objects).await.unwrap();

        let output = String::from_utf8(buffer).unwrap();
        assert_eq!(
            output,
            r#"[{"id":1,"name":"Alice"}
,{"id":2,"name":"Bob"}
]
"#
        );
    }

    #[tokio::test]
    async fn test_write_json_array_is_valid_json() {
        let mut obj1 = IndexMap::new();
        obj1.insert("id".to_string(), Value::from(1));
        let mut obj2 = IndexMap::new();
        obj2.insert("id".to_string(), Value::from(2));
        let objects = vec![obj1, obj2];

        let mut buffer = Vec::new();
        write_json_array(&mut buffer, &objects).await.unwrap();

        let output = String::from_utf8(buffer).unwrap();

        // Verify output is valid JSON by parsing it
        let parsed: Value = serde_json::from_str(&output).unwrap();
        assert!(parsed.is_array());
        let array = parsed.as_array().unwrap();
        assert_eq!(array.len(), 2);
        assert_eq!(array[0]["id"], 1);
        assert_eq!(array[1]["id"], 2);
    }

    #[tokio::test]
    async fn test_write_then_parse_roundtrip() {
        // Create some objects
        let mut obj1 = IndexMap::new();
        obj1.insert("id".to_string(), Value::from(1));
        obj1.insert("name".to_string(), Value::from("Alice"));

        let mut obj2 = IndexMap::new();
        obj2.insert("id".to_string(), Value::from(2));
        obj2.insert("name".to_string(), Value::from("Bob"));

        let original_objects = vec![obj1, obj2];

        // Write to buffer
        let mut buffer = Vec::new();
        write_json_array(&mut buffer, &original_objects)
            .await
            .unwrap();

        // Parse back
        let output = String::from_utf8(buffer).unwrap();
        let parsed_objects = parse_json_string(&output).unwrap();

        // Verify roundtrip
        assert_eq!(parsed_objects.len(), original_objects.len());
        for (parsed, original) in parsed_objects.iter().zip(original_objects.iter()) {
            assert_eq!(parsed, original);
        }
    }
}
