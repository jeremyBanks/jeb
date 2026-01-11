//! CLI output formatting
//! [impl _trace.cli]

use crate::errors::ErrorCollector;
use crate::model::RequirementTree;
use crate::satisfaction::SatisfactionStatus;
use std::collections::HashMap;

/// Output options
pub struct OutputOptions {
    pub limit: usize,
    pub skip: usize,
    pub show_context: bool,
    pub show_lines: bool,
    pub context_of: Vec<String>,
    pub filter_types: Vec<String>,
    pub filter_prefixes: Vec<String>,
}

impl Default for OutputOptions {
    fn default() -> Self {
        Self {
            limit: 32,
            skip: 0,
            show_context: false,
            show_lines: false,
            context_of: vec![],
            filter_types: vec![],
            filter_prefixes: vec![],
        }
    }
}

/// Print summary output (default, no args)
/// [impl _trace.cli.default-output]
pub fn print_summary(
    tree: &RequirementTree,
    statuses: &HashMap<String, SatisfactionStatus>,
    errors: &ErrorCollector,
) {
    let total = tree.requirements.len();
    let satisfied = statuses.values().filter(|s| s.satisfied).count();
    let unsatisfied = total - satisfied;

    println!("Requirements: {} total, {} satisfied, {} unsatisfied", total, satisfied, unsatisfied);

    if !errors.is_empty() {
        println!("\nWarnings: {} non-fatal error(s)", errors.len());
    }

    println!();
    print_help_hints();
}

/// Print requirement list
/// [impl _trace.cli.list]
/// [impl _trace.cli.list.format]
/// [impl _trace.cli.limit]
/// [impl _trace.cli.pagination]
pub fn print_list(
    tree: &RequirementTree,
    statuses: &HashMap<String, SatisfactionStatus>,
    options: &OutputOptions,
) {
    let all_ids = tree.iter_depth_first();

    // Filter by prefixes if specified
    // [impl _trace.cli.filter-prefix]
    let filtered: Vec<&String> = if options.filter_prefixes.is_empty() {
        all_ids
    } else {
        all_ids
            .into_iter()
            .filter(|id| {
                options
                    .filter_prefixes
                    .iter()
                    .any(|prefix| id.starts_with(prefix) || *id == prefix)
            })
            .collect()
    };

    let total = filtered.len();

    // Apply pagination
    // [impl _trace.cli.pagination]
    let paginated: Vec<&String> = filtered
        .into_iter()
        .skip(options.skip)
        .take(options.limit)
        .collect();

    let shown = paginated.len();

    for id in &paginated {
        let req = tree.requirements.get(*id).unwrap();
        let status = statuses.get(*id);

        // Calculate indent based on hierarchy depth
        let depth = id.matches('.').count();
        let indent = "  ".repeat(depth);

        // Format status
        let status_str = format_status(status);

        println!("{}- {}: {}", indent, id, status_str);

        // Show context if requested
        // [impl _trace.cli.context]
        // [impl _trace.cli.context-of]
        if options.show_context || should_show_context(&options.context_of, "def") {
            if let Some(ref def) = req.definition {
                if !def.context.is_empty() {
                    println!("{}  Context: {}", indent, def.location);
                    for line in def.context.lines() {
                        println!("{}    {}", indent, line);
                    }
                }
            }
        }

        // Show lines if requested
        // [impl _trace.cli.lines]
        if options.show_lines {
            if let Some(ref def) = req.definition {
                println!("{}  {}", indent, def.location);
            }
            for (kind, annotations) in &req.annotations {
                for ann in annotations {
                    println!("{}  [{}] {}", indent, kind, ann.location);
                }
            }
        }
    }

    // Pagination message
    // [impl _trace.cli.limit]
    if shown < total {
        let remaining = total - options.skip - shown;
        println!();
        println!(
            "Showing {} of {}. Use --skip={} to see {} more.",
            shown,
            total,
            options.skip + shown,
            remaining.min(options.limit)
        );
    }

    println!();
    print_help_hints();
}

/// Print errors
pub fn print_errors(errors: &ErrorCollector) {
    if errors.is_empty() {
        return;
    }

    println!("Warnings:");
    for error in &errors.errors {
        println!("  {}", error);
    }
    println!();
}

/// Format satisfaction status
fn format_status(status: Option<&SatisfactionStatus>) -> String {
    match status {
        None => "unknown".to_string(),
        Some(s) => {
            if s.satisfied {
                if s.satisfied_types.is_empty() {
                    "done (no requirements)".to_string()
                } else {
                    format!("done with {}", s.satisfied_types.join(", "))
                }
            } else if s.missing_types.is_empty() {
                "unsatisfied".to_string()
            } else {
                format!("missing {}", s.missing_types.join(", "))
            }
        }
    }
}

fn should_show_context(context_of: &[String], kind: &str) -> bool {
    context_of.iter().any(|t| t == kind)
}

/// Print help hints at the end of output
/// [impl _trace.cli.help-in-output]
fn print_help_hints() {
    println!("Options: --context, --lines, --limit=N, --skip=N, --type=TYPE");
    println!("Run `_trace <prefix>` to filter by requirement prefix.");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_status_satisfied() {
        let status = SatisfactionStatus {
            satisfied: true,
            complete: true,
            satisfied_types: vec!["impl".to_string(), "test".to_string()],
            missing_types: vec![],
        };
        assert_eq!(format_status(Some(&status)), "done with impl, test");
    }

    #[test]
    fn test_format_status_missing() {
        let status = SatisfactionStatus {
            satisfied: false,
            complete: false,
            satisfied_types: vec!["impl".to_string()],
            missing_types: vec!["test".to_string()],
        };
        assert_eq!(format_status(Some(&status)), "missing test");
    }
}
