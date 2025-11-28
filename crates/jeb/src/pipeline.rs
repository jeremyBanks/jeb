//! Pipeline infrastructure for jeb command execution.
//!
//! A jeb command is a sequence of subcommands that form a directed acyclic graph (DAG).
//! Data flows left-to-right through the pipeline, with nodes executing asynchronously.

use crate::item::{CoercionWarning, Item, MapValue, NumberValue, StringValue, Value};

/// A node in the pipeline DAG.
///
/// Each node has a statically known number of inputs and outputs.
#[derive(Debug, Clone)]
pub struct Node {
    /// The command this node executes
    pub command: Command,
    /// Node index in the pipeline (used for exit status)
    pub index: usize,
    /// Whether this node was implicitly inserted
    pub implicit: bool,
}

/// Commands that can be executed in a pipeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    // Sources (no inputs, one output)
    /// Read from stdin
    Stdin,
    /// Read from a file
    ReadFile(String),
    /// Emit help text
    Help,

    // Sinks (one input, no outputs)
    /// Write to stdout
    Stdout,
    /// Write to stderr
    Stderr,

    // Parsers (consume Bytes/Text, emit Structured)
    /// Parse JSON
    ParseJson,
    /// Decode base64
    FromBase64,

    // Serializers (consume Structured, emit Text/Bytes)
    /// Serialize to JSON
    ToJson,
    /// Encode to base64
    ToBase64,

    // Chunking
    /// Re-chunk by lines, preserving delimiter
    ByLines,
    /// Re-chunk by lines, consuming delimiter
    SplitLines,
    /// Re-chunk by null, preserving delimiter
    ByNull,
    /// Re-chunk by null, consuming delimiter
    SplitNull,
    /// Join items with newlines
    JoinLines,
    /// Join items with null
    JoinNull,
    /// Join items without delimiter
    Join,

    // Aggregation
    /// Collect stream into a single array
    JoinArray,
    /// Emit each element of an array as a separate item
    SplitArray,

    // Combining streams
    /// Concatenate streams in order
    Chain,
    /// Interleave by global item ordering
    Merge,

    // Transforms
    /// Sort items
    Sort,
    /// Filter items
    Filter,
    /// Keep first N items
    First(usize),
    /// Keep last N items
    Last(usize),
    /// Collapse whitespace
    Collapse,

    // Encoding
    /// Encode to Z85
    EncodeZ85,
    /// Encode to JEB85
    EncodeJeb85,
    
    // Shell tokenization
    /// Split shell arguments
    SplitShell,
}

impl Command {
    /// Returns the number of inputs this command accepts.
    #[must_use]
    pub const fn input_count(&self) -> InputCount {
        match self {
            // Sources have no inputs
            Self::Stdin | Self::ReadFile(_) | Self::Help => InputCount::Zero,

            // Most commands have one input
            Self::Stdout
            | Self::Stderr
            | Self::ParseJson
            | Self::FromBase64
            | Self::ToJson
            | Self::ToBase64
            | Self::ByLines
            | Self::SplitLines
            | Self::ByNull
            | Self::SplitNull
            | Self::JoinLines
            | Self::JoinNull
            | Self::Join
            | Self::JoinArray
            | Self::SplitArray
            | Self::Sort
            | Self::Filter
            | Self::First(_)
            | Self::Last(_)
            | Self::Collapse
            | Self::EncodeZ85
            | Self::EncodeJeb85
            | Self::SplitShell => InputCount::One,

            // Chain and Merge accept any number of inputs
            Self::Chain | Self::Merge => InputCount::Many,
        }
    }

    /// Returns the number of outputs this command produces.
    #[must_use]
    pub const fn output_count(&self) -> OutputCount {
        match self {
            // Sinks have no outputs
            Self::Stdout | Self::Stderr => OutputCount::Zero,

            // Most commands have one output
            _ => OutputCount::One,
        }
    }

