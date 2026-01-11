//! _trace CLI
//! [impl _trace.cli]

use clap::Parser;
use std::env;
use std::path::PathBuf;

use _trace::{
    build_tree, compute_satisfaction, extract_contexts, parse_annotations, print_errors,
    print_list, print_summary, scan_files, ErrorCollector, OutputOptions,
};

/// A language-agnostic requirements tracking tool
/// [impl _trace.cli]
#[derive(Parser, Debug)]
#[command(name = "_trace")]
#[command(about = "Track requirements and their satisfaction status")]
#[command(version)]
struct Args {
    /// Filter by requirement ID prefix(es)
    /// [impl _trace.cli.filter-prefix]
    #[arg(value_name = "PREFIX")]
    prefixes: Vec<String>,

    /// Show full context for all annotations
    /// [impl _trace.cli.context]
    #[arg(long)]
    context: bool,

    /// Show only file:line:col for annotations
    /// [impl _trace.cli.lines]
    #[arg(long)]
    lines: bool,

    /// Show context only for specific annotation types (comma-separated)
    /// [impl _trace.cli.context-of]
    #[arg(long, value_delimiter = ',')]
    context_of: Vec<String>,

    /// Filter by annotation type (comma-separated)
    /// [impl _trace.cli.filter-type]
    #[arg(long = "type", value_delimiter = ',')]
    types: Vec<String>,

    /// Maximum number of items to show
    /// [impl _trace.cli.limit]
    #[arg(long, default_value = "32")]
    limit: usize,

    /// Number of items to skip
    /// [impl _trace.cli.pagination]
    #[arg(long, default_value = "0")]
    skip: usize,
}

fn main() {
    let args = Args::parse();

    // Get the current directory as root
    let root = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    // Scan files
    // [impl _trace.files]
    let files = scan_files(&root);

    // Parse annotations from all files
    let mut all_annotations = Vec::new();
    for file in &files {
        let contexts = extract_contexts(&file.content);
        let annotations = parse_annotations(&file.path, &file.content, &contexts);
        all_annotations.extend(annotations);
    }

    // Build requirement tree
    let mut errors = ErrorCollector::new();
    let tree = build_tree(all_annotations, &mut errors);

    // Compute satisfaction
    let statuses = compute_satisfaction(&tree);

    // Print errors first if any
    print_errors(&errors);

    // Check if we should show summary (before moving values)
    let show_summary = args.prefixes.is_empty()
        && !args.context
        && !args.lines
        && args.context_of.is_empty()
        && args.types.is_empty();

    // Determine output mode
    let options = OutputOptions {
        limit: args.limit,
        skip: args.skip,
        show_context: args.context,
        show_lines: args.lines,
        context_of: args.context_of,
        filter_types: args.types,
        filter_prefixes: args.prefixes,
    };

    if show_summary {
        // Default: show summary
        // [impl _trace.cli.default-output]
        print_summary(&tree, &statuses, &errors);
    } else {
        // Show list with options
        // [impl _trace.cli.list]
        print_list(&tree, &statuses, &options);
    }
}
