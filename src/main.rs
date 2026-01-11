//! _trace CLI
//! [impl _trace.cli]

use clap::Parser;
use std::env;
use std::path::PathBuf;
use std::process;

use _trace::{
    build_tree, compute_satisfaction, extract_contexts, install_skills, parse_annotations,
    print_errors, print_list, print_summary, scan_files, ErrorCollector, OutputOptions,
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

    /// Show all requirements (including complete ones)
    #[arg(long)]
    all: bool,

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

    /// Install Claude Code skills to .claude/skills/
    /// [impl _trace.cli.install-skills]
    #[arg(long)]
    install_skills: bool,
}

fn main() {
    let args = Args::parse();

    // Get the current directory as root
    let root = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    // Handle --install-skills
    // [impl _trace.cli.install-skills]
    if args.install_skills {
        match install_skills(&root) {
            Ok(()) => {
                process::exit(0);
            }
            Err(e) => {
                eprintln!("Error installing skills: {}", e);
                process::exit(1);
            }
        }
    }

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

    // Determine output mode
    let options = OutputOptions {
        limit: args.limit,
        skip: args.skip,
        show_context: args.context,
        show_lines: args.lines,
        context_of: args.context_of,
        filter_types: args.types,
        filter_prefixes: args.prefixes,
        show_all: args.all,
        show_incomplete: !args.all, // Show incomplete by default
    };

    // Count incomplete requirements
    let incomplete_count = statuses.values().filter(|s| !s.complete).count();

    if incomplete_count == 0 && !args.all {
        // All requirements complete - show success summary
        // [impl _trace.cli.default-output]
        print_summary(&tree, &statuses, &errors);
    } else {
        // Show list with options (incomplete items by default)
        // [impl _trace.cli.list]
        print_list(&tree, &statuses, &options);
    }
}
