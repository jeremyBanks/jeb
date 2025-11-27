// Node implementations for the pipeline system

use crate::{
    model::{CoercionKind, ErrorValue, StreamItem, Structured, Warning},
    pipeline::{ExecutionResult, Node, NodeError, Stream},
};
use data_encoding::BASE64;
use std::io::{Read, Write};

// MARK: Source Nodes

/// Reads from standard input
pub struct StdinNode;

impl Node for StdinNode {
    fn name(&self) -> &str {
        "stdin"
    }

    fn input_arity(&self) -> (usize, Option<usize>) {
        (0, Some(0)) // No inputs
    }

    fn output_count(&self) -> usize {
        1
    }

    fn execute(
        &self,
        _inputs: Vec<Stream>,
        _node_index: usize,
    ) -> Result<ExecutionResult, NodeError> {
        let mut buffer = Vec::new();
        std::io::stdin().read_to_end(&mut buffer)?;

        Ok(ExecutionResult {
            outputs: vec![vec![StreamItem::Bytes(buffer)]],
            errors: Vec::new(),
            warnings: Vec::new(),
        })
    }
}

/// Reads from a file
pub struct FileSourceNode {
    path: String,
}

impl FileSourceNode {
    pub fn new(path: String) -> Self {
        Self { path }
    }
}

impl Node for FileSourceNode {
    fn name(&self) -> &str {
        "file"
    }

    fn input_arity(&self) -> (usize, Option<usize>) {
        (0, Some(0))
    }

    fn output_count(&self) -> usize {
        1
    }

    fn execute(
        &self,
        _inputs: Vec<Stream>,
        _node_index: usize,
    ) -> Result<ExecutionResult, NodeError> {
        let data = std::fs::read(&self.path)?;

        Ok(ExecutionResult {
            outputs: vec![vec![StreamItem::Bytes(data)]],
            errors: Vec::new(),
            warnings: Vec::new(),
        })
    }
}

// MARK: Sink Nodes

/// Writes to standard output
pub struct StdoutNode;

impl Node for StdoutNode {
    fn name(&self) -> &str {
        "stdout"
    }

    fn input_arity(&self) -> (usize, Option<usize>) {
        (1, Some(1))
    }

    fn output_count(&self) -> usize {
        0 // Sink has no outputs
    }

    fn execute(
        &self,
        inputs: Vec<Stream>,
        node_index: usize,
    ) -> Result<ExecutionResult, NodeError> {
        if inputs.len() != 1 {
            return Err(NodeError::InvalidInputCount {
                expected: "1".to_string(),
                got: inputs.len(),
            });
        }

        let mut stdout = std::io::stdout();
        let mut warnings = Vec::new();

        for item in &inputs[0] {
            match item {
                StreamItem::Text(s) => stdout.write_all(s.as_bytes())?,
                StreamItem::Bytes(b) => stdout.write_all(b)?,
                StreamItem::Structured(s) => {
                    // Coerce structured to JSON text and emit warning
                    if let Some(json_value) = structured_to_json_value(s) {
                        if let Ok(json_string) = serde_json::to_string(&json_value) {
                            stdout.write_all(json_string.as_bytes())?;
                            stdout.write_all(b"\n")?;
                            warnings.push(Warning {
                                node_index,
                                message: "Coerced Structured item to JSON text for stdout"
                                    .to_string(),
                                coercion: CoercionKind::StructuredToText,
                            });
                        }
                    }
                }
            }
        }

        stdout.flush()?;

        Ok(ExecutionResult {
            outputs: Vec::new(),
            errors: Vec::new(),
            warnings,
        })
    }
}

/// Writes to standard error
pub struct StderrNode;

impl Node for StderrNode {
    fn name(&self) -> &str {
        "stderr"
    }

    fn input_arity(&self) -> (usize, Option<usize>) {
        (1, Some(1))
    }

    fn output_count(&self) -> usize {
        0
    }

    fn execute(
        &self,
        inputs: Vec<Stream>,
        node_index: usize,
    ) -> Result<ExecutionResult, NodeError> {
        if inputs.len() != 1 {
            return Err(NodeError::InvalidInputCount {
                expected: "1".to_string(),
                got: inputs.len(),
            });
        }

        let mut stderr = std::io::stderr();
        let mut warnings = Vec::new();

        for item in &inputs[0] {
            match item {
                StreamItem::Text(s) => stderr.write_all(s.as_bytes())?,
                StreamItem::Bytes(b) => stderr.write_all(b)?,
                StreamItem::Structured(s) => {
                    // Coerce structured to JSON text and emit warning
                    if let Some(json_value) = structured_to_json_value(s) {
                        if let Ok(json_string) = serde_json::to_string(&json_value) {
                            stderr.write_all(json_string.as_bytes())?;
                            stderr.write_all(b"\n")?;
                            warnings.push(Warning {
                                node_index,
                                message: "Coerced Structured item to JSON text for stderr"
                                    .to_string(),
                                coercion: CoercionKind::StructuredToText,
                            });
                        }
                    }
                }
            }
        }

        stderr.flush()?;

        Ok(ExecutionResult {
            outputs: Vec::new(),
            errors: Vec::new(),
            warnings,
        })
    }
}

// MARK: Parser Nodes

/// Parses JSON from text or bytes
pub struct ParseJsonNode;

impl Node for ParseJsonNode {
    fn name(&self) -> &str {
        "parse-json"
    }

    fn input_arity(&self) -> (usize, Option<usize>) {
        (1, Some(1))
    }

    fn output_count(&self) -> usize {
        1
    }

    fn execute(
        &self,
        inputs: Vec<Stream>,
        node_index: usize,
    ) -> Result<ExecutionResult, NodeError> {
        if inputs.len() != 1 {
            return Err(NodeError::InvalidInputCount {
                expected: "1".to_string(),
                got: inputs.len(),
            });
        }

        let mut output = Vec::new();
        let mut errors = Vec::new();

        for item in &inputs[0] {
            let bytes = match item {
                StreamItem::Bytes(b) => b.clone(),
                StreamItem::Text(s) => s.as_bytes().to_vec(),
                StreamItem::Structured(_) => {
                    // Already structured, pass through
                    output.push(item.clone());
                    continue;
                }
            };

            // Try to parse as JSON
            match serde_json::from_slice::<serde_json::Value>(&bytes) {
                Ok(value) => {
                    // Convert serde_json::Value to our Structured type
                    if let Some(structured) = json_value_to_structured(&value) {
                        output.push(StreamItem::Structured(structured));
                    } else {
                        errors.push(ErrorValue {
                            node_index,
                            message: "Failed to convert JSON value".to_string(),
                            context: None,
                        });
                    }
                }
                Err(e) => {
                    errors.push(ErrorValue {
                        node_index,
                        message: format!("JSON parse error: {}", e),
                        context: None,
                    });
                }
            }
        }

        Ok(ExecutionResult {
            outputs: vec![output],
            errors,
            warnings: Vec::new(),
        })
    }
}

