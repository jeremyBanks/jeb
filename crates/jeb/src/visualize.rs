// Pipeline visualization with ASCII art

use crate::pipeline::Pipeline;
use std::collections::HashMap;

/// Generate ASCII visualization of the pipeline
pub fn visualize_pipeline(pipeline: &Pipeline, terminal_width: usize) -> String {
    let mut output = String::new();

    if pipeline.nodes.is_empty() {
        return output;
    }

    // Build a simple linear representation for now
    // TODO: Handle more complex DAG structures with branching

    // Create a mapping of node positions
    let mut node_lines = Vec::new();

    for (idx, node) in pipeline.nodes.iter().enumerate() {
        let node_name = &node.label;

        // Check if this node has inputs
        let has_input = pipeline.input_edges.get(&idx).map_or(false, |v| !v.is_empty());

        // Check if this node has outputs
        let has_output = pipeline.output_edges.get(&idx).map_or(false, |v| !v.is_empty());

        let line = if idx == 0 && !has_input {
            // Source node
            format!("{} →", node_name)
        } else if !has_output {
            // Sink node
            format!("→ {}", node_name)
        } else {
            // Transform node
            format!("→ {} →", node_name)
        };

        node_lines.push(line);
    }

    // Join the lines with proper spacing
    let visualization = node_lines.join(" ");

    // Wrap if needed
    if visualization.len() > terminal_width {
        // Simple wrapping - split at spaces
        let mut current_line = String::new();
        for part in visualization.split_whitespace() {
            if current_line.len() + part.len() + 1 > terminal_width {
                output.push_str(&current_line);
                output.push('\n');
                current_line = String::new();
            }
            if !current_line.is_empty() {
                current_line.push(' ');
            }
            current_line.push_str(part);
        }
        if !current_line.is_empty() {
            output.push_str(&current_line);
        }
    } else {
        output.push_str(&visualization);
    }

    output
}

/// Generate detailed pipeline visualization with node indices
pub fn visualize_pipeline_detailed(pipeline: &Pipeline) -> String {
    let mut output = String::new();

    output.push_str("Pipeline Nodes:\n");
    for node in &pipeline.nodes {
        output.push_str(&format!("  [{}] {}\n", node.index, node.label));

        // Show input arity
        let (min_in, max_in) = node.node.input_arity();
        let max_in_str = max_in.map_or("∞".to_string(), |n| n.to_string());
        output.push_str(&format!("      inputs: {}-{}\n", min_in, max_in_str));

        // Show output count
        output.push_str(&format!("      outputs: {}\n", node.node.output_count()));
    }

    output.push_str("\nPipeline Edges:\n");
    for edge in &pipeline.edges {
        let from_label = &pipeline.nodes[edge.from_node].label;
        let to_label = &pipeline.nodes[edge.to_node].label;
        output.push_str(&format!(
            "  [{}]:{} ({}) → [{}]:{} ({})\n",
            edge.from_node,
            edge.from_output,
            from_label,
            edge.to_node,
            edge.to_input,
            to_label
        ));
    }

    output
}

/// Colorize pipeline visualization for terminal output
pub fn colorize_pipeline(visualization: &str) -> String {
    // For now, just return the visualization as-is
    // TODO: Add color codes based on node types
    // - Blue for output streams
    // - Red for error streams
    // - Yellow for implicit nodes
    visualization.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{nodes::*, parser::parse_pipeline};

    #[test]
    fn test_visualize_simple_pipeline() {
        let args = vec!["parse-json".to_string(), "to-json".to_string()];
        let pipeline = parse_pipeline(&args).unwrap();

        let vis = visualize_pipeline(&pipeline, 80);
        assert!(vis.contains("stdin"));
        assert!(vis.contains("parse-json"));
        assert!(vis.contains("to-json"));
        assert!(vis.contains("stdout"));
    }

    #[test]
    fn test_visualize_detailed() {
        let args = vec!["parse-json".to_string()];
        let pipeline = parse_pipeline(&args).unwrap();

        let vis = visualize_pipeline_detailed(&pipeline);
        assert!(vis.contains("Pipeline Nodes:"));
        assert!(vis.contains("Pipeline Edges:"));
    }
}
