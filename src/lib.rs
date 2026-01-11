//! _trace: A language-agnostic requirements tracking tool
//!
//! This crate provides the core functionality for scanning files,
//! parsing annotations, building requirement hierarchies, and
//! computing satisfaction status.

pub mod context;
pub mod errors;
pub mod hierarchy;
pub mod model;
pub mod output;
pub mod parser;
pub mod satisfaction;
pub mod scanner;

pub use context::extract_contexts;
pub use errors::ErrorCollector;
pub use hierarchy::build_tree;
pub use model::{Annotation, Location, Requirement, RequirementTree, SatisfactionMode};
pub use output::{print_errors, print_list, print_summary, OutputOptions};
pub use parser::parse_annotations;
pub use satisfaction::compute_satisfaction;
pub use scanner::scan_files;