/// Decodes base64 data
pub struct FromBase64Node;

impl Node for FromBase64Node {
    fn name(&self) -> &str {
        "from-base64"
    }

    fn input_arity(&self) -> (usize, Option<usize>) {
        (1, Some(1))
    }

    fn output_count(&self) -> usize {
        1
    }

    fn execute(
        &self,
        inputs: Vec<Stream>,
        node_index: usize,
    ) -> Result<ExecutionResult, NodeError> {
        if inputs.len() != 1 {
            return Err(NodeError::InvalidInputCount {
                expected: "1".to_string(),
                got: inputs.len(),
            });
        }

        let mut output = Vec::new();
        let mut errors = Vec::new();

        for item in &inputs[0] {
            let input_bytes = match item {
                StreamItem::Text(s) => s.trim().as_bytes().to_vec(),
                StreamItem::Bytes(b) => {
                    // Trim whitespace from bytes
                    let s = String::from_utf8_lossy(b);
                    s.trim().as_bytes().to_vec()
                }
                StreamItem::Structured(_) => {
                    // Pass through structured items
                    output.push(item.clone());
                    continue;
                }
            };

            match BASE64.decode(&input_bytes) {
                Ok(decoded) => {
                    output.push(StreamItem::Bytes(decoded));
                }
                Err(e) => {
                    errors.push(ErrorValue {
                        node_index,
                        message: format!("Base64 decode error: {}", e),
                        context: None,
                    });
                }
            }
        }

        Ok(ExecutionResult {
            outputs: vec![output],
            errors,
            warnings: Vec::new(),
        })
    }
}

// MARK: Serializer Nodes

/// Serializes structured data to JSON
pub struct ToJsonNode;

impl Node for ToJsonNode {
    fn name(&self) -> &str {
        "to-json"
    }

    fn input_arity(&self) -> (usize, Option<usize>) {
        (1, Some(1))
    }

    fn output_count(&self) -> usize {
        1
    }

    fn execute(
        &self,
        inputs: Vec<Stream>,
        _node_index: usize,
    ) -> Result<ExecutionResult, NodeError> {
        if inputs.len() != 1 {
            return Err(NodeError::InvalidInputCount {
                expected: "1".to_string(),
                got: inputs.len(),
            });
        }

        let mut output = Vec::new();

        for item in &inputs[0] {
            match item {
                StreamItem::Structured(s) => {
                    // Convert to JSON and output as Text
                    if let Some(json_value) = structured_to_json_value(s) {
                        if let Ok(json_string) = serde_json::to_string(&json_value) {
                            output.push(StreamItem::Text(json_string + "\n"));
                        }
                    }
                }
                _ => {
                    // Pass through non-structured items
                    output.push(item.clone());
                }
            }
        }

        Ok(ExecutionResult {
            outputs: vec![output],
            errors: Vec::new(),
            warnings: Vec::new(),
        })
    }
}

/// Encodes data as base64
pub struct ToBase64Node;

impl Node for ToBase64Node {
    fn name(&self) -> &str {
        "to-base64"
    }

    fn input_arity(&self) -> (usize, Option<usize>) {
        (1, Some(1))
    }

    fn output_count(&self) -> usize {
        1
    }

    fn execute(
        &self,
        inputs: Vec<Stream>,
        _node_index: usize,
    ) -> Result<ExecutionResult, NodeError> {
        if inputs.len() != 1 {
            return Err(NodeError::InvalidInputCount {
                expected: "1".to_string(),
                got: inputs.len(),
            });
        }

        let mut output = Vec::new();

        for item in &inputs[0] {
            let input_bytes = match item {
                StreamItem::Bytes(b) => b.clone(),
                StreamItem::Text(s) => s.as_bytes().to_vec(),
                StreamItem::Structured(_) => {
                    // Pass through structured items
                    output.push(item.clone());
                    continue;
                }
            };

            let encoded = BASE64.encode(&input_bytes);
            output.push(StreamItem::Text(encoded));
        }

        Ok(ExecutionResult {
            outputs: vec![output],
            errors: Vec::new(),
            warnings: Vec::new(),
        })
    }
}

// MARK: Chunking Nodes

/// Splits by lines, keeping line endings
pub struct ByLinesNode;

impl Node for ByLinesNode {
    fn name(&self) -> &str {
        "by-lines"
    }

    fn input_arity(&self) -> (usize, Option<usize>) {
        (1, Some(1))
    }

    fn output_count(&self) -> usize {
        1
    }

    fn execute(
        &self,
        inputs: Vec<Stream>,
        _node_index: usize,
    ) -> Result<ExecutionResult, NodeError> {
        if inputs.len() != 1 {
            return Err(NodeError::InvalidInputCount {
                expected: "1".to_string(),
                got: inputs.len(),
            });
        }

        let mut output = Vec::new();

        for item in &inputs[0] {
            let text = match item {
                StreamItem::Text(s) => s.clone(),
                StreamItem::Bytes(b) => String::from_utf8_lossy(b).to_string(),
                StreamItem::Structured(_) => continue,
            };

            // Split by lines, keeping the newline
            for line in text.split_inclusive('\n') {
                output.push(StreamItem::Text(line.to_string()));
            }
        }

        Ok(ExecutionResult {
            outputs: vec![output],
            errors: Vec::new(),
            warnings: Vec::new(),
        })
    }
}

/// Splits by lines, removing line endings
pub struct SplitLinesNode;

impl Node for SplitLinesNode {
    fn name(&self) -> &str {
        "split-lines"
    }

    fn input_arity(&self) -> (usize, Option<usize>) {
        (1, Some(1))
    }

    fn output_count(&self) -> usize {
        1
    }

    fn execute(
        &self,
        inputs: Vec<Stream>,
        _node_index: usize,
    ) -> Result<ExecutionResult, NodeError> {
        if inputs.len() != 1 {
            return Err(NodeError::InvalidInputCount {
                expected: "1".to_string(),
                got: inputs.len(),
            });
        }

        let mut output = Vec::new();

        for item in &inputs[0] {
            let text = match item {
                StreamItem::Text(s) => s.clone(),
                StreamItem::Bytes(b) => String::from_utf8_lossy(b).to_string(),
                StreamItem::Structured(_) => continue,
            };

            for line in text.lines() {
                output.push(StreamItem::Text(line.to_string()));
            }
        }

        Ok(ExecutionResult {
            outputs: vec![output],
            errors: Vec::new(),
            warnings: Vec::new(),
        })
    }
}

/// Splits by null bytes, keeping them
pub struct ByNullNode;

impl Node for ByNullNode {
    fn name(&self) -> &str {
        "by-null"
    }

    fn input_arity(&self) -> (usize, Option<usize>) {
        (1, Some(1))
    }

    fn output_count(&self) -> usize {
        1
    }

