use async_stream::stream;
use futures::stream::{Stream, StreamExt};
use indexmap::IndexMap;
use serde_json::Value;
use std::cmp::Ordering;
use tokio::io::{AsyncBufRead, AsyncReadExt, BufReader};
use tracing::{debug, info, instrument};

/// Type alias for JSON objects using IndexMap to preserve insertion order
pub type JsonObject = IndexMap<String, Value>;

/// Type alias for JSON parsing errors
pub type JsonError = Box<dyn std::error::Error + Send + Sync>;

/// Specification for how to sort keys in JSON objects
#[derive(Debug, Clone, PartialEq)]
pub enum SortSpec {
    /// Keep original insertion order
    Unsorted,
    /// Sort all keys alphabetically
    Sorted,
    /// Custom sorting: prefix keys, middle section (sorted or not), suffix keys
    Custom {
        prefix: Vec<String>,
        middle_sorted: bool,
        suffix: Vec<String>,
    },
}

impl SortSpec {
    /// Parse a sort specification from a string
    /// - "false" or "" -> Unsorted
    /// - "true" -> Sorted
    /// - JSON array -> Custom with prefix/middle/suffix
    pub fn parse(s: &str) -> Result<Self, String> {
        let trimmed = s.trim();

        if trimmed.is_empty() || trimmed == "false" {
            return Ok(SortSpec::Unsorted);
        }

        if trimmed == "true" {
            return Ok(SortSpec::Sorted);
        }

        // Try to parse as JSON array
        let value: Value = serde_json::from_str(trimmed)
            .map_err(|e| format!("Failed to parse sort spec as JSON: {}", e))?;

        let array = value
            .as_array()
            .ok_or_else(|| "Sort spec must be a boolean or JSON array".to_string())?;

        let mut prefix = Vec::new();
        let mut middle_sorted = true; // default to sorted if not specified
        let mut suffix = Vec::new();
        let mut in_suffix = false;
        let mut middle_specified = false;

        for item in array {
            match item {
                Value::String(s) => {
                    if in_suffix {
                        suffix.push(s.clone());
                    } else {
                        prefix.push(s.clone());
                    }
                }
                Value::Bool(b) => {
                    if middle_specified {
                        return Err("Sort spec can only contain one boolean value".to_string());
                    }
                    middle_sorted = *b;
                    middle_specified = true;
                    in_suffix = true;
                }
                _ => {
                    return Err(
                        "Sort spec array must contain only strings and at most one boolean"
                            .to_string(),
                    );
                }
            }
        }

        Ok(SortSpec::Custom {
            prefix,
            middle_sorted,
            suffix,
        })
    }

    /// Apply this sort specification to reorder a JSON object's keys
    pub fn apply(&self, obj: &JsonObject) -> JsonObject {
        match self {
            SortSpec::Unsorted => obj.clone(),
            SortSpec::Sorted => {
                let mut sorted: Vec<_> = obj.iter().collect();
                sorted.sort_by(|a, b| a.0.cmp(b.0));
                sorted
                    .into_iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect()
            }
            SortSpec::Custom {
                prefix,
                middle_sorted,
                suffix,
            } => {
                let mut result = IndexMap::new();

                // Add prefix keys in specified order
                for key in prefix {
                    if let Some(value) = obj.get(key) {
                        result.insert(key.clone(), value.clone());
                    }
                }

                // Collect middle keys (not in prefix or suffix)
                let prefix_set: std::collections::HashSet<_> = prefix.iter().collect();
                let suffix_set: std::collections::HashSet<_> = suffix.iter().collect();

                let mut middle_keys: Vec<_> = obj
                    .keys()
                    .filter(|k| !prefix_set.contains(k) && !suffix_set.contains(k))
                    .collect();

                if *middle_sorted {
                    middle_keys.sort();
                }

                for key in middle_keys {
                    if let Some(value) = obj.get(key) {
                        result.insert(key.clone(), value.clone());
                    }
                }

                // Add suffix keys in specified order
                for key in suffix {
                    if let Some(value) = obj.get(key) {
                        result.insert(key.clone(), value.clone());
                    }
                }

                result
            }
        }
    }
}

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

