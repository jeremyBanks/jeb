// Parser for building pipeline DAGs from command-line arguments

use crate::{
    nodes::*,
    pipeline::{Edge, Pipeline},
};

/// Parser state during pipeline construction
struct ParserState {
    /// The pipeline being built
    pipeline: Pipeline,

    /// Stack of unconnected outputs: (node_index, output_index)
    unconnected_outputs: Vec<(usize, usize)>,

    /// Current argument index (for error reporting)
    arg_index: usize,
}

impl ParserState {
    fn new() -> Self {
        Self {
            pipeline: Pipeline::new(),
            unconnected_outputs: Vec::new(),
            arg_index: 0,
        }
    }

    /// Add a node to the pipeline and connect it according to stack-based rules
    fn add_node_with_connections(
        &mut self,
        node: Box<dyn crate::pipeline::Node>,
        label: String,
    ) -> Result<usize, ParseError> {
        let (min_inputs, max_inputs) = node.input_arity();
        let output_count = node.output_count();

        let node_index = self.pipeline.add_node(node, label);

        // Connect inputs using stack-based resolution
        let inputs_needed = if max_inputs.is_none() {
            // If node accepts unlimited inputs, consume all unconnected outputs
            self.unconnected_outputs.len()
        } else {
            // Otherwise, consume min_inputs (or all available, whichever is less)
            min_inputs.min(self.unconnected_outputs.len())
        };

        // Connect left-to-right (FIFO), matching conceptual model
        // Drain from the front to preserve ordering
        let drained_outputs: Vec<(usize, usize)> = self
            .unconnected_outputs
            .drain(0..inputs_needed)
            .collect();
        for (input_idx, (from_node, from_output)) in drained_outputs.into_iter().enumerate() {
            self.pipeline
                .add_edge(Edge {
                    from_node,
                    from_output,
                    to_node: node_index,
                    to_input: input_idx,
                })
                .map_err(|e| ParseError::InvalidArgument {
                    message: format!("Failed to add edge: {:?}", e),
                    position: self.arg_index,
                })?;
        }

        // Add this node's outputs to the stack
        for output_idx in 0..output_count {
            self.unconnected_outputs.push((node_index, output_idx));
        }

        Ok(node_index)
    }
}

/// Parse command-line arguments into a pipeline
pub fn parse_pipeline(args: &[String]) -> Result<Pipeline, ParseError> {
    let mut state = ParserState::new();

    // Check if we need an implicit stdin source
    // If the first command is not a source node, add stdin
    let needs_stdin = if !args.is_empty() {
        let first_arg = &args[0];
        !is_source_node(first_arg)
    } else {
        true // Empty pipeline needs stdin
    };

    if needs_stdin {
        let stdin_index = state.pipeline.add_node(Box::new(StdinNode), "stdin".to_string());
        state.unconnected_outputs.push((stdin_index, 0));
    }

    for (idx, arg) in args.iter().enumerate() {
        state.arg_index = idx;

        // Parse the argument and create the appropriate node
        let node: Box<dyn crate::pipeline::Node> = match arg.as_str() {
            // Sources
            "stdin" => Box::new(StdinNode),
            "self" => Box::new(SelfNode),

            // Sinks
            "stdout" => Box::new(StdoutNode),
            "stderr" => Box::new(StderrNode),

            // Parsers
            "parse-json" => Box::new(ParseJsonNode),
            "from-base64" => Box::new(FromBase64Node),

            // Serializers
            "to-json" => Box::new(ToJsonNode),
            "to-base64" => Box::new(ToBase64Node),

            // Encoding
            "encode-z85" => Box::new(EncodeZ85Node),
            "encode-jeb85" => Box::new(EncodeJeb85Node),

            // Chunking
            "by-lines" => Box::new(ByLinesNode),
            "split-lines" => Box::new(SplitLinesNode),
            "by-null" => Box::new(ByNullNode),
            "split-null" => Box::new(SplitNullNode),

            // Aggregation
            "join-array" => Box::new(JoinArrayNode),
            "split-array" => Box::new(SplitArrayNode),
            "join-lines" => Box::new(JoinLinesNode),
            "join-space" => Box::new(JoinSpaceNode),
            "join" => Box::new(JoinNode),

            // Stream combining
            "chain" => Box::new(ChainNode),
            "merge" => Box::new(MergeNode),

            // Transforms
            "sort" => Box::new(SortNode),
            "filter" => Box::new(FilterNode),
            "collapse" => Box::new(CollapseNode),

            // Selection
            "first" => Box::new(FirstNode),
            "last" => Box::new(LastNode),

            // File literals (paths starting with . or /)
            path if path.starts_with('.') || path.starts_with('/') => {
                Box::new(FileSourceNode::new(path.to_string()))
            }

            // Parametrized commands
            arg if arg.starts_with("first-") => {
                let n_str = &arg[6..];
                let n: usize = n_str.parse().map_err(|_| ParseError::InvalidArgument {
                    message: format!("Invalid number: {}", n_str),
                    position: idx,
                })?;
                Box::new(FirstNNode::new(n))
            }

            arg if arg.starts_with("last-") => {
                let n_str = &arg[5..];
                let n: usize = n_str.parse().map_err(|_| ParseError::InvalidArgument {
                    message: format!("Invalid number: {}", n_str),
                    position: idx,
                })?;
                Box::new(LastNNode::new(n))
            }

            _ => {
                return Err(ParseError::UnknownCommand {
                    command: arg.clone(),
                    position: idx,
                });
            }
        };

        state.add_node_with_connections(node, arg.clone())?;
    }

    // Apply implicit commands to ensure well-formed pipeline
    finalize_pipeline(&mut state)?;

    Ok(state.pipeline)
}