    fn execute(
        &self,
        inputs: Vec<Stream>,
        _node_index: usize,
    ) -> Result<ExecutionResult, NodeError> {
        if inputs.len() != 1 {
            return Err(NodeError::InvalidInputCount {
                expected: "1".to_string(),
                got: inputs.len(),
            });
        }

        let mut output = Vec::new();

        for item in &inputs[0] {
            let bytes = match item {
                StreamItem::Bytes(b) => b.clone(),
                StreamItem::Text(s) => s.as_bytes().to_vec(),
                StreamItem::Structured(_) => continue,
            };

            let mut current = Vec::new();
            for &byte in &bytes {
                current.push(byte);
                if byte == 0 {
                    output.push(StreamItem::Bytes(current.clone()));
                    current.clear();
                }
            }

            if !current.is_empty() {
                output.push(StreamItem::Bytes(current));
            }
        }

        Ok(ExecutionResult {
            outputs: vec![output],
            errors: Vec::new(),
            warnings: Vec::new(),
        })
    }
}

/// Splits by null bytes, removing them
pub struct SplitNullNode;

impl Node for SplitNullNode {
    fn name(&self) -> &str {
        "split-null"
    }

    fn input_arity(&self) -> (usize, Option<usize>) {
        (1, Some(1))
    }

    fn output_count(&self) -> usize {
        1
    }

    fn execute(
        &self,
        inputs: Vec<Stream>,
        _node_index: usize,
    ) -> Result<ExecutionResult, NodeError> {
        if inputs.len() != 1 {
            return Err(NodeError::InvalidInputCount {
                expected: "1".to_string(),
                got: inputs.len(),
            });
        }

        let mut output = Vec::new();

        for item in &inputs[0] {
            let bytes = match item {
                StreamItem::Bytes(b) => b.clone(),
                StreamItem::Text(s) => s.as_bytes().to_vec(),
                StreamItem::Structured(_) => continue,
            };

            for chunk in bytes.split(|&b| b == 0) {
                if !chunk.is_empty() {
                    output.push(StreamItem::Bytes(chunk.to_vec()));
                }
            }
        }

        Ok(ExecutionResult {
            outputs: vec![output],
            errors: Vec::new(),
            warnings: Vec::new(),
        })
    }
}

/// Joins items with newlines
pub struct JoinLinesNode;

impl Node for JoinLinesNode {
    fn name(&self) -> &str {
        "join-lines"
    }

    fn input_arity(&self) -> (usize, Option<usize>) {
        (1, Some(1))
    }

    fn output_count(&self) -> usize {
        1
    }

    fn execute(
        &self,
        inputs: Vec<Stream>,
        _node_index: usize,
    ) -> Result<ExecutionResult, NodeError> {
        if inputs.len() != 1 {
            return Err(NodeError::InvalidInputCount {
                expected: "1".to_string(),
                got: inputs.len(),
            });
        }

        // Use bytes for consistent behavior across Text and Bytes items
        let mut result = Vec::new();

        for item in &inputs[0] {
            match item {
                StreamItem::Text(s) => {
                    result.extend_from_slice(s.as_bytes());
                    if !s.ends_with('\n') {
                        result.push(b'\n');
                    }
                }
                StreamItem::Bytes(b) => {
                    result.extend_from_slice(b);
                    if !b.ends_with(b"\n") {
                        result.push(b'\n');
                    }
                }
                StreamItem::Structured(_) => {}
            }
        }

        Ok(ExecutionResult {
            outputs: vec![vec![StreamItem::Bytes(result)]],
            errors: Vec::new(),
            warnings: Vec::new(),
        })
    }
}

// MARK: Aggregation Nodes

/// Collects stream items into an array
pub struct JoinArrayNode;

impl Node for JoinArrayNode {
    fn name(&self) -> &str {
        "join-array"
    }

    fn input_arity(&self) -> (usize, Option<usize>) {
        (1, Some(1))
    }

    fn output_count(&self) -> usize {
        1
    }

    fn execute(
        &self,
        inputs: Vec<Stream>,
        node_index: usize,
    ) -> Result<ExecutionResult, NodeError> {
        if inputs.len() != 1 {
            return Err(NodeError::InvalidInputCount {
                expected: "1".to_string(),
                got: inputs.len(),
            });
        }

        let mut array = Vec::new();
        let mut warnings = Vec::new();

        for item in &inputs[0] {
            match item {
                StreamItem::Structured(s) => {
                    array.push(s.clone());
                }
                StreamItem::Text(t) => {
                    warnings.push(Warning {
                        node_index,
                        message: format!(
                            "join-array: Ignored unexpected Text item: {:?}",
                            if t.len() > 50 {
                                format!("{}...", &t[..50])
                            } else {
                                t.clone()
                            }
                        ),
                        coercion: CoercionKind::Custom("Ignored non-Structured item".to_string()),
                    });
                }
                StreamItem::Bytes(b) => {
                    warnings.push(Warning {
                        node_index,
                        message: format!("join-array: Ignored unexpected Bytes item ({} bytes)", b.len()),
                        coercion: CoercionKind::Custom("Ignored non-Structured item".to_string()),
                    });
                }
            }
        }

        Ok(ExecutionResult {
            outputs: vec![vec![StreamItem::Structured(Structured::Array(array))]],
            errors: Vec::new(),
            warnings,
        })
    }
}

/// Splits an array into individual items
pub struct SplitArrayNode;

impl Node for SplitArrayNode {
    fn name(&self) -> &str {
        "split-array"
    }

    fn input_arity(&self) -> (usize, Option<usize>) {
        (1, Some(1))
    }

    fn output_count(&self) -> usize {
        1
    }

    fn execute(
        &self,
        inputs: Vec<Stream>,
        _node_index: usize,
    ) -> Result<ExecutionResult, NodeError> {
        if inputs.len() != 1 {
            return Err(NodeError::InvalidInputCount {
                expected: "1".to_string(),
                got: inputs.len(),
            });
        }

        let mut output = Vec::new();

        for item in &inputs[0] {
            if let StreamItem::Structured(Structured::Array(arr)) = item {
                for element in arr {
                    output.push(StreamItem::Structured(element.clone()));
                }
            } else {
                // Pass through non-arrays
                output.push(item.clone());
            }
        }

        Ok(ExecutionResult {
            outputs: vec![output],
            errors: Vec::new(),
            warnings: Vec::new(),
        })
    }
}

// MARK: Stream Combining Nodes

/// Concatenates multiple input streams
pub struct ChainNode;

impl Node for ChainNode {
    fn name(&self) -> &str {
        "chain"
    }

    fn input_arity(&self) -> (usize, Option<usize>) {
        (0, None) // Accepts any number of inputs
    }

    fn output_count(&self) -> usize {
        1
    }

    fn execute(
        &self,
        inputs: Vec<Stream>,
        _node_index: usize,
    ) -> Result<ExecutionResult, NodeError> {
        let mut output = Vec::new();

        for input_stream in inputs {
            output.extend(input_stream);
        }

        Ok(ExecutionResult {
            outputs: vec![output],
            errors: Vec::new(),
            warnings: Vec::new(),
        })
    }
}