/// Implement total ordering for JSON values.
///
/// Ordering rules:
/// 1. null < boolean < number < string < array < object
/// 2. For booleans: false < true
/// 3. For numbers: standard numeric comparison (treating all as f64)
/// 4. For strings: lexicographic comparison
/// 5. For arrays: lexicographic comparison element-by-element
/// 6. For objects: compare by sorted keys, then by values
pub fn json_total_order(a: &Value, b: &Value) -> Ordering {
    use Value::*;

    match (a, b) {
        (Null, Null) => Ordering::Equal,
        (Null, _) => Ordering::Less,
        (_, Null) => Ordering::Greater,

        (Bool(a), Bool(b)) => a.cmp(b),
        (Bool(_), _) => Ordering::Less,
        (_, Bool(_)) => Ordering::Greater,

        (Number(a), Number(b)) => {
            let a_f64 = a.as_f64().unwrap_or(0.0);
            let b_f64 = b.as_f64().unwrap_or(0.0);
            a_f64.partial_cmp(&b_f64).unwrap_or(Ordering::Equal)
        }
        (Number(_), _) => Ordering::Less,
        (_, Number(_)) => Ordering::Greater,

        (String(a), String(b)) => a.cmp(b),
        (String(_), _) => Ordering::Less,
        (_, String(_)) => Ordering::Greater,

        (Array(a), Array(b)) => {
            for (a_elem, b_elem) in a.iter().zip(b.iter()) {
                match json_total_order(a_elem, b_elem) {
                    Ordering::Equal => continue,
                    other => return other,
                }
            }
            a.len().cmp(&b.len())
        }
        (Array(_), _) => Ordering::Less,
        (_, Array(_)) => Ordering::Greater,

        (Object(a), Object(b)) => {
            // Compare objects by their keys first, then by values
            let a_keys: Vec<_> = a.keys().collect();
            let b_keys: Vec<_> = b.keys().collect();

            match a_keys.cmp(&b_keys) {
                Ordering::Equal => {
                    // Keys are the same, compare values in key order
                    for key in a_keys {
                        let a_val = &a[key];
                        let b_val = &b[key];
                        match json_total_order(a_val, b_val) {
                            Ordering::Equal => continue,
                            other => return other,
                        }
                    }
                    Ordering::Equal
                }
                other => other,
            }
        }
    }
}

/// Merge multiple sorted streams into a single sorted stream
///
/// Assumes that each input stream is already sorted according to json_total_order.
/// Performs an n-way merge to produce a single sorted output stream.
pub fn merge_sorted_streams<S>(streams: Vec<S>) -> impl Stream<Item = JsonObject>
where
    S: Stream<Item = Result<JsonObject, JsonError>> + Unpin + Send + 'static,
{
    stream! {
        // Collect all items from all streams first
        // TODO: Implement true streaming n-way merge with peekable streams
        let mut all_objects = Vec::new();
        let stream_count = streams.len();

        for mut stream in streams {
            while let Some(result) = stream.next().await {
                match result {
                    Ok(obj) => all_objects.push(obj),
                    Err(e) => {
                        debug!("Error in stream: {}", e);
                        // Skip errors for now
                    }
                }
            }
        }

        // Sort all objects
        all_objects.sort_by(|a, b| {
            let a_val = Value::Object(a.iter().map(|(k, v)| (k.clone(), v.clone())).collect());
            let b_val = Value::Object(b.iter().map(|(k, v)| (k.clone(), v.clone())).collect());
            json_total_order(&a_val, &b_val)
        });

        info!("Merged {} total objects from {} streams", all_objects.len(), stream_count);

        for obj in all_objects {
            yield obj;
        }
    }
}

