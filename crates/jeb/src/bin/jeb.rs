// jeb: Pipeline-based data processing tool

use jeb::{
    parser::parse_pipeline,
    visualize::visualize_pipeline,
};
use owo_colors::OwoColorize;
use std::process;

fn main() {
    let mut args: Vec<String> = std::env::args().collect();
    let program_path = args.remove(0);

    // Check for help flags
    if args.is_empty() || args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_help();
        return;
    }

    // Check for special flags
    let show_detailed = args.contains(&"--visualize-detailed".to_string());
    let dry_run = args.contains(&"--dry-run".to_string());
    let quiet = args.contains(&"--quiet".to_string()) || args.contains(&"-q".to_string());

    // Filter out flags from the pipeline args
    let pipeline_args: Vec<String> = args
        .into_iter()
        .filter(|a| !matches!(a.as_str(), "--visualize-detailed" | "--dry-run" | "--quiet" | "-q"))
        .collect();

    // Parse the pipeline
    let pipeline = match parse_pipeline(&pipeline_args) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{}: {:?}", "error".red(), e);
            process::exit(1);
        }
    };

    // Visualize the pipeline with colors
    if !quiet {
        if show_detailed {
            eprintln!("{}", jeb::visualize::visualize_pipeline_detailed(&pipeline));
        } else {
            let term_width = term_size::dimensions().map(|(w, _)| w).unwrap_or(80);
            let visualization = visualize_pipeline(&pipeline, term_width);

            // Color the visualization with original jeb's style
            let colored = colorize_visualization(&program_path, &visualization, &pipeline_args);
            eprintln!("{}", colored);
        }
    }

    if dry_run {
        eprintln!("\n{}", "(Dry run - not executing pipeline)".yellow());
        return;
    }

    // Execute the pipeline
    match pipeline.execute() {
        Ok(result) => {
            // Print warnings if any
            if !result.warnings.is_empty() && !quiet {
                eprintln!("\n{}:", "Warnings".yellow());
                for warning in &result.warnings {
                    eprintln!("  {} {}", format!("[{}]", warning.node_index).yellow(), warning.message);
                }
            }

            // Print errors if any
            if !result.errors.is_empty() && !quiet {
                eprintln!("\n{}:", "Errors".red());
                for error in &result.errors {
                    eprintln!("  {} {}", format!("[{}]", error.node_index).red(), error.message);
                }
            }

            // Calculate exit status based on first error (63 + node_index, capped at 96)
            let exit_code = if !result.errors.is_empty() {
                let first_error_node = result.errors[0].node_index;
                (63 + first_error_node).min(96) as i32
            } else if !result.warnings.is_empty() {
                let first_warning_node = result.warnings[0].node_index;
                (63 + first_warning_node).min(96) as i32
            } else {
                0
            };

            process::exit(exit_code);
        }
        Err(e) => {
            eprintln!("\n{}: {:?}", "Pipeline execution failed".red(), e);
            process::exit(1);
        }
    }
}

fn colorize_visualization(program_path: &str, visualization: &str, original_args: &[String]) -> String {
    // Create a colored version similar to original jeb
    let colored_program = program_path.magenta().to_string();

    // Color command names in yellow, implicit commands in red
    let mut colored_vis = String::new();
    let parts: Vec<&str> = visualization.split_whitespace().collect();

    for (i, part) in parts.iter().enumerate() {
        if i > 0 {
            colored_vis.push(' ');
        }

        // Check if this command was in the original args
        let was_explicit = original_args.iter().any(|a| part.contains(a.as_str()));

        if part.contains("stdin") || part.contains("stdout") {
            // Implicit commands in red if not explicitly provided
            if was_explicit {
                colored_vis.push_str(&part.yellow().to_string());
            } else {
                colored_vis.push_str(&part.red().to_string());
            }
        } else if *part == "→" {
            // Arrows in dim
            colored_vis.push_str(&part.dimmed().to_string());
        } else {
            // Explicit commands in yellow
            colored_vis.push_str(&part.yellow().to_string());
        }
    }

    format!("{} {}", colored_program, colored_vis)
}

