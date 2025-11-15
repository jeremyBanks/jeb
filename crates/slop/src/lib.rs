#![doc = include_str!("../../../README.md")]

// Import json-encoded-binary (currently unused, but available for future use)
#[allow(unused_imports)]
use json_encoded_binary as _;
use {
    async_stream::stream,
    futures::stream::{Stream, StreamExt},
    indexmap::IndexMap,
    serde_json::Value,
    std::cmp::Ordering,
    tokio::io::{AsyncBufRead, AsyncReadExt, BufReader},
    tracing::{debug, info, instrument},
};

// New module for stream/text conversion (work in progress, currently unused)
#[allow(dead_code)]
mod json_stream;

// SQLite integration module
pub mod sqlite;

// CGP-Serde integration module for context-generic serialization
pub mod cgp_serde;

/// Type alias for JSON objects using IndexMap to preserve insertion order
pub type JsonObject = IndexMap<String, Value>;

/// Type alias for JSON parsing errors
pub type JsonError = Box<dyn std::error::Error + Send + Sync>;

/// Options for how to order keys in JSON objects
#[derive(Debug, Clone, PartialEq)]
pub struct KeyOrderOptions {
    /// Apply recursively to nested objects
    pub recursive: bool,
    /// Keys to place first in specified order
    pub first: Vec<String>,
    /// Keys to place last in specified order
    pub last: Vec<String>,
    /// Sort remaining keys alphabetically
    pub sort: bool,
}

impl Default for KeyOrderOptions {
    fn default() -> Self {
        Self {
            recursive: true,
            first: Vec::new(),
            last: Vec::new(),
            sort: true,
        }
    }
}

impl KeyOrderOptions {
    /// Parse key order options from a string
    /// - "false" or "" -> sort=false, recursive=true
    /// - "true" -> sort=true, recursive=true
    /// - JSON array -> Custom with first/sort/last
    pub fn parse(s: &str) -> Result<Self, String> {
        let trimmed = s.trim();

        if trimmed.is_empty() || trimmed == "false" {
            return Ok(Self {
                recursive: true,
                first: Vec::new(),
                last: Vec::new(),
                sort: false,
            });
        }

        if trimmed == "true" {
            return Ok(Self::default());
        }

        // Try to parse as JSON array
        let value: Value = serde_json::from_str(trimmed)
            .map_err(|e| format!("Failed to parse key order spec as JSON: {e}"))?;

        let array = value
            .as_array()
            .ok_or_else(|| "Key order spec must be a boolean or JSON array".to_string())?;

        let mut first = Vec::new();
        let mut sort = true; // default to sorted if not specified
        let mut last = Vec::new();
        let mut in_last = false;
        let mut sort_specified = false;

        for item in array {
            match item {
                Value::String(s) => {
                    if in_last {
                        last.push(s.clone());
                    } else {
                        first.push(s.clone());
                    }
                }
                Value::Bool(b) => {
                    if sort_specified {
                        return Err("Key order spec can only contain one boolean value".to_string());
                    }
                    sort = *b;
                    sort_specified = true;
                    in_last = true;
                }
                _ => {
                    return Err(
                        "Key order spec array must contain only strings and at most one boolean"
                            .to_string(),
                    );
                }
            }
        }

        Ok(Self {
            recursive: true,
            first,
            last,
            sort,
        })
    }

