//! Satisfaction computation
//! [impl _trace.satisfaction]

use crate::model::{RequirementTree, SatisfactionMode};
use std::collections::HashMap;

/// Satisfaction status for a requirement
#[derive(Debug, Clone)]
pub struct SatisfactionStatus {
    /// Is this requirement satisfied?
    pub satisfied: bool,
    /// Is this requirement complete (satisfied + all descendants complete)?
    pub complete: bool,
    /// Types that are satisfied
    pub satisfied_types: Vec<String>,
    /// Types that are missing
    pub missing_types: Vec<String>,
}

/// Compute satisfaction for all requirements
/// [impl _trace.satisfaction]
/// [impl _trace.satisfaction.mode.self]
/// [impl _trace.satisfaction.mode.child]
/// [impl _trace.satisfaction.mode.either]
pub fn compute_satisfaction(tree: &RequirementTree) -> HashMap<String, SatisfactionStatus> {
    let mut statuses: HashMap<String, SatisfactionStatus> = HashMap::new();

    // Process in reverse depth order (leaves first, then parents)
    let mut ids: Vec<String> = tree.requirements.keys().cloned().collect();
    ids.sort_by_key(|id| std::cmp::Reverse(id.matches('.').count()));

    for id in ids {
        let status = compute_requirement_satisfaction(tree, &id, &statuses);
        statuses.insert(id, status);
    }

    // Second pass: compute completion (requires children to be computed first)
    let ids_for_completion: Vec<String> = statuses.keys().cloned().collect();
    for id in ids_for_completion {
        let complete = is_complete(tree, &id, &statuses);
        if let Some(status) = statuses.get_mut(&id) {
            status.complete = complete;
        }
    }

    statuses
}

/// Compute satisfaction for a single requirement
fn compute_requirement_satisfaction(
    tree: &RequirementTree,
    id: &str,
    child_statuses: &HashMap<String, SatisfactionStatus>,
) -> SatisfactionStatus {
    let req = match tree.requirements.get(id) {
        Some(r) => r,
        None => {
            return SatisfactionStatus {
                satisfied: false,
                complete: false,
                satisfied_types: vec![],
                missing_types: vec![],
            }
        }
    };

    let mut satisfied_types = Vec::new();
    let mut missing_types = Vec::new();

    // Check @self satisfaction
    // [impl _trace.satisfaction.mode.self]
    let self_satisfied = check_self_satisfaction(req, &mut satisfied_types, &mut missing_types);

    // Check @child satisfaction
    // [impl _trace.satisfaction.mode.child]
    let child_satisfied = check_child_satisfaction(tree, id, child_statuses);

    // Determine overall satisfaction based on mode
    // [impl _trace.satisfaction.mode]
    let satisfied = match req.mode {
        SatisfactionMode::Self_ => self_satisfied,
        SatisfactionMode::Child => child_satisfied,
        SatisfactionMode::Either => self_satisfied || child_satisfied,
    };

    SatisfactionStatus {
        satisfied,
        complete: false, // Computed in second pass
        satisfied_types,
        missing_types,
    }
}

/// Check if requirement is satisfied by direct annotations (@self)
/// [impl _trace.satisfaction.mode.self]
fn check_self_satisfaction(
    req: &crate::model::Requirement,
    satisfied_types: &mut Vec<String>,
    missing_types: &mut Vec<String>,
) -> bool {
    let mut all_satisfied = true;

    for required_type in &req.required_types {
        if req
            .annotations
            .get(required_type)
            .map(|v| !v.is_empty())
            .unwrap_or(false)
        {
            satisfied_types.push(required_type.clone());
        } else {
            missing_types.push(required_type.clone());
            all_satisfied = false;
        }
    }

    // Sort for deterministic output
    satisfied_types.sort();
    missing_types.sort();

    all_satisfied
}

/// Check if requirement is satisfied by children (@child)
/// [impl _trace.satisfaction.mode.child]
fn check_child_satisfaction(
    tree: &RequirementTree,
    id: &str,
    child_statuses: &HashMap<String, SatisfactionStatus>,
) -> bool {
    let req = match tree.requirements.get(id) {
        Some(r) => r,
        None => return false,
    };

    // Must have at least one child
    if req.children.is_empty() {
        return false;
    }

    // For each required type, check that:
    // 1. At least one child requires this type
    // 2. All children that require this type are satisfied (overall, not just self)
    for required_type in &req.required_types {
        let mut has_child_requiring_type = false;
        let mut all_requiring_satisfied = true;

        for child_id in &req.children {
            if let Some(child_req) = tree.requirements.get(child_id) {
                if child_req.required_types.contains(required_type) {
                    has_child_requiring_type = true;

                    // Check if this child is satisfied overall (not just self)
                    // A child can be satisfied via @self, @child, or @either
                    if let Some(child_status) = child_statuses.get(child_id) {
                        if !child_status.satisfied {
                            all_requiring_satisfied = false;
                        }
                    } else {
                        all_requiring_satisfied = false;
                    }
                }
            }
        }

        // Both conditions must be met for this type
        if !has_child_requiring_type || !all_requiring_satisfied {
            return false;
        }
    }

    true
}