/// Check if a command is a source node (has no inputs)
fn is_source_node(command: &str) -> bool {
    matches!(command, "stdin" | "self") || command.starts_with('.') || command.starts_with('/')
}

/// Finalize the pipeline by adding implicit commands
fn finalize_pipeline(state: &mut ParserState) -> Result<(), ParseError> {
    // Note: stdin is now added at the beginning if needed, not here

    // If there are unconnected outputs, we need to add a sink
    if !state.unconnected_outputs.is_empty() {
        // Check if the last node is already a sink
        let last_node_is_sink = state
            .pipeline
            .nodes
            .last()
            .map(|n| n.node.output_count() == 0)
            .unwrap_or(false);

        if !last_node_is_sink {
            // If multiple unconnected outputs, add chain first
            if state.unconnected_outputs.len() > 1 {
                let chain_index = state
                    .pipeline
                    .add_node(Box::new(ChainNode), "chain".to_string());

                // Connect all unconnected outputs to chain
                for (input_idx, (from_node, from_output)) in
                    state.unconnected_outputs.drain(..).enumerate()
                {
                    state
                        .pipeline
                        .add_edge(Edge {
                            from_node,
                            from_output,
                            to_node: chain_index,
                            to_input: input_idx,
                        })
                        .map_err(|e| ParseError::InvalidArgument {
                            message: format!("Failed to add edge: {:?}", e),
                            position: state.arg_index,
                        })?;
                }

                state.unconnected_outputs.push((chain_index, 0));
            }

            // Add stdout sink
            let stdout_index = state
                .pipeline
                .add_node(Box::new(StdoutNode), "stdout".to_string());

            if let Some((from_node, from_output)) = state.unconnected_outputs.pop() {
                state
                    .pipeline
                    .add_edge(Edge {
                        from_node,
                        from_output,
                        to_node: stdout_index,
                        to_input: 0,
                    })
                    .map_err(|e| ParseError::InvalidArgument {
                        message: format!("Failed to add edge: {:?}", e),
                        position: state.arg_index,
                    })?;
            }
        }
    }

    Ok(())
}

/// Errors that can occur during pipeline parsing
#[derive(Debug)]
pub enum ParseError {
    /// Unknown command
    UnknownCommand { command: String, position: usize },

    /// Invalid argument format
    InvalidArgument { message: String, position: usize },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_args_adds_implicit_stdin_stdout() {
        let args: Vec<String> = vec![];
        let pipeline = parse_pipeline(&args).unwrap();

        // Should have stdin and stdout
        assert_eq!(pipeline.nodes.len(), 2);
        assert_eq!(pipeline.nodes[0].label, "stdin");
        assert_eq!(pipeline.nodes[1].label, "stdout");
    }

    #[test]
    fn test_single_transform_adds_stdin_stdout() {
        let args = vec!["parse-json".to_string()];
        let pipeline = parse_pipeline(&args).unwrap();

        // Should have stdin, parse-json, stdout
        assert_eq!(pipeline.nodes.len(), 3);
        assert_eq!(pipeline.nodes[0].label, "stdin");
        assert_eq!(pipeline.nodes[1].label, "parse-json");
        assert_eq!(pipeline.nodes[2].label, "stdout");
    }