    /// Apply this key ordering to a JSON object
    pub fn apply(&self, obj: &JsonObject) -> JsonObject {
        let mut result = IndexMap::new();

        // Add first keys in specified order
        for key in &self.first {
            if let Some(value) = obj.get(key) {
                let processed_value = if self.recursive {
                    self.apply_to_value(value)
                } else {
                    value.clone()
                };
                result.insert(key.clone(), processed_value);
            }
        }

        // Collect middle keys (not in first or last)
        let first_set: std::collections::HashSet<_> = self.first.iter().collect();
        let last_set: std::collections::HashSet<_> = self.last.iter().collect();

        let mut middle_keys: Vec<_> = obj
            .keys()
            .filter(|k| !first_set.contains(k) && !last_set.contains(k))
            .collect();

        if self.sort {
            middle_keys.sort();
        }

        for key in middle_keys {
            if let Some(value) = obj.get(key) {
                let processed_value = if self.recursive {
                    self.apply_to_value(value)
                } else {
                    value.clone()
                };
                result.insert(key.clone(), processed_value);
            }
        }

        // Add last keys in specified order
        for key in &self.last {
            if let Some(value) = obj.get(key) {
                let processed_value = if self.recursive {
                    self.apply_to_value(value)
                } else {
                    value.clone()
                };
                result.insert(key.clone(), processed_value);
            }
        }

        result
    }

    /// Apply key ordering recursively to a JSON value
    fn apply_to_value(&self, value: &Value) -> Value {
        match value {
            Value::Object(obj) => {
                let index_map: JsonObject =
                    obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
                let reordered = self.apply(&index_map);
                Value::Object(reordered.into_iter().collect())
            }
            Value::Array(arr) => Value::Array(arr.iter().map(|v| self.apply_to_value(v)).collect()),
            _ => value.clone(),
        }
    }
}

// Type alias for backward compatibility
pub type SortSpec = KeyOrderOptions;

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
/// Ordering rules based on ASCII ordering of representative characters:
/// 1. string (") < number (0) < array ([) < false (f) < null (n) < true (t) <
///    object ({)
/// 2. For strings: lexicographic UTF-8 byte comparison
/// 3. For numbers: standard numeric comparison (treating all as f64)
/// 4. For arrays: element-by-element comparison; shorter arrays sort before
///    longer when all compared elements are equal
/// 5. For objects: compared as flattened array [key1, value1, key2, value2,
///    ...], so key order matters
pub fn json_total_order(a: &Value, b: &Value) -> Ordering {
    use Value::*;

    match (a, b) {
        // Strings (lowest)
        (String(a), String(b)) => a.cmp(b),
        (String(_), _) => Ordering::Less,
        (_, String(_)) => Ordering::Greater,

        // Numbers
        (Number(a), Number(b)) => {
            let a_f64 = a.as_f64().unwrap_or(0.0);
            let b_f64 = b.as_f64().unwrap_or(0.0);
            a_f64.partial_cmp(&b_f64).unwrap_or(Ordering::Equal)
        }
        (Number(_), _) => Ordering::Less,
        (_, Number(_)) => Ordering::Greater,

        // Arrays
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

        // False
        (Bool(false), Bool(false)) => Ordering::Equal,
        (Bool(false), _) => Ordering::Less,
        (_, Bool(false)) => Ordering::Greater,

        // Null
        (Null, Null) => Ordering::Equal,
        (Null, _) => Ordering::Less,
        (_, Null) => Ordering::Greater,

        // True
        (Bool(true), Bool(true)) => Ordering::Equal,
        (Bool(true), _) => Ordering::Less,
        (_, Bool(true)) => Ordering::Greater,

        // Objects (highest)
        (Object(a), Object(b)) => {
            // Compare objects as flattened [key1, value1, key2, value2, ...]
            // This means key order matters
            let a_items: Vec<_> = a.iter().collect();
            let b_items: Vec<_> = b.iter().collect();

            for ((a_key, a_val), (b_key, b_val)) in a_items.iter().zip(b_items.iter()) {
                // Compare keys first
                match a_key.cmp(b_key) {
                    Ordering::Equal => {
                        // Keys are equal, compare values
                        match json_total_order(a_val, b_val) {
                            Ordering::Equal => continue,
                            other => return other,
                        }
                    }
                    other => return other,
                }
            }
            // All compared pairs were equal, compare lengths
            a_items.len().cmp(&b_items.len())
        }
    }
}