/// Merges multiple sorted streams
pub struct MergeNode;

impl Node for MergeNode {
    fn name(&self) -> &str {
        "merge"
    }

    fn input_arity(&self) -> (usize, Option<usize>) {
        (0, None) // Accepts any number of inputs
    }

    fn output_count(&self) -> usize {
        1
    }

    fn execute(
        &self,
        inputs: Vec<Stream>,
        _node_index: usize,
    ) -> Result<ExecutionResult, NodeError> {
        // Implement n-way merge of sorted streams
        // Each input stream is assumed to be sorted; we merge them maintaining order
        let mut iterators: Vec<std::iter::Peekable<std::vec::IntoIter<StreamItem>>> = inputs
            .into_iter()
            .map(|s| s.into_iter().peekable())
            .collect();

        let mut output = Vec::new();

        loop {
            // Find the minimum element across all iterators
            // First, collect references to peeked items with their indices
            let mut candidates: Vec<(usize, &StreamItem)> = Vec::new();
            for (idx, iter) in iterators.iter_mut().enumerate() {
                if let Some(item) = iter.peek() {
                    candidates.push((idx, item));
                }
            }

            if candidates.is_empty() {
                break;
            }

            // Find the minimum among candidates
            let min_idx = candidates
                .iter()
                .min_by(|(_, a), (_, b)| compare_stream_items(a, b))
                .map(|(idx, _)| *idx)
                .unwrap();

            // Pop from the iterator with minimum element
            if let Some(item) = iterators[min_idx].next() {
                output.push(item);
            }
        }

        Ok(ExecutionResult {
            outputs: vec![output],
            errors: Vec::new(),
            warnings: Vec::new(),
        })
    }
}

// MARK: Transform Nodes

/// Sorts stream items
pub struct SortNode;

impl Node for SortNode {
    fn name(&self) -> &str {
        "sort"
    }

    fn input_arity(&self) -> (usize, Option<usize>) {
        (1, Some(1))
    }

    fn output_count(&self) -> usize {
        1
    }

    fn execute(
        &self,
        inputs: Vec<Stream>,
        _node_index: usize,
    ) -> Result<ExecutionResult, NodeError> {
        if inputs.len() != 1 {
            return Err(NodeError::InvalidInputCount {
                expected: "1".to_string(),
                got: inputs.len(),
            });
        }

        let mut output = inputs[0].clone();

        // Sort using proper total ordering for all item types
        output.sort_by(compare_stream_items);

        Ok(ExecutionResult {
            outputs: vec![output],
            errors: Vec::new(),
            warnings: Vec::new(),
        })
    }
}

/// Filters stream items
pub struct FilterNode;

impl Node for FilterNode {
    fn name(&self) -> &str {
        "filter"
    }

    fn input_arity(&self) -> (usize, Option<usize>) {
        (1, Some(1))
    }

    fn output_count(&self) -> usize {
        1
    }

    fn execute(
        &self,
        inputs: Vec<Stream>,
        _node_index: usize,
    ) -> Result<ExecutionResult, NodeError> {
        if inputs.len() != 1 {
            return Err(NodeError::InvalidInputCount {
                expected: "1".to_string(),
                got: inputs.len(),
            });
        }

        // Filter: only pass through Structured items where the field "keep" is true
        // For non-structured items, pass through if they are non-empty
        let mut filtered = Vec::new();

        for item in &inputs[0] {
            match item {
                StreamItem::Structured(s) => {
                    // Check for "keep" field
                    let should_keep = match s {
                        Structured::TextMap(map) => {
                            map.get("keep")
                                .map(|v| matches!(v, Structured::Bool(true)))
                                .unwrap_or(true) // Pass through if no "keep" field
                        }
                        _ => true, // Non-map structured values pass through
                    };
                    if should_keep {
                        filtered.push(item.clone());
                    }
                }
                StreamItem::Text(t) => {
                    // Filter out empty text
                    if !t.is_empty() {
                        filtered.push(item.clone());
                    }
                }
                StreamItem::Bytes(b) => {
                    // Filter out empty bytes
                    if !b.is_empty() {
                        filtered.push(item.clone());
                    }
                }
            }
        }

        Ok(ExecutionResult {
            outputs: vec![filtered],
            errors: Vec::new(),
            warnings: Vec::new(),
        })
    }
}

// MARK: Helper Functions

/// Compare two StreamItems for total ordering
/// Order: Bytes < Text < Structured
/// Within each type: lexicographic/natural ordering
fn compare_stream_items(a: &StreamItem, b: &StreamItem) -> std::cmp::Ordering {
    use std::cmp::Ordering;

    match (a, b) {
        // Bytes comparisons
        (StreamItem::Bytes(a), StreamItem::Bytes(b)) => a.cmp(b),
        (StreamItem::Bytes(_), _) => Ordering::Less,
        (_, StreamItem::Bytes(_)) => Ordering::Greater,

        // Text comparisons
        (StreamItem::Text(a), StreamItem::Text(b)) => a.cmp(b),
        (StreamItem::Text(_), StreamItem::Structured(_)) => Ordering::Less,
        (StreamItem::Structured(_), StreamItem::Text(_)) => Ordering::Greater,

        // Structured comparisons
        (StreamItem::Structured(a), StreamItem::Structured(b)) => compare_structured(a, b),
    }
}