/// Apply a sorting buffer to correct slight disorder in the stream
///
/// This function maintains a sliding window buffer of size `buffer_size`.
/// It fills the buffer, sorts it, outputs the smallest element, and continues
/// until all elements are processed. This allows correcting out-of-order elements
/// within the buffer window.
///
/// If buffer_size is 0, returns the stream unchanged.
pub fn apply_sort_buffer<S>(stream: S, buffer_size: usize) -> impl Stream<Item = JsonObject>
where
    S: Stream<Item = JsonObject> + Unpin + Send + 'static,
{
    stream! {
        if buffer_size == 0 {
            debug!("Sort buffer disabled (size=0), passing through unchanged");
            let mut stream = Box::pin(stream);
            while let Some(obj) = stream.next().await {
                yield obj;
            }
            return;
        }

        // Collect all items for now - TODO: implement true sliding window
        let mut objects: Vec<JsonObject> = Vec::new();
        let mut stream = Box::pin(stream);

        while let Some(obj) = stream.next().await {
            objects.push(obj);
        }

        if objects.is_empty() {
            return;
        }

        let mut result = Vec::with_capacity(objects.len());
        let mut buffer: Vec<JsonObject> = Vec::with_capacity(buffer_size);
        let mut input_iter = objects.into_iter();

        // Fill the initial buffer
        for obj in input_iter.by_ref().take(buffer_size) {
            buffer.push(obj);
        }

        // Sort the initial buffer
        buffer.sort_by(|a, b| {
            let a_val = Value::Object(a.iter().map(|(k, v)| (k.clone(), v.clone())).collect());
            let b_val = Value::Object(b.iter().map(|(k, v)| (k.clone(), v.clone())).collect());
            json_total_order(&a_val, &b_val)
        });

        // Process remaining objects
        for obj in input_iter {
            // Output the smallest element from buffer
            if !buffer.is_empty() {
                result.push(buffer.remove(0));
            }

            // Insert new object in sorted position
            let obj_val = Value::Object(obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect());
            let insert_pos = buffer
                .iter()
                .position(|b| {
                    let b_val = Value::Object(b.iter().map(|(k, v)| (k.clone(), v.clone())).collect());
                    json_total_order(&obj_val, &b_val) == Ordering::Less
                })
                .unwrap_or(buffer.len());
            buffer.insert(insert_pos, obj);
        }

        // Output remaining buffer contents (already sorted)
        result.extend(buffer);

        info!("Applied sort buffer of size {} to {} objects", buffer_size, result.len());

        for obj in result {
            yield obj;
        }
    }
}

// For backward compatibility during transition - keep the sync version for tests
#[doc(hidden)]
pub fn parse_json_stream_sync(input: &str) -> Result<Vec<JsonObject>, JsonError> {
    parse_json_string(input)
}

#[doc(hidden)]
pub fn merge_sorted_streams_sync(streams: Vec<Vec<JsonObject>>) -> Vec<JsonObject> {
    let total_capacity: usize = streams.iter().map(|s| s.len()).sum();
    let mut result = Vec::with_capacity(total_capacity);

    // Track the current position in each stream
    let mut indices: Vec<usize> = vec![0; streams.len()];

    loop {
        // Find the smallest element among all stream heads
        let mut min_stream_idx: Option<usize> = None;

        for (stream_idx, stream) in streams.iter().enumerate() {
            let pos = indices[stream_idx];
            if pos >= stream.len() {
                continue; // This stream is exhausted
            }

            // Convert current JsonObject to Value for comparison
            let current_obj = Value::Object(
                stream[pos]
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect(),
            );

            match min_stream_idx {
                None => {
                    min_stream_idx = Some(stream_idx);
                }
                Some(min_idx) => {
                    // Compare with current minimum
                    let min_pos = indices[min_idx];
                    let min_obj = Value::Object(
                        streams[min_idx][min_pos]
                            .iter()
                            .map(|(k, v)| (k.clone(), v.clone()))
                            .collect(),
                    );

                    if json_total_order(&current_obj, &min_obj) == Ordering::Less {
                        min_stream_idx = Some(stream_idx);
                    }
                }
            }
        }

        // If no minimum was found, all streams are exhausted
        if let Some(stream_idx) = min_stream_idx {
            let pos = indices[stream_idx];
            result.push(streams[stream_idx][pos].clone());
            indices[stream_idx] += 1;
        } else {
            break;
        }
    }

    info!(
        "Merged {} streams into {} total objects",
        streams.len(),
        result.len()
    );
    result
}