    /// Returns true if this is a source command.
    #[must_use]
    pub const fn is_source(&self) -> bool {
        matches!(self.input_count(), InputCount::Zero)
    }

    /// Returns true if this is a sink command.
    #[must_use]
    pub const fn is_sink(&self) -> bool {
        matches!(self.output_count(), OutputCount::Zero)
    }
}

/// Number of inputs a command accepts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputCount {
    /// No inputs (source)
    Zero,
    /// Exactly one input
    One,
    /// Any number of inputs
    Many,
}

/// Number of outputs a command produces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputCount {
    /// No outputs (sink)
    Zero,
    /// Exactly one output
    One,
}

/// A compiled pipeline ready for execution.
#[derive(Debug)]
pub struct Pipeline {
    /// Nodes in the pipeline
    pub nodes: Vec<Node>,
    /// Connections between nodes (`from_node_idx`, `to_node_idx`)
    pub connections: Vec<(usize, usize)>,
}

impl Pipeline {
    /// Parse command-line arguments into a pipeline.
    ///
    /// This implements stack-based connection semantics:
    /// - When a multi-input command appears, it consumes unconnected outputs from right to left
    /// - Implicit commands are inserted to ensure well-formed pipelines
    #[must_use]
    pub fn parse(args: &[String]) -> Self {
        let mut nodes = Vec::new();
        let mut connections = Vec::new();
        let mut unconnected_outputs: Vec<usize> = Vec::new();

        // Parse commands
        for arg in args {
            if let Some(cmd) = Self::parse_command(arg) {
                let node_idx = nodes.len();
                let _is_source = cmd.is_source();
                let is_sink = cmd.is_sink();
                let input_count = cmd.input_count();

                nodes.push(Node {
                    command: cmd,
                    index: node_idx,
                    implicit: false,
                });

                // Connect inputs based on stack semantics
                match input_count {
                    InputCount::Zero => {
                        // Source - no inputs to connect
                    }
                    InputCount::One => {
                        // Single input - connect to most recent unconnected output
                        if let Some(from_idx) = unconnected_outputs.pop() {
                            connections.push((from_idx, node_idx));
                        }
                    }
                    InputCount::Many => {
                        // Multi-input - consume all unconnected outputs
                        for from_idx in unconnected_outputs.drain(..) {
                            connections.push((from_idx, node_idx));
                        }
                    }
                }

                // Track this node's output if it has one
                if !is_sink {
                    unconnected_outputs.push(node_idx);
                }
            }
        }

        // Insert implicit commands
        Self::insert_implicit_commands(&mut nodes, &mut connections, &mut unconnected_outputs);

        Self { nodes, connections }
    }

    fn parse_command(arg: &str) -> Option<Command> {
        Some(match arg {
            "help" | "--help" | "-h" | "-?" => Command::Help,
            "stdin" => Command::Stdin,
            "stdout" => Command::Stdout,
            "stderr" => Command::Stderr,
            "parse-json" | "from-json" => Command::ParseJson,
            "to-json" => Command::ToJson,
            "from-base64" => Command::FromBase64,
            "to-base64" => Command::ToBase64,
            "by-lines" => Command::ByLines,
            "split-lines" => Command::SplitLines,
            "by-null" => Command::ByNull,
            "split-null" => Command::SplitNull,
            "join-lines" => Command::JoinLines,
            "join-null" => Command::JoinNull,
            "join" => Command::Join,
            "join-array" => Command::JoinArray,
            "split-array" => Command::SplitArray,
            "chain" => Command::Chain,
            "merge" => Command::Merge,
            "sort" => Command::Sort,
            "filter" => Command::Filter,
            "first" => Command::First(1),
            "last" => Command::Last(1),
            "collapse" => Command::Collapse,
            "encode-z85" => Command::EncodeZ85,
            "encode-jeb85" => Command::EncodeJeb85,
            "split-shell" => Command::SplitShell,
            _ => {
                // Check for file paths
                if arg.starts_with('.') || arg.starts_with('/') {
                    Command::ReadFile(arg.to_string())
                }
                // Check for first-N or last-N
                else if let Some(n) = arg.strip_prefix("first-") {
                    Command::First(n.parse().unwrap_or(1))
                } else if let Some(n) = arg.strip_prefix("last-") {
                    Command::Last(n.parse().unwrap_or(1))
                } else {
                    return None;
                }
            }
        })
    }