/// Compare two Structured values for total ordering
/// Order: Null < Bool < Numbers < TextString < BinaryString < Array < TextMap < BinaryMap
fn compare_structured(a: &Structured, b: &Structured) -> std::cmp::Ordering {
    use std::cmp::Ordering;

    fn type_order(s: &Structured) -> u8 {
        match s {
            Structured::Null => 0,
            Structured::Bool(_) => 1,
            Structured::SignedInt(_) | Structured::UnsignedInt(_) | Structured::Float(_) => 2,
            Structured::TextString(_) => 3,
            Structured::BinaryString(_) => 4,
            Structured::Array(_) => 5,
            Structured::TextMap(_) => 6,
            Structured::BinaryMap(_) => 7,
        }
    }

    let type_a = type_order(a);
    let type_b = type_order(b);

    if type_a != type_b {
        return type_a.cmp(&type_b);
    }

    match (a, b) {
        (Structured::Null, Structured::Null) => Ordering::Equal,
        (Structured::Bool(a), Structured::Bool(b)) => a.cmp(b),
        // Compare same numeric types directly for precision
        (Structured::SignedInt(a), Structured::SignedInt(b)) => a.cmp(b),
        (Structured::UnsignedInt(a), Structured::UnsignedInt(b)) => a.cmp(b),
        // For floats, treat NaN as greater than all values for consistent ordering
        (Structured::Float(a), Structured::Float(b)) => {
            match (a.is_nan(), b.is_nan()) {
                (true, true) => Ordering::Equal,
                (true, false) => Ordering::Greater,
                (false, true) => Ordering::Less,
                (false, false) => a.partial_cmp(b).unwrap_or(Ordering::Equal),
            }
        }
        // Mixed integer types: compare directly when possible
        (Structured::SignedInt(a), Structured::UnsignedInt(b)) => {
            if *a < 0 {
                Ordering::Less
            } else {
                (*a as u64).cmp(b)
            }
        }
        (Structured::UnsignedInt(a), Structured::SignedInt(b)) => {
            if *b < 0 {
                Ordering::Greater
            } else {
                a.cmp(&(*b as u64))
            }
        }
        // Mixed float/integer: use f64 comparison with NaN handling
        // Note: Large integers (>2^53) may lose precision when converted to f64
        (Structured::SignedInt(a), Structured::Float(b)) => {
            if b.is_nan() {
                Ordering::Less
            } else {
                (*a as f64).partial_cmp(b).unwrap_or(Ordering::Equal)
            }
        }
        (Structured::Float(a), Structured::SignedInt(b)) => {
            if a.is_nan() {
                Ordering::Greater
            } else {
                a.partial_cmp(&(*b as f64)).unwrap_or(Ordering::Equal)
            }
        }
        (Structured::UnsignedInt(a), Structured::Float(b)) => {
            if b.is_nan() {
                Ordering::Less
            } else {
                (*a as f64).partial_cmp(b).unwrap_or(Ordering::Equal)
            }
        }
        (Structured::Float(a), Structured::UnsignedInt(b)) => {
            if a.is_nan() {
                Ordering::Greater
            } else {
                a.partial_cmp(&(*b as f64)).unwrap_or(Ordering::Equal)
            }
        }
        (Structured::TextString(a), Structured::TextString(b)) => a.cmp(b),
        (Structured::BinaryString(a), Structured::BinaryString(b)) => a.cmp(b),
        (Structured::Array(a), Structured::Array(b)) => {
            for (item_a, item_b) in a.iter().zip(b.iter()) {
                match compare_structured(item_a, item_b) {
                    Ordering::Equal => continue,
                    other => return other,
                }
            }
            a.len().cmp(&b.len())
        }
        (Structured::TextMap(a), Structured::TextMap(b)) => {
            for ((ka, va), (kb, vb)) in a.iter().zip(b.iter()) {
                match ka.cmp(kb) {
                    Ordering::Equal => match compare_structured(va, vb) {
                        Ordering::Equal => continue,
                        other => return other,
                    },
                    other => return other,
                }
            }
            a.len().cmp(&b.len())
        }
        (Structured::BinaryMap(a), Structured::BinaryMap(b)) => {
            for ((ka, va), (kb, vb)) in a.iter().zip(b.iter()) {
                match ka.cmp(kb) {
                    Ordering::Equal => match compare_structured(va, vb) {
                        Ordering::Equal => continue,
                        other => return other,
                    },
                    other => return other,
                }
            }
            a.len().cmp(&b.len())
        }
        _ => Ordering::Equal, // Should not happen due to type_order check
    }
}

/// Convert serde_json::Value to our Structured type
fn json_value_to_structured(value: &serde_json::Value) -> Option<Structured> {
    match value {
        serde_json::Value::Null => Some(Structured::Null),
        serde_json::Value::Bool(b) => Some(Structured::Bool(*b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Some(Structured::SignedInt(i))
            } else if let Some(u) = n.as_u64() {
                Some(Structured::UnsignedInt(u))
            } else if let Some(f) = n.as_f64() {
                Some(Structured::Float(f))
            } else {
                None
            }
        }
        serde_json::Value::String(s) => Some(Structured::TextString(s.clone())),
        serde_json::Value::Array(arr) => {
            let mut result = Vec::new();
            for item in arr {
                if let Some(structured) = json_value_to_structured(item) {
                    result.push(structured);
                } else {
                    return None;
                }
            }
            Some(Structured::Array(result))
        }
        serde_json::Value::Object(obj) => {
            let mut result = indexmap::IndexMap::new();
            for (key, value) in obj {
                if let Some(structured) = json_value_to_structured(value) {
                    result.insert(key.clone(), structured);
                } else {
                    return None;
                }
            }
            Some(Structured::TextMap(result))
        }
    }
}

/// Convert our Structured type to serde_json::Value
fn structured_to_json_value(structured: &Structured) -> Option<serde_json::Value> {
    match structured {
        Structured::Null => Some(serde_json::Value::Null),
        Structured::Bool(b) => Some(serde_json::Value::Bool(*b)),
        // Use Number::from() directly to avoid precision loss
        Structured::SignedInt(i) => Some(serde_json::Value::Number(serde_json::Number::from(*i))),
        Structured::UnsignedInt(u) => Some(serde_json::Value::Number(serde_json::Number::from(*u))),
        Structured::Float(f) => serde_json::Number::from_f64(*f)
            .map(serde_json::Value::Number),
        Structured::TextString(s) => Some(serde_json::Value::String(s.clone())),
        Structured::BinaryString(b) => {
            // Convert binary to base64 string for JSON to preserve data integrity
            Some(serde_json::Value::String(BASE64.encode(b)))
        }
        Structured::Array(arr) => {
            let mut result = Vec::new();
            for item in arr {
                if let Some(value) = structured_to_json_value(item) {
                    result.push(value);
                } else {
                    return None;
                }
            }
            Some(serde_json::Value::Array(result))
        }
        Structured::TextMap(map) => {
            let mut result = serde_json::Map::new();
            for (key, value) in map {
                if let Some(json_value) = structured_to_json_value(value) {
                    result.insert(key.clone(), json_value);
                } else {
                    return None;
                }
            }
            Some(serde_json::Value::Object(result))
        }
        Structured::BinaryMap(_) => {
            // Can't represent binary-keyed maps in JSON
            None
        }
    }
}

// MARK: JEB-Specific Encoding Nodes

/// Encodes data using Z85 encoding
pub struct EncodeZ85Node;

impl Node for EncodeZ85Node {
    fn name(&self) -> &str {
        "encode-z85"
    }

    fn input_arity(&self) -> (usize, Option<usize>) {
        (1, Some(1))
    }

    fn output_count(&self) -> usize {
        1
    }

    fn execute(
        &self,
        inputs: Vec<Stream>,
        _node_index: usize,
    ) -> Result<ExecutionResult, NodeError> {
        if inputs.len() != 1 {
            return Err(NodeError::InvalidInputCount {
                expected: "1".to_string(),
                got: inputs.len(),
            });
        }

        let mut output = Vec::new();

        for item in &inputs[0] {
            let bytes = match item {
                StreamItem::Bytes(b) => b,
                StreamItem::Text(s) => s.as_bytes(),
                StreamItem::Structured(_) => continue,
            };

            let encoded = crate::encode_z85(bytes);
            output.push(StreamItem::Bytes(encoded));
        }

        Ok(ExecutionResult {
            outputs: vec![output],
            errors: Vec::new(),
            warnings: Vec::new(),
        })
    }
}