#[doc(hidden)]
pub fn apply_sort_buffer_sync(objects: Vec<JsonObject>, buffer_size: usize) -> Vec<JsonObject> {
    if buffer_size == 0 {
        debug!("Sort buffer disabled (size=0), returning objects unchanged");
        return objects;
    }

    if objects.is_empty() {
        return objects;
    }

    let mut result = Vec::with_capacity(objects.len());
    let mut buffer: Vec<JsonObject> = Vec::with_capacity(buffer_size);
    let mut input_iter = objects.into_iter();

    // Fill the initial buffer
    for obj in input_iter.by_ref().take(buffer_size) {
        buffer.push(obj);
    }

    // Sort the initial buffer
    buffer.sort_by(|a, b| {
        let a_val = Value::Object(a.iter().map(|(k, v)| (k.clone(), v.clone())).collect());
        let b_val = Value::Object(b.iter().map(|(k, v)| (k.clone(), v.clone())).collect());
        json_total_order(&a_val, &b_val)
    });

    // Process remaining objects
    for obj in input_iter {
        // Output the smallest element from buffer
        if !buffer.is_empty() {
            result.push(buffer.remove(0));
        }

        // Insert new object in sorted position
        let obj_val = Value::Object(obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect());
        let insert_pos = buffer
            .iter()
            .position(|b| {
                let b_val = Value::Object(b.iter().map(|(k, v)| (k.clone(), v.clone())).collect());
                json_total_order(&obj_val, &b_val) == Ordering::Less
            })
            .unwrap_or(buffer.len());
        buffer.insert(insert_pos, obj);
    }

    // Output remaining buffer contents (already sorted)
    result.extend(buffer);

    info!(
        "Applied sort buffer of size {} to {} objects",
        buffer_size,
        result.len()
    );
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // JSON stream parsing tests

    #[test]
    fn test_parse_json_lines() {
        let input = r#"{"id": 1, "name": "Alice"}
{"id": 2, "name": "Bob"}
{"id": 3, "name": "Charlie"}"#;

        let objects = parse_json_stream_sync(input).unwrap();
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

        let objects = parse_json_stream_sync(input).unwrap();
        assert_eq!(objects.len(), 3);
        assert_eq!(objects[0].get("id").unwrap(), &Value::from(1));
        assert_eq!(objects[1].get("id").unwrap(), &Value::from(2));
        assert_eq!(objects[2].get("id").unwrap(), &Value::from(3));
    }

    #[test]
    fn test_parse_concatenated_json() {
        let input =
            r#"{"id": 1, "name": "Alice"}{"id": 2, "name": "Bob"}{"id": 3, "name": "Charlie"}"#;

        let objects = parse_json_stream_sync(input).unwrap();
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

        let objects = parse_json_stream_sync(input).unwrap();
        assert_eq!(objects.len(), 3);
        assert_eq!(objects[0].get("id").unwrap(), &Value::from(1));
        assert_eq!(objects[1].get("id").unwrap(), &Value::from(2));
        assert_eq!(objects[2].get("id").unwrap(), &Value::from(3));
    }

    #[test]
    fn test_parse_nested_objects() {
        let input = r#"{"id": 1, "data": {"nested": "value"}}
{"id": 2, "data": {"nested": {"deep": "value"}}}"#;

        let objects = parse_json_stream_sync(input).unwrap();
        assert_eq!(objects.len(), 2);
        assert_eq!(objects[0].get("id").unwrap(), &Value::from(1));
        assert!(objects[0].get("data").unwrap().is_object());
        assert_eq!(objects[1].get("id").unwrap(), &Value::from(2));
    }

    #[test]
    fn test_parse_with_escaped_braces() {
        let input = r#"{"id": 1, "text": "This has a } in it"}
{"id": 2, "text": "And this has a { in it"}"#;

        let objects = parse_json_stream_sync(input).unwrap();
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
        let objects = parse_json_stream_sync(input).unwrap();
        assert_eq!(objects.len(), 0);
    }

    #[test]
    fn test_parse_no_json_objects() {
        let input = "This is just plain text with no JSON objects";
        let objects = parse_json_stream_sync(input).unwrap();
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

    // JSON total ordering tests

    #[test]
    fn test_json_total_order_types() {
        use serde_json::json;

        // Test type ordering: null < bool < number < string < array < object
        assert_eq!(
            json_total_order(&json!(null), &json!(false)),
            Ordering::Less
        );
        assert_eq!(json_total_order(&json!(false), &json!(0)), Ordering::Less);
        assert_eq!(json_total_order(&json!(0), &json!("")), Ordering::Less);
        assert_eq!(json_total_order(&json!(""), &json!([])), Ordering::Less);
        assert_eq!(json_total_order(&json!([]), &json!({})), Ordering::Less);
    }

    #[test]
    fn test_json_total_order_booleans() {
        use serde_json::json;

        assert_eq!(
            json_total_order(&json!(false), &json!(true)),
            Ordering::Less
        );
        assert_eq!(
            json_total_order(&json!(true), &json!(false)),
            Ordering::Greater
        );
        assert_eq!(
            json_total_order(&json!(true), &json!(true)),
            Ordering::Equal
        );
    }

    #[test]
    fn test_json_total_order_numbers() {
        use serde_json::json;

        assert_eq!(json_total_order(&json!(1), &json!(2)), Ordering::Less);
        assert_eq!(json_total_order(&json!(2.5), &json!(2.5)), Ordering::Equal);
        assert_eq!(json_total_order(&json!(10), &json!(5)), Ordering::Greater);
    }

    #[test]
    fn test_json_total_order_strings() {
        use serde_json::json;

        assert_eq!(
            json_total_order(&json!("apple"), &json!("banana")),
            Ordering::Less
        );
        assert_eq!(
            json_total_order(&json!("test"), &json!("test")),
            Ordering::Equal
        );
    }

    #[test]
    fn test_json_total_order_arrays() {
        use serde_json::json;

        assert_eq!(
            json_total_order(&json!([1, 2]), &json!([1, 3])),
            Ordering::Less
        );
        assert_eq!(
            json_total_order(&json!([1, 2]), &json!([1, 2])),
            Ordering::Equal
        );
        assert_eq!(
            json_total_order(&json!([1, 2]), &json!([1])),
            Ordering::Greater
        );
    }

    #[test]
    fn test_json_total_order_objects() {
        use serde_json::json;

        assert_eq!(
            json_total_order(&json!({"a": 1}), &json!({"a": 2})),
            Ordering::Less
        );
        assert_eq!(
            json_total_order(&json!({"a": 1, "b": 2}), &json!({"a": 1, "b": 2})),
            Ordering::Equal
        );
    }

    // Merge sorted streams tests

    #[test]
    fn test_merge_sorted_streams_simple() {
        let stream1 = parse_json_stream_sync(r#"{"id": 1}{"id": 3}{"id": 5}"#).unwrap();
        let stream2 = parse_json_stream_sync(r#"{"id": 2}{"id": 4}{"id": 6}"#).unwrap();

        let merged = merge_sorted_streams_sync(vec![stream1, stream2]);

        assert_eq!(merged.len(), 6);
        for (i, obj) in merged.iter().enumerate() {
            assert_eq!(obj.get("id").unwrap(), &Value::from(i + 1));
        }
    }

    #[test]
    fn test_merge_sorted_streams_empty() {
        let stream1: Vec<JsonObject> = vec![];
        let stream2 = parse_json_stream_sync(r#"{"id": 1}"#).unwrap();

        let merged = merge_sorted_streams_sync(vec![stream1, stream2]);
        assert_eq!(merged.len(), 1);
    }

    #[test]
    fn test_merge_sorted_streams_single() {
        let stream1 = parse_json_stream_sync(r#"{"id": 1}{"id": 2}{"id": 3}"#).unwrap();

        let merged = merge_sorted_streams_sync(vec![stream1]);
        assert_eq!(merged.len(), 3);
    }

    // Sort buffer tests

    #[test]
    fn test_apply_sort_buffer_disabled() {
        let objects = parse_json_stream_sync(r#"{"id": 3}{"id": 1}{"id": 2}"#).unwrap();
        let result = apply_sort_buffer_sync(objects.clone(), 0);

        // Should return unchanged when buffer size is 0
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].get("id").unwrap(), &Value::from(3));
        assert_eq!(result[1].get("id").unwrap(), &Value::from(1));
        assert_eq!(result[2].get("id").unwrap(), &Value::from(2));
    }

    #[test]
    fn test_apply_sort_buffer_small_disorder() {
        // Slightly out of order - within buffer window
        let objects = parse_json_stream_sync(r#"{"id": 1}{"id": 3}{"id": 2}{"id": 4}"#).unwrap();
        let result = apply_sort_buffer_sync(objects, 3);

        // Should be sorted
        assert_eq!(result.len(), 4);
        for (i, obj) in result.iter().enumerate() {
            assert_eq!(obj.get("id").unwrap(), &Value::from(i + 1));
        }
    }

    #[test]
    fn test_apply_sort_buffer_already_sorted() {
        let objects = parse_json_stream_sync(r#"{"id": 1}{"id": 2}{"id": 3}{"id": 4}"#).unwrap();
        let result = apply_sort_buffer_sync(objects, 3);

        // Should remain sorted
        assert_eq!(result.len(), 4);
        for (i, obj) in result.iter().enumerate() {
            assert_eq!(obj.get("id").unwrap(), &Value::from(i + 1));
        }
    }

    #[test]
    fn test_apply_sort_buffer_large_buffer() {
        // Buffer larger than input
        let objects = parse_json_stream_sync(r#"{"id": 3}{"id": 1}{"id": 2}"#).unwrap();
        let result = apply_sort_buffer_sync(objects, 10);

        // Should fully sort
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].get("id").unwrap(), &Value::from(1));
        assert_eq!(result[1].get("id").unwrap(), &Value::from(2));
        assert_eq!(result[2].get("id").unwrap(), &Value::from(3));
    }

    #[test]
    fn test_apply_sort_buffer_empty() {
        let objects: Vec<JsonObject> = vec![];
        let result = apply_sort_buffer_sync(objects, 3);

        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_apply_sort_buffer_complex() {
        // More complex disorder pattern
        let objects =
            parse_json_stream_sync(r#"{"id": 1}{"id": 2}{"id": 5}{"id": 3}{"id": 4}{"id": 6}"#)
                .unwrap();
        let result = apply_sort_buffer_sync(objects, 3);

        // With buffer size 3, should correct the disorder
        assert_eq!(result.len(), 6);
        for (i, obj) in result.iter().enumerate() {
            assert_eq!(obj.get("id").unwrap(), &Value::from(i + 1));
        }
    }

    #[test]
    fn test_sort_spec_parse_unsorted() {
        assert_eq!(SortSpec::parse("").unwrap(), SortSpec::Unsorted);
        assert_eq!(SortSpec::parse("false").unwrap(), SortSpec::Unsorted);
    }

    #[test]
    fn test_sort_spec_parse_sorted() {
        assert_eq!(SortSpec::parse("true").unwrap(), SortSpec::Sorted);
    }

    #[test]
    fn test_sort_spec_parse_custom_prefix_only() {
        let spec = SortSpec::parse(r#"["id","name"]"#).unwrap();
        assert_eq!(
            spec,
            SortSpec::Custom {
                prefix: vec!["id".to_string(), "name".to_string()],
                middle_sorted: true,
                suffix: vec![],
            }
        );
    }

    #[test]
    fn test_sort_spec_parse_custom_with_sorted_middle() {
        let spec = SortSpec::parse(r#"["id",true,"zip"]"#).unwrap();
        assert_eq!(
            spec,
            SortSpec::Custom {
                prefix: vec!["id".to_string()],
                middle_sorted: true,
                suffix: vec!["zip".to_string()],
            }
        );
    }

    #[test]
    fn test_sort_spec_parse_custom_with_unsorted_middle() {
        let spec = SortSpec::parse(r#"["id",false,"name"]"#).unwrap();
        assert_eq!(
            spec,
            SortSpec::Custom {
                prefix: vec!["id".to_string()],
                middle_sorted: false,
                suffix: vec!["name".to_string()],
            }
        );
    }

    #[test]
    fn test_sort_spec_parse_invalid() {
        assert!(SortSpec::parse("invalid").is_err());
        assert!(SortSpec::parse("123").is_err());
        assert!(SortSpec::parse(r#"["key",true,false]"#).is_err()); // Two booleans
    }

    #[test]
    fn test_sort_spec_apply_unsorted() {
        let mut obj = IndexMap::new();
        obj.insert("name".to_string(), Value::from("Alice"));
        obj.insert("id".to_string(), Value::from(1));
        obj.insert("age".to_string(), Value::from(30));

        let result = SortSpec::Unsorted.apply(&obj);
        let keys: Vec<_> = result.keys().collect();
        assert_eq!(keys, vec!["name", "id", "age"]);
    }

    #[test]
    fn test_sort_spec_apply_sorted() {
        let mut obj = IndexMap::new();
        obj.insert("name".to_string(), Value::from("Alice"));
        obj.insert("id".to_string(), Value::from(1));
        obj.insert("age".to_string(), Value::from(30));

        let result = SortSpec::Sorted.apply(&obj);
        let keys: Vec<_> = result.keys().collect();
        assert_eq!(keys, vec!["age", "id", "name"]);
    }

    #[test]
    fn test_sort_spec_apply_custom_prefix() {
        let mut obj = IndexMap::new();
        obj.insert("name".to_string(), Value::from("Alice"));
        obj.insert("id".to_string(), Value::from(1));
        obj.insert("age".to_string(), Value::from(30));
        obj.insert("city".to_string(), Value::from("NYC"));

        let spec = SortSpec::Custom {
            prefix: vec!["id".to_string(), "name".to_string()],
            middle_sorted: true,
            suffix: vec![],
        };

        let result = spec.apply(&obj);
        let keys: Vec<_> = result.keys().collect();
        assert_eq!(keys, vec!["id", "name", "age", "city"]);
    }

    #[test]
    fn test_sort_spec_apply_custom_with_suffix() {
        let mut obj = IndexMap::new();
        obj.insert("name".to_string(), Value::from("Alice"));
        obj.insert("id".to_string(), Value::from(1));
        obj.insert("age".to_string(), Value::from(30));
        obj.insert("zip".to_string(), Value::from("12345"));

        let spec = SortSpec::Custom {
            prefix: vec!["id".to_string()],
            middle_sorted: true,
            suffix: vec!["zip".to_string()],
        };

        let result = spec.apply(&obj);
        let keys: Vec<_> = result.keys().collect();
        assert_eq!(keys, vec!["id", "age", "name", "zip"]);
    }

    #[test]
    fn test_sort_spec_apply_custom_unsorted_middle() {
        let mut obj = IndexMap::new();
        obj.insert("name".to_string(), Value::from("Alice"));
        obj.insert("id".to_string(), Value::from(1));
        obj.insert("age".to_string(), Value::from(30));
        obj.insert("city".to_string(), Value::from("NYC"));

        let spec = SortSpec::Custom {
            prefix: vec!["id".to_string()],
            middle_sorted: false,
            suffix: vec!["city".to_string()],
        };

        let result = spec.apply(&obj);
        let keys: Vec<_> = result.keys().collect();
        // id first, then name and age in original order (name, age), then city
        assert_eq!(keys, vec!["id", "name", "age", "city"]);
    }
}
