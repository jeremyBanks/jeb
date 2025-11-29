// Pipeline infrastructure for DAG-based stream processing

use crate::model::{ErrorValue, StreamItem, Warning};
use std::collections::HashMap;

/// Node trait that all pipeline commands implement.
/// Each node has a statically known number of inputs and outputs.
pub trait Node: Send + Sync {
    /// Human-readable name for this node type
    fn name(&self) -> &str;

    /// Number of input streams this node accepts
    /// Returns (min, max) where None for max means unlimited
    fn input_arity(&self) -> (usize, Option<usize>);

    /// Number of output streams this node produces
    fn output_count(&self) -> usize;

    /// Execute the node's transformation
    /// Takes input streams and produces output streams
    fn execute(
        &self,
        inputs: Vec<Stream>,
        node_index: usize,
    ) -> Result<ExecutionResult, NodeError>;
}

/// A stream of items flowing between nodes
pub type Stream = Vec<StreamItem>;

/// Result of executing a node
pub struct ExecutionResult {
    /// Output streams produced by this node
    pub outputs: Vec<Stream>,

    /// Error values generated during execution
    pub errors: Vec<ErrorValue>,

    /// Warnings about implicit coercions
    pub warnings: Vec<Warning>,
}

/// Errors that can occur during node execution
#[derive(Debug)]
pub enum NodeError {
    /// Node received wrong number of inputs
    InvalidInputCount { expected: String, got: usize },

    /// Node received unexpected input type
    UnexpectedInputType { expected: String, got: String },

    /// Internal node error
    Internal(String),

    /// I/O error
    Io(std::io::Error),
}

impl From<std::io::Error> for NodeError {
    fn from(e: std::io::Error) -> Self {
        NodeError::Io(e)
    }
}

/// Represents a node in the pipeline graph
pub struct PipelineNode {
    /// The node implementation
    pub node: Box<dyn Node>,

    /// Index of this node in the pipeline
    pub index: usize,

    /// Human-readable label for this node
    pub label: String,
}

/// An edge connecting two nodes in the pipeline
#[derive(Debug, Clone)]
pub struct Edge {
    /// Source node index
    pub from_node: usize,

    /// Source output index (which of the node's outputs)
    pub from_output: usize,

    /// Destination node index
    pub to_node: usize,

    /// Destination input index (which of the node's inputs)
    pub to_input: usize,
}

/// The complete pipeline graph
pub struct Pipeline {
    /// All nodes in the pipeline
    pub nodes: Vec<PipelineNode>,

    /// All edges connecting nodes
    pub edges: Vec<Edge>,

    /// Mapping from node indices to their output edges
    pub output_edges: HashMap<usize, Vec<Edge>>,

    /// Mapping from node indices to their input edges
    pub input_edges: HashMap<usize, Vec<Edge>>,
}

impl Pipeline {
    /// Create a new empty pipeline
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            output_edges: HashMap::new(),
            input_edges: HashMap::new(),
        }
    }

    /// Add a node to the pipeline, returning its index
    pub fn add_node(&mut self, node: Box<dyn Node>, label: String) -> usize {
        let index = self.nodes.len();
        self.nodes.push(PipelineNode { node, index, label });
        index
    }

    /// Add an edge between two nodes
    pub fn add_edge(&mut self, edge: Edge) {
        self.output_edges
            .entry(edge.from_node)
            .or_default()
            .push(edge.clone());

        self.input_edges
            .entry(edge.to_node)
            .or_default()
            .push(edge.clone());

        self.edges.push(edge);
    }

    /// Execute the pipeline
    pub fn execute(&self) -> Result<PipelineResult, PipelineError> {
        // For now, implement simple sequential execution
        // TODO: Implement true DAG-based async execution

        let mut streams: HashMap<(usize, usize), Stream> = HashMap::new();
        let mut all_warnings = Vec::new();
        let mut all_errors = Vec::new();

        // Topological sort of nodes
        let execution_order = self.topological_sort()?;

        for node_index in execution_order {
            let pipeline_node = &self.nodes[node_index];

            // Gather inputs for this node
            let input_edges = self.input_edges.get(&node_index).cloned().unwrap_or_default();
            let mut inputs = Vec::new();

            for edge in &input_edges {
                if let Some(stream) = streams.get(&(edge.from_node, edge.from_output)) {
                    inputs.push(stream.clone());
                } else {
                    // No input available - this could be a source node or an error
                    if pipeline_node.node.input_arity().0 > 0 {
                        return Err(PipelineError::MissingInput {
                            node: node_index,
                            expected_input: edge.to_input,
                        });
                    }
                }
            }

            // Execute the node
            match pipeline_node.node.execute(inputs, node_index) {
                Ok(result) => {
                    // Store output streams
                    for (output_idx, stream) in result.outputs.into_iter().enumerate() {
                        streams.insert((node_index, output_idx), stream);
                    }

                    all_warnings.extend(result.warnings);
                    all_errors.extend(result.errors);
                }
                Err(e) => {
                    return Err(PipelineError::NodeExecution {
                        node: node_index,
                        error: format!("{:?}", e),
                    });
                }
            }
        }

        Ok(PipelineResult {
            warnings: all_warnings,
            errors: all_errors,
        })
    }

    /// Perform topological sort to determine execution order
    fn topological_sort(&self) -> Result<Vec<usize>, PipelineError> {
        let mut in_degree: HashMap<usize, usize> = HashMap::new();
        let mut result = Vec::new();

        // Calculate in-degree for each node
        for i in 0..self.nodes.len() {
            in_degree.insert(i, 0);
        }

        for edge in &self.edges {
            *in_degree.entry(edge.to_node).or_insert(0) += 1;
        }

        // Find all nodes with in-degree 0
        let mut queue: Vec<usize> = in_degree
            .iter()
            .filter(|&(_, degree)| *degree == 0)
            .map(|(&node, _)| node)
            .collect();

        queue.sort(); // For deterministic ordering

        while let Some(node) = queue.pop() {
            result.push(node);

            // Reduce in-degree for all neighbors
            if let Some(edges) = self.output_edges.get(&node) {
                for edge in edges {
                    if let Some(degree) = in_degree.get_mut(&edge.to_node) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push(edge.to_node);
                            queue.sort();
                        }
                    }
                }
            }
        }

        if result.len() != self.nodes.len() {
            return Err(PipelineError::CyclicGraph);
        }

        Ok(result)
    }
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of executing a pipeline
pub struct PipelineResult {
    /// All warnings generated during execution
    pub warnings: Vec<Warning>,

    /// All errors generated during execution
    pub errors: Vec<ErrorValue>,
}

/// Errors that can occur during pipeline construction or execution
#[derive(Debug)]
pub enum PipelineError {
    /// Pipeline contains a cycle
    CyclicGraph,

    /// Node is missing required input
    MissingInput { node: usize, expected_input: usize },

    /// Error during node execution
    NodeExecution { node: usize, error: String },

    /// Invalid pipeline structure
    InvalidStructure(String),
}