/// Convert a JSON value to a byte string that preserves the ordering defined by
/// json_total_order.
///
/// This encoding is designed for use as an index key in databases like SQLite.
/// The bytes are ordered such that lexicographic byte comparison matches
/// json_total_order.
///
/// Encoding scheme:
/// - Type prefix byte: " (string), 0 (number), [ (array), f (false), n (null),
///   t (true), { (object)
/// - Strings: null-byte escaped, terminated with \x00\x00
/// - Numbers: order-preserving IEEE 754 encoding (sign-magnitude with bit
///   flipping)
/// - Arrays: recursively encoded elements with \x00\x00 separators, terminated
///   with \x00\x00
/// - Objects: recursively encoded (key, value) pairs with \x00\x00 separators,
///   terminated with \x00\x00
/// - Booleans and null: just the prefix byte
pub fn to_sortable_bytes(value: &Value) -> Vec<u8> {
    let mut bytes = Vec::new();
    encode_value(value, &mut bytes);
    bytes
}

fn encode_value(value: &Value, bytes: &mut Vec<u8>) {
    use Value::*;

    match value {
        // String: prefix '"', null-escaped string, terminator \x00\x00
        String(s) => {
            bytes.push(b'"');
            for byte in s.as_bytes() {
                if *byte == 0x00 {
                    bytes.push(0x00);
                    bytes.push(0x01);
                } else {
                    bytes.push(*byte);
                }
            }
            bytes.push(0x00);
            bytes.push(0x00);
        }

        // Number: prefix '0', then order-preserving float encoding
        Number(n) => {
            bytes.push(b'0');
            let f = n.as_f64().unwrap_or(0.0);
            encode_number(f, bytes);
        }

        // Array: prefix '[', recursively encode elements with separators, terminator
        Array(arr) => {
            bytes.push(b'[');
            for elem in arr {
                encode_value(elem, bytes);
                bytes.push(0x00);
                bytes.push(0x00);
            }
            // Final terminator (empty element signals end)
            bytes.push(0x00);
            bytes.push(0x00);
        }

        // Booleans: just prefix byte
        Bool(false) => {
            bytes.push(b'f');
        }
        Bool(true) => {
            bytes.push(b't');
        }

        // Null: just prefix byte
        Null => {
            bytes.push(b'n');
        }

        // Object: prefix '{', recursively encode (key, value) pairs, terminator
        Object(obj) => {
            bytes.push(b'{');
            for (key, val) in obj {
                // Encode key as string
                encode_value(&Value::String(key.clone()), bytes);
                // Encode value
                encode_value(val, bytes);
                bytes.push(0x00);
                bytes.push(0x00);
            }
            // Final terminator
            bytes.push(0x00);
            bytes.push(0x00);
        }
    }
}

/// Encode a f64 in order-preserving format
///
/// Uses IEEE 754 bit representation with transformations:
/// - If positive: flip sign bit (so positive > negative in byte order)
/// - If negative: flip all bits (so more negative < less negative)
fn encode_number(f: f64, bytes: &mut Vec<u8>) {
    let bits = f.to_bits();
    let transformed = if f >= 0.0 {
        // Positive: flip sign bit (set bit 63)
        bits ^ 0x8000_0000_0000_0000
    } else {
        // Negative: flip all bits
        !bits
    };
    bytes.extend_from_slice(&transformed.to_be_bytes());
}

/// Merge multiple sorted streams into a single sorted stream
///
/// Assumes that each input stream is already sorted according to
/// json_total_order. Performs an n-way merge to produce a single sorted output
/// stream.
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
/// until all elements are processed. This allows correcting out-of-order
/// elements within the buffer window.
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

/// Built-in reduction strategies for consecutive group reduction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReductionStrategy {
    /// Return only the first item from each group
    First,
    /// Return only the last item from each group
    Last,
    /// Return first and last items from each group
    FirstAndLast,
    /// Merge all objects in the group (fails if there are conflicts)
    Merge,
}

