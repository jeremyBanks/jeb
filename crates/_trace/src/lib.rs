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
pub mod skills;

pub use {
    context::extract_contexts,
    errors::ErrorCollector,
    hierarchy::build_tree,
    model::{
        Annotation,
        Location,
        Requirement,
        RequirementTree,
        SatisfactionMode,
    },
    output::{
        OutputOptions,
        print_errors,
        print_list,
        print_summary,
    },
    parser::parse_annotations,
    satisfaction::compute_satisfaction,
    scanner::scan_files,
    skills::install_skills,
};
