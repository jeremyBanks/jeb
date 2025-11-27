// New jeb pipeline-based CLI (demonstration)

use jeb::{parser::parse_pipeline, visualize::{visualize_pipeline, visualize_pipeline_detailed}};
use std::process;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    // Check for special flags
    if args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_help();
        return;
    }

    let show_detailed = args.contains(&"--visualize-detailed".to_string());
    let dry_run = args.contains(&"--dry-run".to_string());

    // Filter out flags from the pipeline args
    let pipeline_args: Vec<String> = args
        .into_iter()
        .filter(|a| !a.starts_with("--") && !a.starts_with("-"))
        .collect();

    // Parse the pipeline
    let pipeline = match parse_pipeline(&pipeline_args) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error parsing pipeline: {:?}", e);
            process::exit(1);
        }
    };

    // Visualize the pipeline
    if show_detailed {
        eprintln!("{}", visualize_pipeline_detailed(&pipeline));
    } else {
        let term_width = term_size::dimensions().map(|(w, _)| w).unwrap_or(80);
        eprintln!("{}", visualize_pipeline(&pipeline, term_width));
    }

    if dry_run {
        eprintln!("\n(Dry run - not executing pipeline)");
        return;
    }

    // Execute the pipeline
    match pipeline.execute() {
        Ok(result) => {
            // Print warnings if any
            if !result.warnings.is_empty() {
                eprintln!("\nWarnings:");
                for warning in &result.warnings {
                    eprintln!("  [{}] {}", warning.node_index, warning.message);
                }
            }

            // Print errors if any
            if !result.errors.is_empty() {
                eprintln!("\nErrors:");
                for error in &result.errors {
                    eprintln!("  [{}] {}", error.node_index, error.message);
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
            eprintln!("\nPipeline execution failed: {:?}", e);
            process::exit(1);
        }
    }
}

fn print_help() {
    println!(
        r#"jeb-pipeline - Pipeline-based data processing tool

USAGE:
    jeb-pipeline [OPTIONS] [COMMANDS...]

OPTIONS:
    -h, --help                  Show this help message
    --visualize-detailed        Show detailed pipeline visualization
    --dry-run                   Parse and visualize pipeline without executing

PIPELINE COMMANDS:

Sources (no inputs, one output):
    stdin                       Read from standard input
    ./path or /path            Read from file

Sinks (one input, no outputs):
    stdout                      Write to standard output
    stderr                      Write to standard error

Parsers (consume Bytes/Text, emit Structured):
    parse-json                  Parse JSON data
    from-base64                 Decode base64 data

Serializers (consume Structured, emit Text/Bytes):
    to-json                     Serialize to JSON
    to-base64                   Encode as base64

Chunking:
    by-lines                    Split by lines (keeping newlines)
    split-lines                 Split by lines (removing newlines)
    by-null                     Split by null bytes (keeping them)
    split-null                  Split by null bytes (removing them)
    join-lines                  Join items with newlines

Aggregation:
    join-array                  Collect stream into single array
    split-array                 Emit each array element separately

Stream combining:
    chain                       Concatenate multiple streams
    merge                       Merge sorted streams

Transforms:
    sort                        Sort stream items
    filter                      Filter stream items

EXAMPLES:
    # Parse JSON from stdin and pretty-print to stdout
    jeb-pipeline parse-json to-json

    # Read file, parse JSON, sort, and output
    jeb-pipeline ./data.json parse-json sort to-json

    # Split input by lines and count
    jeb-pipeline split-lines

    # Visualize pipeline without executing
    jeb-pipeline --dry-run parse-json sort to-json

EXIT STATUS:
    0       Success, no warnings
    63-96   Warnings or errors (63 + node_index where first issue occurred)
"#
    );
}