/// Check if a requirement is complete (satisfied + all descendants complete)
/// [impl _trace.hierarchy.completion]
fn is_complete(
    tree: &RequirementTree,
    id: &str,
    statuses: &HashMap<String, SatisfactionStatus>,
) -> bool {
    // Must be satisfied
    let status = match statuses.get(id) {
        Some(s) => s,
        None => return false,
    };

    if !status.satisfied {
        return false;
    }

    // All children must be complete
    if let Some(req) = tree.requirements.get(id) {
        for child_id in &req.children {
            if let Some(child_status) = statuses.get(child_id) {
                if !child_status.complete {
                    return false;
                }
            } else {
                return false;
            }
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::ErrorCollector;
    use crate::hierarchy::build_tree;
    use crate::model::{Annotation, Location, Modifiers, SatisfactionMode};
    use std::path::PathBuf;

    fn make_annotation(kind: &str, id: &str) -> Annotation {
        Annotation {
            kind: kind.to_string(),
            id: id.to_string(),
            modifiers: Modifiers::default(),
            location: Location::new(PathBuf::from("test.md"), 1, 1),
            context: String::new(),
        }
    }

    fn make_def_with_mode(id: &str, mode: SatisfactionMode) -> Annotation {
        Annotation {
            kind: "def".to_string(),
            id: id.to_string(),
            modifiers: Modifiers {
                add_types: vec![],
                remove_types: vec![],
                mode: Some(mode),
            },
            location: Location::new(PathBuf::from("test.md"), 1, 1),
            context: String::new(),
        }
    }

    /// [test _trace.satisfaction]
    /// [test _trace.satisfaction.mode.self]
    #[test]
    fn test_self_satisfaction() {
        let mut errors = ErrorCollector::new();
        let annotations = vec![
            make_annotation("def", "myreq"),
            make_annotation("impl", "myreq"),
            make_annotation("test", "myreq"),
        ];

        let tree = build_tree(annotations, &mut errors);
        let statuses = compute_satisfaction(&tree);

        let status = statuses.get("myreq").unwrap();
        assert!(status.satisfied);
        assert!(status.complete);
        assert!(status.missing_types.is_empty());
    }

    /// [test _trace.satisfaction]
    #[test]
    fn test_missing_type() {
        let mut errors = ErrorCollector::new();
        let annotations = vec![
            make_annotation("def", "myreq"),
            make_annotation("impl", "myreq"),
            // missing test
        ];

        let tree = build_tree(annotations, &mut errors);
        let statuses = compute_satisfaction(&tree);

        let status = statuses.get("myreq").unwrap();
        assert!(!status.satisfied);
        assert!(!status.complete);
        assert_eq!(status.missing_types, vec!["test"]);
    }

    /// [test _trace.satisfaction.mode]
    /// [test _trace.satisfaction.mode.either]
    #[test]
    fn test_either_mode_self() {
        // @either mode: satisfied by self
        let mut errors = ErrorCollector::new();
        let annotations = vec![
            make_annotation("def", "either.req"),
            make_annotation("impl", "either.req"),
            make_annotation("test", "either.req"),
        ];

        let tree = build_tree(annotations, &mut errors);
        let statuses = compute_satisfaction(&tree);

        let status = statuses.get("either.req").unwrap();
        assert!(status.satisfied, "@either should be satisfied by self annotations");
    }

    /// [test _trace.satisfaction.mode.child]
    #[test]
    fn test_child_mode() {
        let mut errors = ErrorCollector::new();
        let annotations = vec![
            make_def_with_mode("parent", SatisfactionMode::Child),
            make_annotation("def", "parent.a"),
            make_annotation("impl", "parent.a"),
            make_annotation("test", "parent.a"),
            make_annotation("def", "parent.b"),
            make_annotation("impl", "parent.b"),
            make_annotation("test", "parent.b"),
        ];

        let tree = build_tree(annotations, &mut errors);
        let statuses = compute_satisfaction(&tree);

        // Children should be satisfied
        assert!(statuses.get("parent.a").unwrap().satisfied);
        assert!(statuses.get("parent.b").unwrap().satisfied);

        // Parent with @child should be satisfied when all children are satisfied
        let parent_status = statuses.get("parent").unwrap();
        assert!(parent_status.satisfied, "@child mode should be satisfied when children are satisfied");
    }

    /// [test _trace.satisfaction.mode.child]
    #[test]
    fn test_child_mode_no_children() {
        let mut errors = ErrorCollector::new();
        let annotations = vec![
            make_def_with_mode("lonely", SatisfactionMode::Child),
        ];

        let tree = build_tree(annotations, &mut errors);
        let statuses = compute_satisfaction(&tree);

        let status = statuses.get("lonely").unwrap();
        assert!(!status.satisfied, "@child mode with no children should not be satisfied");
    }
}
