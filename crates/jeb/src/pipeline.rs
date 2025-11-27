// Pipeline infrastructure for DAG-based stream processing

use crate::model::{ErrorValue, StreamItem, Warning};
use std::collections::{HashMap, VecDeque};

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

    /// Add an edge between two nodes with validation
    pub fn add_edge(&mut self, edge: Edge) -> Result<(), PipelineError> {
        // Validate node indices
        if edge.from_node >= self.nodes.len() {
            return Err(PipelineError::InvalidStructure(format!(
                "Invalid from_node index {}: only {} nodes exist",
                edge.from_node,
                self.nodes.len()
            )));
        }
        if edge.to_node >= self.nodes.len() {
            return Err(PipelineError::InvalidStructure(format!(
                "Invalid to_node index {}: only {} nodes exist",
                edge.to_node,
                self.nodes.len()
            )));
        }

        // Validate output index
        let from_outputs = self.nodes[edge.from_node].node.output_count();
        if edge.from_output >= from_outputs {
            return Err(PipelineError::InvalidStructure(format!(
                "Invalid from_output index {}: node {} only has {} outputs",
                edge.from_output, edge.from_node, from_outputs
            )));
        }

        // Validate input index (check against node's max inputs)
        let (_, max_inputs) = self.nodes[edge.to_node].node.input_arity();
        if let Some(max) = max_inputs {
            if edge.to_input >= max {
                return Err(PipelineError::InvalidStructure(format!(
                    "Invalid to_input index {}: node {} accepts at most {} inputs",
                    edge.to_input, edge.to_node, max
                )));
            }
        }

        self.output_edges
            .entry(edge.from_node)
            .or_default()
            .push(edge.clone());

        self.input_edges
            .entry(edge.to_node)
            .or_default()
            .push(edge.clone());

        self.edges.push(edge);
        Ok(())
    }

    /// Execute the pipeline
    pub fn execute(&self) -> Result<PipelineResult, PipelineError> {
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

        // Find all nodes with in-degree 0 and use VecDeque for proper FIFO ordering
        let mut initial_nodes: Vec<usize> = in_degree
            .iter()
            .filter(|&(_, degree)| *degree == 0)
            .map(|(&node, _)| node)
            .collect();

        // Sort for deterministic ordering
        initial_nodes.sort();
        let mut queue: VecDeque<usize> = initial_nodes.into_iter().collect();

        while let Some(node) = queue.pop_front() {
            result.push(node);

            // Reduce in-degree for all neighbors
            if let Some(edges) = self.output_edges.get(&node) {
                let mut new_nodes = Vec::new();
                for edge in edges {
                    if let Some(degree) = in_degree.get_mut(&edge.to_node) {
                        *degree -= 1;
                        if *degree == 0 {
                            new_nodes.push(edge.to_node);
                        }
                    }
                }
                // Sort new nodes before adding to queue for deterministic ordering
                new_nodes.sort();
                for n in new_nodes {
                    queue.push_back(n);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::StreamItem;
    use crate::nodes::{ChainNode, ParseJsonNode, SortNode, ToJsonNode};

    /// A simple pass-through node for testing
    struct PassThroughNode;

    impl Node for PassThroughNode {
        fn name(&self) -> &str {
            "pass-through"
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
            Ok(ExecutionResult {
                outputs: inputs,
                errors: Vec::new(),
                warnings: Vec::new(),
            })
        }
    }

    /// A source node that produces a fixed stream
    struct TestSourceNode {
        items: Vec<StreamItem>,
    }

    impl Node for TestSourceNode {
        fn name(&self) -> &str {
            "test-source"
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
            Ok(ExecutionResult {
                outputs: vec![self.items.clone()],
                errors: Vec::new(),
                warnings: Vec::new(),
            })
        }
    }

    /// A sink node that does nothing (for testing)
    struct TestSinkNode;

    impl Node for TestSinkNode {
        fn name(&self) -> &str {
            "test-sink"
        }

        fn input_arity(&self) -> (usize, Option<usize>) {
            (1, Some(1))
        }

        fn output_count(&self) -> usize {
            0
        }

        fn execute(
            &self,
            _inputs: Vec<Stream>,
            _node_index: usize,
        ) -> Result<ExecutionResult, NodeError> {
            Ok(ExecutionResult {
                outputs: Vec::new(),
                errors: Vec::new(),
                warnings: Vec::new(),
            })
        }
    }

    #[test]
    fn test_empty_pipeline() {
        let pipeline = Pipeline::new();
        let result = pipeline.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_single_source_node() {
        let mut pipeline = Pipeline::new();
        let source = TestSourceNode {
            items: vec![StreamItem::Text("hello".to_string())],
        };
        pipeline.add_node(Box::new(source), "source".to_string());

        let result = pipeline.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_source_to_sink() {
        let mut pipeline = Pipeline::new();

        let source = TestSourceNode {
            items: vec![StreamItem::Text("hello".to_string())],
        };
        let source_idx = pipeline.add_node(Box::new(source), "source".to_string());

        let sink_idx = pipeline.add_node(Box::new(TestSinkNode), "sink".to_string());

        pipeline
            .add_edge(Edge {
                from_node: source_idx,
                from_output: 0,
                to_node: sink_idx,
                to_input: 0,
            })
            .expect("Failed to add edge");

        let result = pipeline.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_topological_sort_order() {
        let mut pipeline = Pipeline::new();

        // Create a chain: source -> pass1 -> pass2 -> sink
        let source = TestSourceNode {
            items: vec![StreamItem::Text("test".to_string())],
        };
        let source_idx = pipeline.add_node(Box::new(source), "source".to_string());
        let pass1_idx = pipeline.add_node(Box::new(PassThroughNode), "pass1".to_string());
        let pass2_idx = pipeline.add_node(Box::new(PassThroughNode), "pass2".to_string());
        let sink_idx = pipeline.add_node(Box::new(TestSinkNode), "sink".to_string());

        pipeline
            .add_edge(Edge {
                from_node: source_idx,
                from_output: 0,
                to_node: pass1_idx,
                to_input: 0,
            })
            .unwrap();
        pipeline
            .add_edge(Edge {
                from_node: pass1_idx,
                from_output: 0,
                to_node: pass2_idx,
                to_input: 0,
            })
            .unwrap();
        pipeline
            .add_edge(Edge {
                from_node: pass2_idx,
                from_output: 0,
                to_node: sink_idx,
                to_input: 0,
            })
            .unwrap();

        let order = pipeline.topological_sort().unwrap();
        // Verify source comes first, sink comes last
        assert_eq!(order[0], source_idx);
        assert_eq!(order[order.len() - 1], sink_idx);

        // Verify execution succeeds
        let result = pipeline.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_cycle_detection() {
        let mut pipeline = Pipeline::new();

        // Create a cycle: A -> B -> A
        let node_a = pipeline.add_node(Box::new(PassThroughNode), "A".to_string());
        let node_b = pipeline.add_node(Box::new(PassThroughNode), "B".to_string());

        pipeline
            .add_edge(Edge {
                from_node: node_a,
                from_output: 0,
                to_node: node_b,
                to_input: 0,
            })
            .unwrap();
        pipeline
            .add_edge(Edge {
                from_node: node_b,
                from_output: 0,
                to_node: node_a,
                to_input: 0,
            })
            .unwrap();

        let result = pipeline.topological_sort();
        assert!(matches!(result, Err(PipelineError::CyclicGraph)));
    }

    #[test]
    fn test_multiple_sources() {
        let mut pipeline = Pipeline::new();

        // Two independent sources feeding into a chain node
        let source1 = TestSourceNode {
            items: vec![StreamItem::Text("a".to_string())],
        };
        let source2 = TestSourceNode {
            items: vec![StreamItem::Text("b".to_string())],
        };

        let source1_idx = pipeline.add_node(Box::new(source1), "source1".to_string());
        let source2_idx = pipeline.add_node(Box::new(source2), "source2".to_string());
        let chain_idx = pipeline.add_node(Box::new(ChainNode), "chain".to_string());
        let sink_idx = pipeline.add_node(Box::new(TestSinkNode), "sink".to_string());

        pipeline
            .add_edge(Edge {
                from_node: source1_idx,
                from_output: 0,
                to_node: chain_idx,
                to_input: 0,
            })
            .unwrap();
        pipeline
            .add_edge(Edge {
                from_node: source2_idx,
                from_output: 0,
                to_node: chain_idx,
                to_input: 1,
            })
            .unwrap();
        pipeline
            .add_edge(Edge {
                from_node: chain_idx,
                from_output: 0,
                to_node: sink_idx,
                to_input: 0,
            })
            .unwrap();

        let result = pipeline.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_edge_validation_invalid_from_node() {
        let mut pipeline = Pipeline::new();
        pipeline.add_node(Box::new(PassThroughNode), "node".to_string());

        let result = pipeline.add_edge(Edge {
            from_node: 999, // Invalid
            from_output: 0,
            to_node: 0,
            to_input: 0,
        });

        assert!(matches!(result, Err(PipelineError::InvalidStructure(_))));
    }

    #[test]
    fn test_edge_validation_invalid_to_node() {
        let mut pipeline = Pipeline::new();
        pipeline.add_node(Box::new(PassThroughNode), "node".to_string());

        let result = pipeline.add_edge(Edge {
            from_node: 0,
            from_output: 0,
            to_node: 999, // Invalid
            to_input: 0,
        });

        assert!(matches!(result, Err(PipelineError::InvalidStructure(_))));
    }

    #[test]
    fn test_edge_validation_invalid_output_index() {
        let mut pipeline = Pipeline::new();
        let source = TestSourceNode {
            items: vec![],
        };
        let source_idx = pipeline.add_node(Box::new(source), "source".to_string());
        let sink_idx = pipeline.add_node(Box::new(TestSinkNode), "sink".to_string());

        let result = pipeline.add_edge(Edge {
            from_node: source_idx,
            from_output: 999, // Invalid - source only has 1 output
            to_node: sink_idx,
            to_input: 0,
        });

        assert!(matches!(result, Err(PipelineError::InvalidStructure(_))));
    }

    #[test]
    fn test_disconnected_nodes() {
        let mut pipeline = Pipeline::new();

        // Add two disconnected nodes
        let source = TestSourceNode {
            items: vec![StreamItem::Text("test".to_string())],
        };
        pipeline.add_node(Box::new(source), "source".to_string());
        pipeline.add_node(Box::new(TestSinkNode), "sink".to_string());

        // Both nodes should execute successfully even without connection
        let result = pipeline.execute();
        assert!(result.is_ok());
    }
}