/// Encodes data using JEB85 encoding
pub struct EncodeJeb85Node;

impl Node for EncodeJeb85Node {
    fn name(&self) -> &str {
        "encode-jeb85"
    }

    fn input_arity(&self) -> (usize, Option<usize>) {
        (1, Some(1))
    }

    fn output_count(&self) -> usize {
        1
    }

    fn execute(
        &self,
        inputs: Vec<Stream>,
        _node_index: usize,
    ) -> Result<ExecutionResult, NodeError> {
        if inputs.len() != 1 {
            return Err(NodeError::InvalidInputCount {
                expected: "1".to_string(),
                got: inputs.len(),
            });
        }

        let mut output = Vec::new();

        for item in &inputs[0] {
            let bytes = match item {
                StreamItem::Bytes(b) => b,
                StreamItem::Text(s) => s.as_bytes(),
                StreamItem::Structured(_) => continue,
            };

            let encoded = crate::encode_jeb85(bytes);
            output.push(StreamItem::Bytes(encoded));
        }

        Ok(ExecutionResult {
            outputs: vec![output],
            errors: Vec::new(),
            warnings: Vec::new(),
        })
    }
}

// MARK: Additional Utility Nodes

/// Reads the current executable
pub struct SelfNode;

impl Node for SelfNode {
    fn name(&self) -> &str {
        "self"
    }

    fn input_arity(&self) -> (usize, Option<usize>) {
        (0, Some(0))
    }

    fn output_count(&self) -> usize {
        1
    }

    fn execute(
        &self,
        _inputs: Vec<Stream>,
        _node_index: usize,
    ) -> Result<ExecutionResult, NodeError> {
        let own_path = std::env::current_exe()?;
        let data = std::fs::read(own_path)?;

        Ok(ExecutionResult {
            outputs: vec![vec![StreamItem::Bytes(data)]],
            errors: Vec::new(),
            warnings: Vec::new(),
        })
    }
}

/// Keeps only the first item
pub struct FirstNode;

impl Node for FirstNode {
    fn name(&self) -> &str {
        "first"
    }

    fn input_arity(&self) -> (usize, Option<usize>) {
        (1, Some(1))
    }

    fn output_count(&self) -> usize {
        1
    }

    fn execute(
        &self,
        inputs: Vec<Stream>,
        _node_index: usize,
    ) -> Result<ExecutionResult, NodeError> {
        if inputs.len() != 1 {
            return Err(NodeError::InvalidInputCount {
                expected: "1".to_string(),
                got: inputs.len(),
            });
        }

        let output = inputs[0].iter().take(1).cloned().collect();

        Ok(ExecutionResult {
            outputs: vec![output],
            errors: Vec::new(),
            warnings: Vec::new(),
        })
    }
}

/// Keeps only the last item
pub struct LastNode;

impl Node for LastNode {
    fn name(&self) -> &str {
        "last"
    }

    fn input_arity(&self) -> (usize, Option<usize>) {
        (1, Some(1))
    }

    fn output_count(&self) -> usize {
        1
    }

    fn execute(
        &self,
        inputs: Vec<Stream>,
        _node_index: usize,
    ) -> Result<ExecutionResult, NodeError> {
        if inputs.len() != 1 {
            return Err(NodeError::InvalidInputCount {
                expected: "1".to_string(),
                got: inputs.len(),
            });
        }

        let output = if let Some(last) = inputs[0].last() {
            vec![last.clone()]
        } else {
            Vec::new()
        };

        Ok(ExecutionResult {
            outputs: vec![output],
            errors: Vec::new(),
            warnings: Vec::new(),
        })
    }
}

/// Keeps first N items
pub struct FirstNNode {
    n: usize,
}

impl FirstNNode {
    pub fn new(n: usize) -> Self {
        Self { n }
    }
}

impl Node for FirstNNode {
    fn name(&self) -> &str {
        "first-n"
    }

    fn input_arity(&self) -> (usize, Option<usize>) {
        (1, Some(1))
    }

    fn output_count(&self) -> usize {
        1
    }

    fn execute(
        &self,
        inputs: Vec<Stream>,
        _node_index: usize,
    ) -> Result<ExecutionResult, NodeError> {
        if inputs.len() != 1 {
            return Err(NodeError::InvalidInputCount {
                expected: "1".to_string(),
                got: inputs.len(),
            });
        }

        let output = inputs[0].iter().take(self.n).cloned().collect();

        Ok(ExecutionResult {
            outputs: vec![output],
            errors: Vec::new(),
            warnings: Vec::new(),
        })
    }
}

/// Keeps last N items
pub struct LastNNode {
    n: usize,
}

impl LastNNode {
    pub fn new(n: usize) -> Self {
        Self { n }
    }
}

impl Node for LastNNode {
    fn name(&self) -> &str {
        "last-n"
    }

    fn input_arity(&self) -> (usize, Option<usize>) {
        (1, Some(1))
    }

    fn output_count(&self) -> usize {
        1
    }

    fn execute(
        &self,
        inputs: Vec<Stream>,
        _node_index: usize,
    ) -> Result<ExecutionResult, NodeError> {
        if inputs.len() != 1 {
            return Err(NodeError::InvalidInputCount {
                expected: "1".to_string(),
                got: inputs.len(),
            });
        }

        let len = inputs[0].len();
        let skip = if len > self.n { len - self.n } else { 0 };
        let output = inputs[0].iter().skip(skip).cloned().collect();

        Ok(ExecutionResult {
            outputs: vec![output],
            errors: Vec::new(),
            warnings: Vec::new(),
        })
    }
}

/// Collapses whitespace
pub struct CollapseNode;

impl Node for CollapseNode {
    fn name(&self) -> &str {
        "collapse"
    }

    fn input_arity(&self) -> (usize, Option<usize>) {
        (1, Some(1))
    }

    fn output_count(&self) -> usize {
        1
    }

    fn execute(
        &self,
        inputs: Vec<Stream>,
        _node_index: usize,
    ) -> Result<ExecutionResult, NodeError> {
        if inputs.len() != 1 {
            return Err(NodeError::InvalidInputCount {
                expected: "1".to_string(),
                got: inputs.len(),
            });
        }

        let mut output = Vec::new();

        for item in &inputs[0] {
            let bytes = match item {
                StreamItem::Bytes(b) => b.clone(),
                StreamItem::Text(s) => s.as_bytes().to_vec(),
                StreamItem::Structured(_) => continue,
            };

            let mut result = Vec::new();
            let mut in_whitespace = false;

            for &byte in &bytes {
                if byte.is_ascii_whitespace() {
                    in_whitespace = true;
                } else {
                    if in_whitespace {
                        result.push(b' ');
                        in_whitespace = false;
                    }
                    result.push(byte);
                }
            }

            output.push(StreamItem::Bytes(result));
        }

        Ok(ExecutionResult {
            outputs: vec![output],
            errors: Vec::new(),
            warnings: Vec::new(),
        })
    }
}