impl ReductionStrategy {
    /// Apply this reduction strategy to a group of objects
    pub fn apply(&self, group: Vec<JsonObject>) -> Vec<JsonObject> {
        if group.is_empty() {
            return vec![];
        }

        match self {
            ReductionStrategy::First => vec![group[0].clone()],
            ReductionStrategy::Last => vec![group[group.len() - 1].clone()],
            ReductionStrategy::FirstAndLast => {
                if group.len() == 1 {
                    vec![group[0].clone()]
                } else {
                    vec![group[0].clone(), group[group.len() - 1].clone()]
                }
            }
            ReductionStrategy::Merge => {
                match merge_objects(&group) {
                    Ok(merged) => vec![merged],
                    Err(_) => group, // Return original group if merge fails
                }
            }
        }
    }
}

/// Merge multiple JSON objects into one, returning error if there are conflicts
fn merge_objects(objects: &[JsonObject]) -> Result<JsonObject, String> {
    if objects.is_empty() {
        return Ok(IndexMap::new());
    }

    let mut result = IndexMap::new();

    for obj in objects {
        for (key, value) in obj {
            match result.get(key) {
                None => {
                    result.insert(key.clone(), value.clone());
                }
                Some(existing_value) => {
                    // Try to merge the values
                    match (existing_value, value) {
                        (Value::Object(existing_obj), Value::Object(new_obj)) => {
                            // Recursively merge objects
                            let existing_map: JsonObject = existing_obj
                                .iter()
                                .map(|(k, v)| (k.clone(), v.clone()))
                                .collect();
                            let new_map: JsonObject = new_obj
                                .iter()
                                .map(|(k, v)| (k.clone(), v.clone()))
                                .collect();

                            match merge_objects(&[existing_map, new_map]) {
                                Ok(merged) => {
                                    result.insert(
                                        key.clone(),
                                        Value::Object(merged.into_iter().collect()),
                                    );
                                }
                                Err(_) => {
                                    return Err(format!(
                                        "Conflict when merging nested objects at key '{key}'"
                                    ));
                                }
                            }
                        }
                        (existing, new) if existing == new => {
                            // Values are equal, no conflict
                        }
                        _ => {
                            return Err(format!(
                                "Conflict at key '{key}': cannot merge different values"
                            ));
                        }
                    }
                }
            }
        }
    }

    Ok(result)
}

/// Apply consecutive group reduction to a stream of objects
///
/// Groups consecutive items and reduces each group to output items.
///
/// # Arguments
/// * `objects` - The input objects
/// * `grouping_fn` - Function to determine if two consecutive items belong to
///   the same group
/// * `strategies` - Reduction strategies to apply (in order)
pub fn apply_group_reduction<F>(
    objects: Vec<JsonObject>,
    grouping_fn: F,
    strategies: &[ReductionStrategy],
) -> Vec<JsonObject>
where
    F: Fn(&JsonObject, &JsonObject) -> bool,
{
    if objects.is_empty() {
        return vec![];
    }

    let mut result = Vec::new();
    let mut current_group = vec![objects[0].clone()];

    for i in 1..objects.len() {
        if grouping_fn(&objects[i - 1], &objects[i]) {
            // Same group, add to current group
            current_group.push(objects[i].clone());
        } else {
            // New group, process current group
            let reduced = apply_reduction_strategies(current_group, strategies);
            result.extend(reduced);
            current_group = vec![objects[i].clone()];
        }
    }

    // Process final group
    let reduced = apply_reduction_strategies(current_group, strategies);
    result.extend(reduced);

    result
}

/// Apply a sequence of reduction strategies to a group
fn apply_reduction_strategies(
    mut group: Vec<JsonObject>,
    strategies: &[ReductionStrategy],
) -> Vec<JsonObject> {
    for strategy in strategies {
        group = strategy.apply(group);
    }
    group
}

