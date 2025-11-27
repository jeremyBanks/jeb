// Node implementations for the pipeline system

use crate::{
    model::{ErrorValue, StreamItem, Structured},
    pipeline::{ExecutionResult, Node, NodeError, Stream},
};
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
        _node_index: usize,
    ) -> Result<ExecutionResult, NodeError> {
        if inputs.len() != 1 {
            return Err(NodeError::InvalidInputCount {
                expected: "1".to_string(),
                got: inputs.len(),
            });
        }

        let mut stdout = std::io::stdout();

        for item in &inputs[0] {
            match item {
                StreamItem::Text(s) => stdout.write_all(s.as_bytes())?,
                StreamItem::Bytes(b) => stdout.write_all(b)?,
                StreamItem::Structured(_) => {
                    // For now, just skip structured items
                    // TODO: Implement proper coercion
                }
            }
        }

        stdout.flush()?;

        Ok(ExecutionResult {
            outputs: Vec::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
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
        _node_index: usize,
    ) -> Result<ExecutionResult, NodeError> {
        if inputs.len() != 1 {
            return Err(NodeError::InvalidInputCount {
                expected: "1".to_string(),
                got: inputs.len(),
            });
        }

        let mut stderr = std::io::stderr();

        for item in &inputs[0] {
            match item {
                StreamItem::Text(s) => stderr.write_all(s.as_bytes())?,
                StreamItem::Bytes(b) => stderr.write_all(b)?,
                StreamItem::Structured(_) => {
                    // Skip structured items
                }
            }
        }

        stderr.flush()?;

        Ok(ExecutionResult {
            outputs: Vec::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
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
            // For now, just pass through
            // TODO: Implement actual base64 decoding
            output.push(item.clone());
        }

        Ok(ExecutionResult {
            outputs: vec![output],
            errors: Vec::new(),
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
            // For now, just pass through
            // TODO: Implement actual base64 encoding
            output.push(item.clone());
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

        let mut result = String::new();

        for item in &inputs[0] {
            match item {
                StreamItem::Text(s) => {
                    result.push_str(s);
                    if !s.ends_with('\n') {
                        result.push('\n');
                    }
                }
                StreamItem::Bytes(b) => {
                    result.push_str(&String::from_utf8_lossy(b));
                    result.push('\n');
                }
                StreamItem::Structured(_) => {}
            }
        }

        Ok(ExecutionResult {
            outputs: vec![vec![StreamItem::Text(result)]],
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
        _node_index: usize,
    ) -> Result<ExecutionResult, NodeError> {
        if inputs.len() != 1 {
            return Err(NodeError::InvalidInputCount {
                expected: "1".to_string(),
                got: inputs.len(),
            });
        }

        let mut array = Vec::new();

        for item in &inputs[0] {
            if let StreamItem::Structured(s) = item {
                array.push(s.clone());
            }
        }

        Ok(ExecutionResult {
            outputs: vec![vec![StreamItem::Structured(Structured::Array(array))]],
            errors: Vec::new(),
            warnings: Vec::new(),
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
        // For now, just chain them
        // TODO: Implement true merge with ordering
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

        // Simple sorting for text items
        // TODO: Implement proper sorting with JSON total ordering
        output.sort_by(|a, b| {
            match (a, b) {
                (StreamItem::Text(s1), StreamItem::Text(s2)) => s1.cmp(s2),
                _ => std::cmp::Ordering::Equal,
            }
        });

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

        // For now, just pass through
        // TODO: Implement actual filtering
        let output = inputs[0].clone();

        Ok(ExecutionResult {
            outputs: vec![output],
            errors: Vec::new(),
            warnings: Vec::new(),
        })
    }
}

// MARK: Helper Functions

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
        Structured::SignedInt(i) => serde_json::Number::from_f64(*i as f64)
            .map(serde_json::Value::Number),
        Structured::UnsignedInt(u) => serde_json::Number::from_f64(*u as f64)
            .map(serde_json::Value::Number),
        Structured::Float(f) => serde_json::Number::from_f64(*f)
            .map(serde_json::Value::Number),
        Structured::TextString(s) => Some(serde_json::Value::String(s.clone())),
        Structured::BinaryString(b) => {
            // Convert binary to base64 string for JSON
            Some(serde_json::Value::String(
                String::from_utf8_lossy(b).to_string(),
            ))
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