/// Joins items with spaces
pub struct JoinSpaceNode;

impl Node for JoinSpaceNode {
    fn name(&self) -> &str {
        "join-space"
    }

    fn input_arity(&self) -> (usize, Option<usize>) {
        (1, Some(1))
    }

    fn output_count(&self) -> usize {
        1
    }

    fn execute(
        &self,
        inputs: Vec<Stream>,
        _node_index: usize,
    ) -> Result<ExecutionResult, NodeError> {
        if inputs.len() != 1 {
            return Err(NodeError::InvalidInputCount {
                expected: "1".to_string(),
                got: inputs.len(),
            });
        }

        let mut result = Vec::new();

        for (idx, item) in inputs[0].iter().enumerate() {
            if idx > 0 {
                result.push(b' ');
            }

            match item {
                StreamItem::Bytes(b) => result.extend_from_slice(b),
                StreamItem::Text(s) => result.extend_from_slice(s.as_bytes()),
                StreamItem::Structured(_) => {}
            }
        }

        Ok(ExecutionResult {
            outputs: vec![vec![StreamItem::Bytes(result)]],
            errors: Vec::new(),
            warnings: Vec::new(),
        })
    }
}

/// Joins all bytes together
pub struct JoinNode;

impl Node for JoinNode {
    fn name(&self) -> &str {
        "join"
    }

    fn input_arity(&self) -> (usize, Option<usize>) {
        (1, Some(1))
    }

    fn output_count(&self) -> usize {
        1
    }