    fn insert_implicit_commands(
        nodes: &mut Vec<Node>,
        connections: &mut Vec<(usize, usize)>,
        unconnected_outputs: &mut Vec<usize>,
    ) {
        // If no source exists, prepend stdin
        let has_source = nodes.iter().any(|n| n.command.is_source());
        if !has_source && !nodes.is_empty() {
            // Insert stdin at the beginning
            let stdin_idx = nodes.len();
            nodes.push(Node {
                command: Command::Stdin,
                index: stdin_idx,
                implicit: true,
            });

            // Connect stdin to the first node that needs input
            if let Some(first_node) = nodes.iter().find(|n| {
                !n.command.is_source() && n.index != stdin_idx
            }) {
                let first_idx = first_node.index;
                connections.push((stdin_idx, first_idx));
            }
        }

        // If no sink exists, append stdout
        let has_sink = nodes.iter().any(|n| n.command.is_sink());
        if !has_sink {
            let stdout_idx = nodes.len();
            nodes.push(Node {
                command: Command::Stdout,
                index: stdout_idx,
                implicit: true,
            });

            // If multiple unconnected outputs, insert chain first
            if unconnected_outputs.len() > 1 {
                let chain_idx = nodes.len();
                nodes.push(Node {
                    command: Command::Chain,
                    index: chain_idx,
                    implicit: true,
                });

                for from_idx in unconnected_outputs.drain(..) {
                    connections.push((from_idx, chain_idx));
                }
                connections.push((chain_idx, stdout_idx));
            } else if let Some(from_idx) = unconnected_outputs.pop() {
                connections.push((from_idx, stdout_idx));
            }
        }
    }

    /// Format the pipeline for display.
    #[must_use]
    pub fn display(&self) -> String {
        let mut parts = Vec::new();
        for node in &self.nodes {
            let name = Self::command_name(&node.command);
            if node.implicit {
                parts.push(format!("[{name}]"));
            } else {
                parts.push(name);
            }
        }
        parts.join(" ")
    }

    fn command_name(cmd: &Command) -> String {
        match cmd {
            Command::Stdin => "stdin".to_string(),
            Command::ReadFile(path) => path.clone(),
            Command::Help => "help".to_string(),
            Command::Stdout => "stdout".to_string(),
            Command::Stderr => "stderr".to_string(),
            Command::ParseJson => "parse-json".to_string(),
            Command::FromBase64 => "from-base64".to_string(),
            Command::ToJson => "to-json".to_string(),
            Command::ToBase64 => "to-base64".to_string(),
            Command::ByLines => "by-lines".to_string(),
            Command::SplitLines => "split-lines".to_string(),
            Command::ByNull => "by-null".to_string(),
            Command::SplitNull => "split-null".to_string(),
            Command::JoinLines => "join-lines".to_string(),
            Command::JoinNull => "join-null".to_string(),
            Command::Join => "join".to_string(),
            Command::JoinArray => "join-array".to_string(),
            Command::SplitArray => "split-array".to_string(),
            Command::Chain => "chain".to_string(),
            Command::Merge => "merge".to_string(),
            Command::Sort => "sort".to_string(),
            Command::Filter => "filter".to_string(),
            Command::First(n) => {
                if *n == 1 {
                    "first".to_string()
                } else {
                    format!("first-{n}")
                }
            }
            Command::Last(n) => {
                if *n == 1 {
                    "last".to_string()
                } else {
                    format!("last-{n}")
                }
            }
            Command::Collapse => "collapse".to_string(),
            Command::EncodeZ85 => "encode-z85".to_string(),
            Command::EncodeJeb85 => "encode-jeb85".to_string(),
            Command::SplitShell => "split-shell".to_string(),
        }
    }
}