    #[test]
    fn test_source_node_no_duplicate_stdin() {
        let args = vec!["stdin".to_string(), "parse-json".to_string()];
        let pipeline = parse_pipeline(&args).unwrap();

        // Should have stdin, parse-json, stdout (no duplicate stdin)
        assert_eq!(pipeline.nodes.len(), 3);
        assert_eq!(pipeline.nodes[0].label, "stdin");
        assert_eq!(pipeline.nodes[1].label, "parse-json");
        assert_eq!(pipeline.nodes[2].label, "stdout");
    }

    #[test]
    fn test_file_path_detection() {
        // Relative path
        let args = vec!["./test.txt".to_string()];
        let pipeline = parse_pipeline(&args).unwrap();
        assert_eq!(pipeline.nodes[0].label, "./test.txt");
        assert_eq!(pipeline.nodes[0].node.name(), "file");

        // Absolute path
        let args = vec!["/tmp/test.txt".to_string()];
        let pipeline = parse_pipeline(&args).unwrap();
        assert_eq!(pipeline.nodes[0].label, "/tmp/test.txt");
        assert_eq!(pipeline.nodes[0].node.name(), "file");
    }

    #[test]
    fn test_parametrized_first_n() {
        let args = vec!["first-5".to_string()];
        let pipeline = parse_pipeline(&args).unwrap();

        // Should have stdin, first-5, stdout
        assert_eq!(pipeline.nodes.len(), 3);
        assert_eq!(pipeline.nodes[1].label, "first-5");
    }

    #[test]
    fn test_parametrized_last_n() {
        let args = vec!["last-10".to_string()];
        let pipeline = parse_pipeline(&args).unwrap();

        // Should have stdin, last-10, stdout
        assert_eq!(pipeline.nodes.len(), 3);
        assert_eq!(pipeline.nodes[1].label, "last-10");
    }

    #[test]
    fn test_unknown_command_error() {
        let args = vec!["unknown-command".to_string()];
        let result = parse_pipeline(&args);

        assert!(matches!(
            result,
            Err(ParseError::UnknownCommand { command, position: 0 }) if command == "unknown-command"
        ));
    }

    #[test]
    fn test_invalid_first_n_argument() {
        let args = vec!["first-abc".to_string()];
        let result = parse_pipeline(&args);

        assert!(matches!(result, Err(ParseError::InvalidArgument { .. })));
    }

    #[test]
    fn test_sink_no_implicit_stdout() {
        let args = vec!["parse-json".to_string(), "stdout".to_string()];
        let pipeline = parse_pipeline(&args).unwrap();

        // Should have stdin, parse-json, stdout (no duplicate stdout)
        assert_eq!(pipeline.nodes.len(), 3);
        assert_eq!(pipeline.nodes[2].label, "stdout");
    }

    #[test]
    fn test_edge_connections_fifo_order() {
        // When multiple sources feed into a multi-input node,
        // they should be connected in left-to-right (FIFO) order
        let args = vec!["parse-json".to_string(), "to-json".to_string()];
        let pipeline = parse_pipeline(&args).unwrap();

        // Verify that there are edges connecting nodes
        assert!(!pipeline.edges.is_empty());

        // Check first edge connects stdin to parse-json
        let first_edge = &pipeline.edges[0];
        assert_eq!(first_edge.from_node, 0); // stdin
        assert_eq!(first_edge.to_node, 1); // parse-json
    }

    #[test]
    fn test_chain_multiple_transforms() {
        let args = vec![
            "by-lines".to_string(),
            "sort".to_string(),
            "join-lines".to_string(),
        ];
        let pipeline = parse_pipeline(&args).unwrap();

        // Should have stdin, by-lines, sort, join-lines, stdout
        assert_eq!(pipeline.nodes.len(), 5);
        assert_eq!(pipeline.edges.len(), 4);
    }

    #[test]
    fn test_self_source() {
        let args = vec!["self".to_string(), "to-base64".to_string()];
        let pipeline = parse_pipeline(&args).unwrap();

        // self is a source, so no implicit stdin
        assert_eq!(pipeline.nodes[0].label, "self");
        assert_eq!(pipeline.nodes[0].node.name(), "self");
    }
}