fn print_help() {
    println!(
        r#"{}

{}
    jeb [OPTIONS] [COMMANDS...]

{}
    {}          Show this help message
    {}      Show detailed pipeline visualization
    {}              Parse and visualize pipeline without executing
    {}       Suppress pipeline visualization

{} - Sources (no inputs, one output):
    {}                  Read from standard input
    {}                   Read the current executable
    {}             Read from file

{} - Sinks (one input, no outputs):
    {}                Write to standard output
    {}                Write to standard error

{} - Parsers (consume Bytes/Text, emit Structured):
    {}            Parse JSON data from text/bytes
    {}          Decode base64 data

{} - Serializers (consume Structured, emit Text/Bytes):
    {}               Serialize structured data to JSON
    {}            Encode data as base64

{} - Encoding (JEB-specific):
    {}           Encode using Z85 (ASCII-85 variant)
    {}         Encode using JEB85 (text-preserving Z85)

{} - Chunking:
    {}              Split by lines (keeping newlines)
    {}          Split by lines (removing newlines)
    {}              Split by null bytes (keeping them)
    {}          Split by null bytes (removing them)
    {}            Join items with newlines

{} - Aggregation:
    {}            Collect stream into single JSON array
    {}          Emit each array element separately
    {}                 Join all bytes together
    {}           Join items with spaces

{} - Stream Combining:
    {}                 Concatenate multiple streams in order
    {}                 Merge sorted streams (maintains order)

{} - Transforms:
    {}                  Sort stream items
    {}                Filter non-empty items
    {}             Collapse consecutive whitespace to single space

{} - Selection:
    {}                 Keep only first item
    {}                  Keep only last item
    {}              Keep first N items (e.g., first-10)
    {}               Keep last N items (e.g., last-5)

{}
    # Parse JSON from stdin and pretty-print to stdout
    echo '{{"hello":"world"}}' | jeb parse-json to-json

    # Read file, parse JSON, sort, and output
    jeb ./data.json parse-json sort to-json

    # Split input by lines
    echo -e 'line1\\nline2\\nline3' | jeb split-lines

    # Encode binary data with Z85
    cat binary.dat | jeb encode-z85

    # Get first 100 lines of a file
    jeb ./large.txt split-lines first-100

    # Collapse whitespace in text
    echo 'hello    world' | jeb collapse

    # Visualize pipeline without executing
    jeb --dry-run parse-json sort to-json

    # Quiet mode (no visualization)
    jeb -q ./data.json parse-json to-json

{}
    0       Success, no warnings
    63-96   Warnings or errors (63 + node_index where first issue occurred)

{}
    jeb implements a pipeline architecture where data flows through transformations.
    The pipeline is built using stack-based connection resolution - outputs from
    previous commands connect to inputs of subsequent commands.

    Commands are automatically connected:
    - If no source exists, 'stdin' is prepended
    - If no sink exists, 'stdout' is appended
    - Multiple outputs are combined with 'chain' before the sink

    Three data types flow through pipelines:
    - Text: Unparsed UTF-8 text
    - Bytes: Raw binary data
    - Structured: Parsed data (extended JSON model)
"#,
        "jeb - Pipeline-based data processing".bold(),
        "USAGE:".bold(),
        "OPTIONS:".bold(),
        "-h, --help".green(),
        "--visualize-detailed".green(),
        "--dry-run".green(),
        "-q, --quiet".green(),
        "SOURCES".bold(),
        "stdin".cyan(),
        "self".cyan(),
        "./path or /path".cyan(),
        "SINKS".bold(),
        "stdout".cyan(),
        "stderr".cyan(),
        "PARSERS".bold(),
        "parse-json".cyan(),
        "from-base64".cyan(),
        "SERIALIZERS".bold(),
        "to-json".cyan(),
        "to-base64".cyan(),
        "ENCODING".bold(),
        "encode-z85".cyan(),
        "encode-jeb85".cyan(),
        "CHUNKING".bold(),
        "by-lines".cyan(),
        "split-lines".cyan(),
        "by-null".cyan(),
        "split-null".cyan(),
        "join-lines".cyan(),
        "AGGREGATION".bold(),
        "join-array".cyan(),
        "split-array".cyan(),
        "join".cyan(),
        "join-space".cyan(),
        "COMBINING".bold(),
        "chain".cyan(),
        "merge".cyan(),
        "TRANSFORMS".bold(),
        "sort".cyan(),
        "filter".cyan(),
        "collapse".cyan(),
        "SELECTION".bold(),
        "first".cyan(),
        "last".cyan(),
        "first-N".cyan(),
        "last-N".cyan(),
        "EXAMPLES:".bold(),
        "EXIT STATUS:".bold(),
        "ARCHITECTURE:".bold(),
    );
}