/// Result of pipeline execution.
#[derive(Debug)]
pub struct ExecutionResult {
    /// Items that were output
    pub items: Vec<Item>,
    /// Warnings that were emitted
    pub warnings: Vec<PipelineWarning>,
    /// Errors that occurred
    pub errors: Vec<PipelineError>,
    /// Exit status (0 = success, 63+ = error at node index)
    pub exit_status: u8,
}

/// A warning from pipeline execution.
#[derive(Debug, Clone)]
pub struct PipelineWarning {
    /// Node index where the warning occurred
    pub node_index: usize,
    /// Warning message
    pub message: String,
    /// Coercion that triggered the warning
    pub coercion: Option<CoercionWarning>,
}

/// An error from pipeline execution.
#[derive(Debug, Clone)]
pub struct PipelineError {
    /// Node index where the error occurred
    pub node_index: usize,
    /// Error message
    pub message: String,
    /// Context for the error
    pub context: Option<String>,
}

impl PipelineError {
    /// Convert this error to a structured Value for the error pipeline.
    #[must_use]
    pub fn to_value(&self) -> Value {
        let mut map = indexmap::IndexMap::new();
        map.insert(
            "node_index".to_string(),
            Value::Number(NumberValue::Unsigned(self.node_index as u64)),
        );
        map.insert(
            "message".to_string(),
            Value::String(StringValue::Text(self.message.clone())),
        );
        if let Some(ctx) = &self.context {
            map.insert(
                "context".to_string(),
                Value::String(StringValue::Text(ctx.clone())),
            );
        }
        Value::Map(MapValue::TextKeyed(map))
    }
}

/// Calculate exit status from the first error's node index.
///
/// Exit status encoding:
/// - 0: success, no warnings
/// - 63: warnings or errors from the first node (index 0)
/// - 64: first error from node at index 1
/// - And so on, up to 96
#[must_use]
pub const fn exit_status_for_node(node_index: usize) -> u8 {
    let status = 63 + node_index;
    if status > 96 {
        96
    } else {
        status as u8
    }
}

/// Executor for running pipelines.
pub struct Executor {
    warnings: Vec<PipelineWarning>,
    errors: Vec<PipelineError>,
}

impl Executor {
    /// Create a new executor.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Execute a pipeline and return the result.
    pub fn execute(&mut self, pipeline: &Pipeline) -> ExecutionResult {
        let mut items: Vec<Item> = Vec::new();

        // Simple sequential execution for now
        for node in &pipeline.nodes {
            match self.execute_node(node, &mut items) {
                Ok(()) => {}
                Err(e) => {
                    self.errors.push(e);
                }
            }
        }

        // Calculate exit status
        let exit_status = if let Some(first_error) = self.errors.first() {
            exit_status_for_node(first_error.node_index)
        } else if !self.warnings.is_empty() {
            exit_status_for_node(self.warnings[0].node_index)
        } else {
            0
        };

        ExecutionResult {
            items,
            warnings: core::mem::take(&mut self.warnings),
            errors: core::mem::take(&mut self.errors),
            exit_status,
        }
    }