/// Default grouping function: use total ordering equality
pub fn default_grouping(a: &JsonObject, b: &JsonObject) -> bool {
    let a_val = Value::Object(a.iter().map(|(k, v)| (k.clone(), v.clone())).collect());
    let b_val = Value::Object(b.iter().map(|(k, v)| (k.clone(), v.clone())).collect());
    json_total_order(&a_val, &b_val) == Ordering::Equal
}

/// Re-root a JSON value by making a node at the specified path the new root
///
/// Transforms a JSON tree by making a node at the specified path the new root,
/// nesting original ancestors under the reversed path.
///
/// # Arguments
/// * `json` - The input JSON value
/// * `path` - The path to the node that should become the new root
///
/// # Returns
/// * `Ok(new_root)` - The re-rooted JSON value
/// * `Err(original_json)` - The original JSON if the path doesn't exist or key
///   collision occurs
///
/// # Example
/// ```ignore
/// let json = json!({
///     "container": {
///         "entity": {
///             "id": 1,
///             "name": "foo"
///         },
///         "metadata": "bar"
///     }
/// });
///
/// let rerooted = reroot(json, &["container", "entity"]).unwrap();
/// // Result: { "id": 1, "name": "foo", "container": { "metadata": "bar" } }
/// ```
pub fn reroot(json: Value, path: &[&str]) -> Result<Value, Value> {
    if path.is_empty() {
        return Ok(json);
    }

    // Walk to the target node, collecting ancestors along the way
    let mut current = &json;
    let mut ancestors: Vec<(String, serde_json::Map<String, Value>)> = Vec::new();

    for &key in path.iter() {
        match current {
            Value::Object(obj) => {
                if let Some(next_value) = obj.get(key) {
                    // Store the current level siblings (without the key we're following)
                    let mut siblings = obj.clone();
                    siblings.remove(key);
                    ancestors.push((key.to_string(), siblings));
                    current = next_value;
                } else {
                    // Path doesn't exist
                    return Err(json);
                }
            }
            _ => {
                // Path doesn't exist (tried to descend into non-object)
                return Err(json);
            }
        }
    }

    // Start with the target node as the new root
    let mut new_root = match current {
        Value::Object(obj) => obj.clone(),
        _ => {
            // Can only reroot to an object
            return Err(json);
        }
    };

    // Build up the inverted structure
    // For path ["a", "b", "c"], we create: target with "c" -> { siblings_of_c, "b"
    // -> { siblings_of_b, "a" -> { siblings_of_a } } } But actually, the keys
    // are not path elements but the parent keys For path ["container",
    // "entity"], we create: entity with "container" -> { siblings_of_entity }

    // Process ancestors from deepest to shallowest
    if let Some((last_key, last_siblings)) = ancestors.pop() {
        // Check for collision
        if new_root.contains_key(&last_key) {
            return Err(json);
        }

        // Build nested structure
        let mut nested = last_siblings;

        // Add each remaining ancestor level
        while let Some((key, siblings)) = ancestors.pop() {
            // Check for collision
            for k in nested.keys() {
                if k == &key {
                    return Err(json);
                }
            }

            // Nest the current structure under the parent key
            let nested_value = Value::Object(nested.clone());
            nested = siblings;
            nested.insert(key, nested_value);
        }

        // Add the final nested structure to the new root
        new_root.insert(last_key, Value::Object(nested));
    }

    Ok(Value::Object(new_root))
}

// For backward compatibility during transition - keep the sync version for
// tests
#[doc(hidden)]
pub fn parse_json_stream_sync(input: &str) -> Result<Vec<JsonObject>, JsonError> {
    parse_json_string(input)
}