    fn execute(
        &self,
        inputs: Vec<Stream>,
        _node_index: usize,
    ) -> Result<ExecutionResult, NodeError> {
        if inputs.len() != 1 {
            return Err(NodeError::InvalidInputCount {
                expected: "1".to_string(),
                got: inputs.len(),
            });
        }

        let mut result = Vec::new();

        for item in &inputs[0] {
            match item {
                StreamItem::Bytes(b) => result.extend_from_slice(b),
                StreamItem::Text(s) => result.extend_from_slice(s.as_bytes()),
                StreamItem::Structured(_) => {}
            }
        }

        Ok(ExecutionResult {
            outputs: vec![vec![StreamItem::Bytes(result)]],
            errors: Vec::new(),
            warnings: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create test streams
    fn text_stream(items: &[&str]) -> Stream {
        items.iter().map(|s| StreamItem::Text(s.to_string())).collect()
    }

    // Test base64 encoding
    #[test]
    fn test_to_base64() {
        let node = ToBase64Node;
        let input = vec![vec![StreamItem::Bytes(b"hello".to_vec())]];
        let result = node.execute(input, 0).unwrap();

        assert_eq!(result.outputs.len(), 1);
        assert_eq!(result.outputs[0].len(), 1);

        if let StreamItem::Text(encoded) = &result.outputs[0][0] {
            assert_eq!(encoded, "aGVsbG8=");
        } else {
            panic!("Expected Text output");
        }
    }

    // Test base64 decoding
    #[test]
    fn test_from_base64() {
        let node = FromBase64Node;
        let input = vec![vec![StreamItem::Text("aGVsbG8=".to_string())]];
        let result = node.execute(input, 0).unwrap();

        assert_eq!(result.outputs.len(), 1);
        assert_eq!(result.outputs[0].len(), 1);

        if let StreamItem::Bytes(decoded) = &result.outputs[0][0] {
            assert_eq!(decoded, b"hello");
        } else {
            panic!("Expected Bytes output");
        }
    }

    // Test base64 decoding with invalid input
    #[test]
    fn test_from_base64_invalid() {
        let node = FromBase64Node;
        let input = vec![vec![StreamItem::Text("!!!invalid!!!".to_string())]];
        let result = node.execute(input, 0).unwrap();

        // Should have errors, no output
        assert!(!result.errors.is_empty());
        assert!(result.outputs[0].is_empty());
    }

    // Test parse-json
    #[test]
    fn test_parse_json() {
        let node = ParseJsonNode;
        let input = vec![vec![StreamItem::Text(r#"{"name":"test"}"#.to_string())]];
        let result = node.execute(input, 0).unwrap();

        assert_eq!(result.outputs.len(), 1);
        assert_eq!(result.outputs[0].len(), 1);

        if let StreamItem::Structured(s) = &result.outputs[0][0] {
            match s {
                Structured::TextMap(map) => {
                    assert!(map.contains_key("name"));
                }
                _ => panic!("Expected TextMap"),
            }
        } else {
            panic!("Expected Structured output");
        }
    }

    // Test parse-json with invalid input
    #[test]
    fn test_parse_json_invalid() {
        let node = ParseJsonNode;
        let input = vec![vec![StreamItem::Text("not json".to_string())]];
        let result = node.execute(input, 0).unwrap();

        // Should have errors
        assert!(!result.errors.is_empty());
    }

    // Test to-json
    #[test]
    fn test_to_json() {
        let node = ToJsonNode;
        let mut map = indexmap::IndexMap::new();
        map.insert("key".to_string(), Structured::TextString("value".to_string()));
        let input = vec![vec![StreamItem::Structured(Structured::TextMap(map))]];

        let result = node.execute(input, 0).unwrap();

        assert_eq!(result.outputs.len(), 1);
        assert_eq!(result.outputs[0].len(), 1);

        if let StreamItem::Text(json) = &result.outputs[0][0] {
            assert!(json.contains("key"));
            assert!(json.contains("value"));
        } else {
            panic!("Expected Text output");
        }
    }

    // Test sort
    #[test]
    fn test_sort_text() {
        let node = SortNode;
        let input = vec![text_stream(&["c", "a", "b"])];
        let result = node.execute(input, 0).unwrap();

        let output: Vec<String> = result.outputs[0]
            .iter()
            .filter_map(|item| {
                if let StreamItem::Text(s) = item {
                    Some(s.clone())
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(output, vec!["a", "b", "c"]);
    }

    // Test filter removes empty items
    #[test]
    fn test_filter_removes_empty() {
        let node = FilterNode;
        let input = vec![text_stream(&["a", "", "b", ""])];
        let result = node.execute(input, 0).unwrap();

        assert_eq!(result.outputs[0].len(), 2);
    }

    // Test filter with structured items
    #[test]
    fn test_filter_structured_keep_field() {
        let node = FilterNode;

        let mut keep_true = indexmap::IndexMap::new();
        keep_true.insert("keep".to_string(), Structured::Bool(true));

        let mut keep_false = indexmap::IndexMap::new();
        keep_false.insert("keep".to_string(), Structured::Bool(false));

        let input = vec![vec![
            StreamItem::Structured(Structured::TextMap(keep_true)),
            StreamItem::Structured(Structured::TextMap(keep_false)),
        ]];

        let result = node.execute(input, 0).unwrap();

        // Only the item with keep: true should pass
        assert_eq!(result.outputs[0].len(), 1);
    }

    // Test split-lines
    #[test]
    fn test_split_lines() {
        let node = SplitLinesNode;
        let input = vec![vec![StreamItem::Text("a\nb\nc".to_string())]];
        let result = node.execute(input, 0).unwrap();

        assert_eq!(result.outputs[0].len(), 3);
    }

    // Test join-lines
    #[test]
    fn test_join_lines() {
        let node = JoinLinesNode;
        let input = vec![text_stream(&["a", "b", "c"])];
        let result = node.execute(input, 0).unwrap();

        assert_eq!(result.outputs[0].len(), 1);
        if let StreamItem::Bytes(b) = &result.outputs[0][0] {
            assert_eq!(b, b"a\nb\nc\n");
        } else {
            panic!("Expected Bytes output");
        }
    }

    // Test join-array
    #[test]
    fn test_join_array() {
        let node = JoinArrayNode;
        let input = vec![vec![
            StreamItem::Structured(Structured::SignedInt(1)),
            StreamItem::Structured(Structured::SignedInt(2)),
            StreamItem::Structured(Structured::SignedInt(3)),
        ]];
        let result = node.execute(input, 0).unwrap();

        assert_eq!(result.outputs[0].len(), 1);
        if let StreamItem::Structured(Structured::Array(arr)) = &result.outputs[0][0] {
            assert_eq!(arr.len(), 3);
        } else {
            panic!("Expected Array output");
        }
    }

    // Test join-array emits warnings for non-structured
    #[test]
    fn test_join_array_warnings() {
        let node = JoinArrayNode;
        let input = vec![vec![
            StreamItem::Text("ignored".to_string()),
            StreamItem::Structured(Structured::SignedInt(1)),
        ]];
        let result = node.execute(input, 0).unwrap();

        // Should have one warning for the ignored Text item
        assert_eq!(result.warnings.len(), 1);
    }

    // Test split-array
    #[test]
    fn test_split_array() {
        let node = SplitArrayNode;
        let arr = vec![
            Structured::SignedInt(1),
            Structured::SignedInt(2),
            Structured::SignedInt(3),
        ];
        let input = vec![vec![StreamItem::Structured(Structured::Array(arr))]];
        let result = node.execute(input, 0).unwrap();

        assert_eq!(result.outputs[0].len(), 3);
    }

    // Test chain
    #[test]
    fn test_chain() {
        let node = ChainNode;
        let input = vec![
            text_stream(&["a", "b"]),
            text_stream(&["c", "d"]),
        ];
        let result = node.execute(input, 0).unwrap();

        assert_eq!(result.outputs[0].len(), 4);
    }

    // Test merge (sorted merge)
    #[test]
    fn test_merge_sorted() {
        let node = MergeNode;
        // Two sorted streams
        let input = vec![
            text_stream(&["a", "c", "e"]),
            text_stream(&["b", "d", "f"]),
        ];
        let result = node.execute(input, 0).unwrap();

        let output: Vec<String> = result.outputs[0]
            .iter()
            .filter_map(|item| {
                if let StreamItem::Text(s) = item {
                    Some(s.clone())
                } else {
                    None
                }
            })
            .collect();

        // Should be merged in sorted order
        assert_eq!(output, vec!["a", "b", "c", "d", "e", "f"]);
    }

    // Test first
    #[test]
    fn test_first() {
        let node = FirstNode;
        let input = vec![text_stream(&["a", "b", "c"])];
        let result = node.execute(input, 0).unwrap();

        assert_eq!(result.outputs[0].len(), 1);
        if let StreamItem::Text(s) = &result.outputs[0][0] {
            assert_eq!(s, "a");
        }
    }

    // Test last
    #[test]
    fn test_last() {
        let node = LastNode;
        let input = vec![text_stream(&["a", "b", "c"])];
        let result = node.execute(input, 0).unwrap();

        assert_eq!(result.outputs[0].len(), 1);
        if let StreamItem::Text(s) = &result.outputs[0][0] {
            assert_eq!(s, "c");
        }
    }

    // Test first-n
    #[test]
    fn test_first_n() {
        let node = FirstNNode::new(2);
        let input = vec![text_stream(&["a", "b", "c", "d"])];
        let result = node.execute(input, 0).unwrap();

        assert_eq!(result.outputs[0].len(), 2);
    }

    // Test last-n
    #[test]
    fn test_last_n() {
        let node = LastNNode::new(2);
        let input = vec![text_stream(&["a", "b", "c", "d"])];
        let result = node.execute(input, 0).unwrap();

        assert_eq!(result.outputs[0].len(), 2);
        if let StreamItem::Text(s) = &result.outputs[0][0] {
            assert_eq!(s, "c");
        }
    }

    // Test collapse whitespace
    #[test]
    fn test_collapse() {
        let node = CollapseNode;
        let input = vec![vec![StreamItem::Text("a   b\t\nc".to_string())]];
        let result = node.execute(input, 0).unwrap();

        if let StreamItem::Bytes(b) = &result.outputs[0][0] {
            assert_eq!(b, b"a b c");
        }
    }

    // Test comparison functions
    #[test]
    fn test_compare_stream_items() {
        // Bytes < Text < Structured
        assert!(compare_stream_items(
            &StreamItem::Bytes(vec![]),
            &StreamItem::Text(String::new())
        ) == std::cmp::Ordering::Less);

        assert!(compare_stream_items(
            &StreamItem::Text(String::new()),
            &StreamItem::Structured(Structured::Null)
        ) == std::cmp::Ordering::Less);

        // Text ordering
        assert!(compare_stream_items(
            &StreamItem::Text("a".to_string()),
            &StreamItem::Text("b".to_string())
        ) == std::cmp::Ordering::Less);
    }

    // Test JSON conversion preserves integers
    #[test]
    fn test_structured_to_json_integers() {
        let large_int = Structured::SignedInt(9007199254740993); // > 2^53
        let json = structured_to_json_value(&large_int).unwrap();

        // Should be a number
        assert!(json.is_number());

        // Convert back and check
        let structured = json_value_to_structured(&json).unwrap();
        if let Structured::SignedInt(i) = structured {
            assert_eq!(i, 9007199254740993);
        } else {
            panic!("Expected SignedInt");
        }
    }

    // Test JSON conversion uses base64 for binary
    #[test]
    fn test_structured_to_json_binary() {
        let binary = Structured::BinaryString(vec![0, 1, 2, 255]);
        let json = structured_to_json_value(&binary).unwrap();

        // Should be a base64 string
        assert!(json.is_string());
        let s = json.as_str().unwrap();
        // Base64 decode should give back original bytes
        let decoded = BASE64.decode(s.as_bytes()).unwrap();
        assert_eq!(decoded, vec![0, 1, 2, 255]);
    }
}