    #[expect(clippy::too_many_lines)]
    fn execute_node(&mut self, node: &Node, items: &mut Vec<Item>) -> Result<(), PipelineError> {
        match &node.command {
            Command::Stdin => {
                use std::io::Read;
                let mut buffer = Vec::new();
                std::io::stdin()
                    .read_to_end(&mut buffer)
                    .map_err(|e| PipelineError {
                        node_index: node.index,
                        message: format!("Failed to read stdin: {e}"),
                        context: None,
                    })?;
                items.push(Item::Bytes(buffer));
            }

            Command::ReadFile(path) => {
                let data = std::fs::read(path).map_err(|e| PipelineError {
                    node_index: node.index,
                    message: format!("Failed to read file: {e}"),
                    context: Some(path.clone()),
                })?;
                items.push(Item::Bytes(data));
            }

            Command::Help => {
                static README: &str = include_str!("../README.md");
                items.push(Item::Text(README.to_string()));
            }

            Command::Stdout => {
                use std::io::Write;
                for item in items.drain(..) {
                    let bytes = item.to_bytes();
                    std::io::stdout().write_all(&bytes).map_err(|e| {
                        PipelineError {
                            node_index: node.index,
                            message: format!("Failed to write stdout: {e}"),
                            context: None,
                        }
                    })?;
                }
            }

            Command::Stderr => {
                use std::io::Write;
                for item in items.drain(..) {
                    let bytes = item.to_bytes();
                    std::io::stderr().write_all(&bytes).map_err(|e| {
                        PipelineError {
                            node_index: node.index,
                            message: format!("Failed to write stderr: {e}"),
                            context: None,
                        }
                    })?;
                }
            }

            Command::ParseJson => {
                let mut new_items = Vec::new();
                for item in items.drain(..) {
                    let text = match item.to_text() {
                        Ok(t) => t,
                        Err(w) => {
                            self.warnings.push(PipelineWarning {
                                node_index: node.index,
                                message: "Coerced bytes to text".to_string(),
                                coercion: Some(w),
                            });
                            continue;
                        }
                    };

                    // Parse JSON values from the text
                    match serde_json::from_str::<serde_json::Value>(&text) {
                        Ok(v) => {
                            new_items.push(Item::Structured(Value::from(v)));
                        }
                        Err(e) => {
                            self.errors.push(PipelineError {
                                node_index: node.index,
                                message: format!("Failed to parse JSON: {e}"),
                                context: Some(text.chars().take(100).collect()),
                            });
                        }
                    }
                }
                *items = new_items;
            }

            Command::ToJson => {
                let mut new_items = Vec::new();
                for item in items.drain(..) {
                    let value = match item.to_structured() {
                        Ok(v) => v,
                        Err(w) => {
                            self.warnings.push(PipelineWarning {
                                node_index: node.index,
                                message: "Coerced to structured".to_string(),
                                coercion: Some(w),
                            });
                            continue;
                        }
                    };
                    let json_value: serde_json::Value = value.into();
                    let text = serde_json::to_string_pretty(&json_value).unwrap_or_default();
                    new_items.push(Item::Text(text));
                }
                *items = new_items;
            }

            Command::SplitLines => {
                let mut new_items = Vec::new();
                for item in items.drain(..) {
                    let bytes = item.to_bytes();
                    for line in bytes.split(|&b| b == b'\n') {
                        if !line.is_empty() {
                            new_items.push(Item::Bytes(line.to_vec()));
                        }
                    }
                }
                *items = new_items;
            }

            Command::ByLines => {
                let mut new_items = Vec::new();
                for item in items.drain(..) {
                    let bytes = item.to_bytes();
                    let mut start = 0;
                    for (i, &b) in bytes.iter().enumerate() {
                        if b == b'\n' {
                            new_items.push(Item::Bytes(bytes[start..=i].to_vec()));
                            start = i + 1;
                        }
                    }
                    if start < bytes.len() {
                        new_items.push(Item::Bytes(bytes[start..].to_vec()));
                    }
                }
                *items = new_items;
            }

            Command::SplitNull => {
                let mut new_items = Vec::new();
                for item in items.drain(..) {
                    let bytes = item.to_bytes();
                    for chunk in bytes.split(|&b| b == 0) {
                        if !chunk.is_empty() {
                            new_items.push(Item::Bytes(chunk.to_vec()));
                        }
                    }
                }
                *items = new_items;
            }

            Command::ByNull => {
                let mut new_items = Vec::new();
                for item in items.drain(..) {
                    let bytes = item.to_bytes();
                    let mut start = 0;
                    for (i, &b) in bytes.iter().enumerate() {
                        if b == 0 {
                            new_items.push(Item::Bytes(bytes[start..=i].to_vec()));
                            start = i + 1;
                        }
                    }
                    if start < bytes.len() {
                        new_items.push(Item::Bytes(bytes[start..].to_vec()));
                    }
                }
                *items = new_items;
            }

            Command::JoinLines => {
                let mut result = Vec::new();
                for item in items.drain(..) {
                    result.extend(item.to_bytes());
                    result.push(b'\n');
                }
                items.push(Item::Bytes(result));
            }

            Command::JoinNull => {
                let mut result = Vec::new();
                for item in items.drain(..) {
                    result.extend(item.to_bytes());
                    result.push(0);
                }
                items.push(Item::Bytes(result));
            }

            Command::Join => {
                let result: Vec<u8> = items.drain(..).flat_map(|i| i.to_bytes()).collect();
                items.push(Item::Bytes(result));
            }

            Command::JoinArray => {
                let arr: Vec<Value> = items
                    .drain(..)
                    .filter_map(|i| i.to_structured().ok())
                    .collect();
                items.push(Item::Structured(Value::Array(arr)));
            }

            Command::SplitArray => {
                let mut new_items = Vec::new();
                for item in items.drain(..) {
                    if let Ok(Value::Array(arr)) = item.to_structured() {
                        for v in arr {
                            new_items.push(Item::Structured(v));
                        }
                    } else {
                        new_items.push(item);
                    }
                }
                *items = new_items;
            }

            Command::Chain | Command::Merge => {
                // Chain just preserves order, already handled by sequential execution
                // TODO: Implement proper merge interleaving by global item ordering
            }

            Command::Sort => {
                items.sort_by(|a, b| {
                    let bytes_a = a.to_bytes();
                    let bytes_b = b.to_bytes();
                    bytes_a.cmp(&bytes_b)
                });
            }

            Command::Filter => {
                items.retain(|item| !item.to_bytes().is_empty());
            }

            Command::First(n) => {
                items.truncate(*n);
            }

            Command::Last(n) => {
                let len = items.len();
                if len > *n {
                    items.drain(0..len - *n);
                }
            }

            Command::Collapse => {
                let mut new_items = Vec::new();
                for item in items.drain(..) {
                    let bytes = item.to_bytes();
                    let mut output = Vec::new();
                    let mut in_whitespace = false;
                    for &b in &bytes {
                        if b.is_ascii_whitespace() {
                            in_whitespace = true;
                        } else {
                            if in_whitespace {
                                output.push(b' ');
                                in_whitespace = false;
                            }
                            output.push(b);
                        }
                    }
                    new_items.push(Item::Bytes(output));
                }
                *items = new_items;
            }

            Command::EncodeZ85 => {
                for item in items.iter_mut() {
                    let bytes = item.to_bytes();
                    let encoded = crate::encode_z85(&bytes);
                    *item = Item::Bytes(encoded);
                }
            }

            Command::EncodeJeb85 => {
                for item in items.iter_mut() {
                    let bytes = item.to_bytes();
                    let encoded = crate::encode_jeb85(&bytes);
                    *item = Item::Bytes(encoded);
                }
            }

            Command::FromBase64 => {
                let mut new_items = Vec::new();
                for item in items.drain(..) {
                    let bytes = item.to_bytes();
                    match base64_decode(&bytes) {
                        Ok(decoded) => {
                            new_items.push(Item::Bytes(decoded));
                        }
                        Err(e) => {
                            self.errors.push(PipelineError {
                                node_index: node.index,
                                message: format!("Failed to decode base64: {e}"),
                                context: None,
                            });
                        }
                    }
                }
                *items = new_items;
            }

            Command::ToBase64 => {
                for item in items.iter_mut() {
                    let bytes = item.to_bytes();
                    let encoded = base64_encode_bytes(&bytes);
                    *item = Item::Bytes(encoded);
                }
            }

            Command::SplitShell => {
                let mut new_items = Vec::new();
                for item in items.drain(..) {
                    let bytes = item.to_bytes();
                    let token_result = crate::shell_tokenizer::tokenize(&bytes);
                    for error in &token_result.errors {
                        self.warnings.push(PipelineWarning {
                            node_index: node.index,
                            message: format!("Shell tokenizer warning: {error}"),
                            coercion: None,
                        });
                    }
                    for arg in token_result.args {
                        new_items.push(Item::Bytes(arg));
                    }
                }
                *items = new_items;
            }
        }

        Ok(())
    }
}

impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}

fn base64_decode(input: &[u8]) -> Result<Vec<u8>, String> {
    const DECODE_TABLE: [i8; 256] = {
        let mut table = [-1i8; 256];
        let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut i = 0;
        while i < 64 {
            table[alphabet[i] as usize] = i as i8;
            i += 1;
        }
        table
    };

    let mut output = Vec::new();
    let mut buffer: u32 = 0;
    let mut bits = 0;

    for &byte in input {
        if byte == b'=' || byte.is_ascii_whitespace() {
            continue;
        }

        let value = DECODE_TABLE[byte as usize];
        if value < 0 {
            return Err(format!("Invalid base64 character: {}", byte as char));
        }

        buffer = (buffer << 6) | (value as u32);
        bits += 6;

        if bits >= 8 {
            bits -= 8;
            output.push((buffer >> bits) as u8);
            buffer &= (1 << bits) - 1;
        }
    }

    Ok(output)
}

fn base64_encode_bytes(input: &[u8]) -> Vec<u8> {
    const ALPHABET: &[u8; 64] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut output = Vec::new();

    for chunk in input.chunks(3) {
        let b0 = chunk.first().copied().unwrap_or(0);
        let b1 = chunk.get(1).copied().unwrap_or(0);
        let b2 = chunk.get(2).copied().unwrap_or(0);

        output.push(ALPHABET[(b0 >> 2) as usize]);
        output.push(ALPHABET[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize]);

        if chunk.len() > 1 {
            output.push(ALPHABET[(((b1 & 0x0f) << 2) | (b2 >> 6)) as usize]);
        } else {
            output.push(b'=');
        }

        if chunk.len() > 2 {
            output.push(ALPHABET[(b2 & 0x3f) as usize]);
        } else {
            output.push(b'=');
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_input_count() {
        assert!(matches!(Command::Stdin.input_count(), InputCount::Zero));
        assert!(matches!(Command::Stdout.input_count(), InputCount::One));
        assert!(matches!(Command::Chain.input_count(), InputCount::Many));
    }

    #[test]
    fn test_command_output_count() {
        assert!(matches!(Command::Stdout.output_count(), OutputCount::Zero));
        assert!(matches!(Command::Stdin.output_count(), OutputCount::One));
    }

    #[test]
    fn test_pipeline_parse_simple() {
        let args: Vec<String> = vec!["sort".to_string()];
        let pipeline = Pipeline::parse(&args);

        // Should have implicit stdin and stdout
        assert!(pipeline.nodes.iter().any(|n| n.command == Command::Stdin && n.implicit));
        assert!(pipeline.nodes.iter().any(|n| n.command == Command::Stdout && n.implicit));
    }

    #[test]
    fn test_exit_status_calculation() {
        assert_eq!(exit_status_for_node(0), 63);
        assert_eq!(exit_status_for_node(1), 64);
        assert_eq!(exit_status_for_node(33), 96);
        assert_eq!(exit_status_for_node(100), 96); // Capped at 96
    }

    #[test]
    fn test_base64_roundtrip() {
        let original = b"Hello, World!";
        let encoded = base64_encode_bytes(original);
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(decoded, original);
    }
}