#[doc(hidden)]
pub fn merge_sorted_streams_sync(streams: Vec<Vec<JsonObject>>) -> Vec<JsonObject> {
    if streams.is_empty() {
        return Vec::new();
    }

    let total_capacity: usize = streams.iter().map(|s| s.len()).sum();
    let mut result = Vec::with_capacity(total_capacity);

    // Track the current position in each stream
    let mut indices: Vec<usize> = vec![0; streams.len()];
    let mut round_robin_ptr = 0;

    loop {
        // Find the next non-exhausted stream starting from round-robin pointer
        let mut attempts = 0;
        while attempts < streams.len() {
            if indices[round_robin_ptr] < streams[round_robin_ptr].len() {
                break; // Found a non-exhausted stream
            }
            round_robin_ptr = (round_robin_ptr + 1) % streams.len();
            attempts += 1;
        }

        if attempts == streams.len() {
            // All streams are exhausted
            break;
        }

        // Start with round-robin head as current minimum
        let mut min_stream_idx = round_robin_ptr;
        let min_pos = indices[min_stream_idx];
        let mut min_obj = Value::Object(
            streams[min_stream_idx][min_pos]
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
        );

        // Rotate through the other stream heads
        for offset in 1..streams.len() {
            let stream_idx = (round_robin_ptr + offset) % streams.len();
            let pos = indices[stream_idx];

            if pos >= streams[stream_idx].len() {
                continue; // This stream is exhausted
            }

            // Convert current JsonObject to Value for comparison
            let current_obj = Value::Object(
                streams[stream_idx][pos]
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect(),
            );

            // If a stream head compares less than the current minimum, it becomes the new
            // minimum Equal or incomparable is not less
            if json_total_order(&current_obj, &min_obj) == Ordering::Less {
                min_stream_idx = stream_idx;
                min_obj = current_obj;
            }
        }

        // Emit the minimum
        let pos = indices[min_stream_idx];
        result.push(streams[min_stream_idx][pos].clone());
        indices[min_stream_idx] += 1;

        // Advance round-robin pointer
        round_robin_ptr = (round_robin_ptr + 1) % streams.len();
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

        // Test type ordering: string < number < array < false < null < true < object
        assert_eq!(json_total_order(&json!(""), &json!(0)), Ordering::Less);
        assert_eq!(json_total_order(&json!(0), &json!([])), Ordering::Less);
        assert_eq!(json_total_order(&json!([]), &json!(false)), Ordering::Less);
        assert_eq!(
            json_total_order(&json!(false), &json!(null)),
            Ordering::Less
        );
        assert_eq!(json_total_order(&json!(null), &json!(true)), Ordering::Less);
        assert_eq!(json_total_order(&json!(true), &json!({})), Ordering::Less);
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
    fn test_key_order_parse_unsorted() {
        let opts = KeyOrderOptions::parse("").unwrap();
        assert!(!opts.sort);
        assert!(opts.recursive);

        let opts = KeyOrderOptions::parse("false").unwrap();
        assert!(!opts.sort);
    }

    #[test]
    fn test_key_order_parse_sorted() {
        let opts = KeyOrderOptions::parse("true").unwrap();
        assert_eq!(opts, KeyOrderOptions::default());
    }

    #[test]
    fn test_key_order_parse_custom_first_only() {
        let opts = KeyOrderOptions::parse(r#"["id","name"]"#).unwrap();
        assert_eq!(opts, KeyOrderOptions {
            recursive: true,
            first: vec!["id".to_string(), "name".to_string()],
            last: vec![],
            sort: true,
        });
    }

    #[test]
    fn test_key_order_parse_custom_with_sorted_middle() {
        let opts = KeyOrderOptions::parse(r#"["id",true,"zip"]"#).unwrap();
        assert_eq!(opts, KeyOrderOptions {
            recursive: true,
            first: vec!["id".to_string()],
            last: vec!["zip".to_string()],
            sort: true,
        });
    }

    #[test]
    fn test_key_order_parse_custom_with_unsorted_middle() {
        let opts = KeyOrderOptions::parse(r#"["id",false,"name"]"#).unwrap();
        assert_eq!(opts, KeyOrderOptions {
            recursive: true,
            first: vec!["id".to_string()],
            last: vec!["name".to_string()],
            sort: false,
        });
    }

    #[test]
    fn test_key_order_parse_invalid() {
        assert!(KeyOrderOptions::parse("invalid").is_err());
        assert!(KeyOrderOptions::parse("123").is_err());
        assert!(KeyOrderOptions::parse(r#"["key",true,false]"#).is_err()); // Two booleans
    }

    #[test]
    fn test_key_order_apply_unsorted() {
        let mut obj = IndexMap::new();
        obj.insert("name".to_string(), Value::from("Alice"));
        obj.insert("id".to_string(), Value::from(1));
        obj.insert("age".to_string(), Value::from(30));

        let opts = KeyOrderOptions {
            recursive: true,
            first: vec![],
            last: vec![],
            sort: false,
        };
        let result = opts.apply(&obj);
        let keys: Vec<_> = result.keys().collect();
        assert_eq!(keys, vec!["name", "id", "age"]);
    }

    #[test]
    fn test_key_order_apply_sorted() {
        let mut obj = IndexMap::new();
        obj.insert("name".to_string(), Value::from("Alice"));
        obj.insert("id".to_string(), Value::from(1));
        obj.insert("age".to_string(), Value::from(30));

        let opts = KeyOrderOptions::default();
        let result = opts.apply(&obj);
        let keys: Vec<_> = result.keys().collect();
        assert_eq!(keys, vec!["age", "id", "name"]);
    }

    #[test]
    fn test_key_order_apply_custom_first() {
        let mut obj = IndexMap::new();
        obj.insert("name".to_string(), Value::from("Alice"));
        obj.insert("id".to_string(), Value::from(1));
        obj.insert("age".to_string(), Value::from(30));
        obj.insert("city".to_string(), Value::from("NYC"));

        let opts = KeyOrderOptions {
            recursive: true,
            first: vec!["id".to_string(), "name".to_string()],
            last: vec![],
            sort: true,
        };

        let result = opts.apply(&obj);
        let keys: Vec<_> = result.keys().collect();
        assert_eq!(keys, vec!["id", "name", "age", "city"]);
    }

    #[test]
    fn test_key_order_apply_custom_with_last() {
        let mut obj = IndexMap::new();
        obj.insert("name".to_string(), Value::from("Alice"));
        obj.insert("id".to_string(), Value::from(1));
        obj.insert("age".to_string(), Value::from(30));
        obj.insert("zip".to_string(), Value::from("12345"));

        let opts = KeyOrderOptions {
            recursive: true,
            first: vec!["id".to_string()],
            last: vec!["zip".to_string()],
            sort: true,
        };

        let result = opts.apply(&obj);
        let keys: Vec<_> = result.keys().collect();
        assert_eq!(keys, vec!["id", "age", "name", "zip"]);
    }

    #[test]
    fn test_key_order_apply_custom_unsorted_middle() {
        let mut obj = IndexMap::new();
        obj.insert("name".to_string(), Value::from("Alice"));
        obj.insert("id".to_string(), Value::from(1));
        obj.insert("age".to_string(), Value::from(30));
        obj.insert("city".to_string(), Value::from("NYC"));

        let opts = KeyOrderOptions {
            recursive: true,
            first: vec!["id".to_string()],
            last: vec!["city".to_string()],
            sort: false,
        };

        let result = opts.apply(&obj);
        let keys: Vec<_> = result.keys().collect();
        // id first, then name and age in original order (name, age), then city
        assert_eq!(keys, vec!["id", "name", "age", "city"]);
    }

    // Binary encoding tests

    #[test]
    fn test_to_sortable_bytes_type_ordering() {
        use serde_json::json;

        // Test that binary encoding preserves type ordering
        let string_bytes = to_sortable_bytes(&json!("test"));
        let number_bytes = to_sortable_bytes(&json!(42));
        let array_bytes = to_sortable_bytes(&json!([]));
        let false_bytes = to_sortable_bytes(&json!(false));
        let null_bytes = to_sortable_bytes(&json!(null));
        let true_bytes = to_sortable_bytes(&json!(true));
        let object_bytes = to_sortable_bytes(&json!({}));

        assert!(string_bytes < number_bytes);
        assert!(number_bytes < array_bytes);
        assert!(array_bytes < false_bytes);
        assert!(false_bytes < null_bytes);
        assert!(null_bytes < true_bytes);
        assert!(true_bytes < object_bytes);
    }

    #[test]
    fn test_to_sortable_bytes_strings() {
        use serde_json::json;

        let a = to_sortable_bytes(&json!("apple"));
        let b = to_sortable_bytes(&json!("banana"));
        let c = to_sortable_bytes(&json!("cherry"));

        assert!(a < b);
        assert!(b < c);
        assert!(a < c);
    }

    #[test]
    fn test_to_sortable_bytes_string_with_null() {
        use serde_json::json;

        // Test that strings with null bytes are handled correctly
        let s1 = "hello";
        let s2 = "hello\x00world";
        let s3 = "hello\x00world\x00";

        let b1 = to_sortable_bytes(&json!(s1));
        let b2 = to_sortable_bytes(&json!(s2));
        let b3 = to_sortable_bytes(&json!(s3));

        // Should sort lexicographically
        assert!(b1 < b2);
        assert!(b2 < b3);
    }

    #[test]
    fn test_to_sortable_bytes_numbers() {
        use serde_json::json;

        let neg_large = to_sortable_bytes(&json!(-1000.0));
        let neg_small = to_sortable_bytes(&json!(-1.0));
        let zero = to_sortable_bytes(&json!(0.0));
        let pos_small = to_sortable_bytes(&json!(1.0));
        let pos_large = to_sortable_bytes(&json!(1000.0));

        assert!(neg_large < neg_small);
        assert!(neg_small < zero);
        assert!(zero < pos_small);
        assert!(pos_small < pos_large);
    }

    #[test]
    fn test_to_sortable_bytes_arrays() {
        use serde_json::json;

        let a1 = to_sortable_bytes(&json!([1]));
        let a2 = to_sortable_bytes(&json!([1, 2]));
        let a3 = to_sortable_bytes(&json!([1, 2, 3]));
        let a4 = to_sortable_bytes(&json!([2]));

        assert!(a1 < a2);
        assert!(a2 < a3);
        assert!(a1 < a4); // [1] < [2] (compares first element)
    }

    #[test]
    fn test_to_sortable_bytes_objects() {
        use serde_json::json;

        let o1 = to_sortable_bytes(&json!({"a": 1}));
        let o2 = to_sortable_bytes(&json!({"a": 2}));
        let o3 = to_sortable_bytes(&json!({"b": 1}));

        assert!(o1 < o2); // Same key, different value
        assert!(o1 < o3); // Different key: "a" < "b"
    }

    #[test]
    fn test_to_sortable_bytes_consistency_with_json_total_order() {
        use serde_json::json;

        // Create a variety of JSON values
        let values = vec![
            json!("aaa"),
            json!("zzz"),
            json!(-100),
            json!(0),
            json!(100),
            json!([]),
            json!([1]),
            json!([1, 2]),
            json!(false),
            json!(null),
            json!(true),
            json!({}),
            json!({"x": 1}),
        ];

        // For every pair of values, binary encoding order should match json_total_order
        for i in 0..values.len() {
            for j in 0..values.len() {
                let bytes_i = to_sortable_bytes(&values[i]);
                let bytes_j = to_sortable_bytes(&values[j]);
                let byte_ordering = bytes_i.cmp(&bytes_j);
                let json_ordering = json_total_order(&values[i], &values[j]);

                assert_eq!(
                    byte_ordering, json_ordering,
                    "Mismatch for {:?} vs {:?}",
                    values[i], values[j]
                );
            }
        }
    }
}

// mod strings; // Commented out - has compilation issues from what-is-this branch
