//! Requirement hierarchy building
//! [impl _trace.hierarchy]

use crate::errors::ErrorCollector;
use crate::model::{Annotation, Requirement, RequirementTree, SatisfactionMode};
use std::collections::{HashMap, HashSet};

/// Default required types
/// [impl _trace.satisfaction.defaults]
fn default_required_types() -> HashSet<String> {
    let mut types = HashSet::new();
    types.insert("impl".to_string());
    types.insert("test".to_string());
    types
}

/// Build the requirement tree from annotations
/// [impl _trace.hierarchy]
/// [impl _trace.hierarchy.implicit-ancestors]
pub fn build_tree(annotations: Vec<Annotation>, errors: &mut ErrorCollector) -> RequirementTree {
    let mut tree = RequirementTree::new();

    // Separate def annotations from others
    let (defs, others): (Vec<_>, Vec<_>) = annotations.into_iter().partition(|a| a.kind == "def");

    // First pass: create explicit requirements from defs
    // [impl _trace.errors.duplicate-def]
    let mut seen_defs: HashMap<String, Annotation> = HashMap::new();
    for def in defs {
        if let Some(existing) = seen_defs.get(&def.id) {
            // Duplicate def - keep the one with lexicographically first path
            if def.location.file < existing.location.file {
                errors.duplicate_def(&existing.location, &def.id);
                seen_defs.insert(def.id.clone(), def);
            } else {
                errors.duplicate_def(&def.location, &def.id);
            }
        } else {
            seen_defs.insert(def.id.clone(), def);
        }
    }

    // Create requirements from defs
    for (id, def) in seen_defs {
        let mut req = Requirement::new(id.clone());
        req.implicit = false;
        req.definition = Some(def.clone());
        req.mode = def.modifiers.mode.unwrap_or(SatisfactionMode::Either);
        tree.requirements.insert(id, req);
    }

    // Second pass: create implicit ancestors
    // [impl _trace.hierarchy.implicit-ancestors]
    let explicit_ids: Vec<String> = tree.requirements.keys().cloned().collect();
    for id in explicit_ids {
        ensure_ancestors(&mut tree, &id);
    }

    // Third pass: compute inherited required_types
    // [impl _trace.satisfaction.inheritance]
    // [impl _trace.satisfaction.modifier-descendants]
    compute_inherited_types(&mut tree, errors);

    // Fourth pass: build parent-child relationships
    build_parent_child(&mut tree);

    // Fifth pass: attach non-def annotations to requirements
    for annotation in others {
        if let Some(req) = tree.requirements.get_mut(&annotation.id) {
            req.annotations
                .entry(annotation.kind.clone())
                .or_default()
                .push(annotation);
        }
        // Note: annotations for undefined requirements are silently ignored
        // (they reference requirements that don't exist)
    }

    tree
}

/// Ensure all ancestors of an ID exist
/// [impl _trace.hierarchy.implicit-ancestors]
fn ensure_ancestors(tree: &mut RequirementTree, id: &str) {
    let parts: Vec<&str> = id.split('.').collect();

    for i in 1..parts.len() {
        let ancestor_id = parts[..i].join(".");
        if !tree.requirements.contains_key(&ancestor_id) {
            let req = Requirement::new(ancestor_id.clone());
            tree.requirements.insert(ancestor_id, req);
        }
    }
}

/// Compute inherited required_types for all requirements
/// [impl _trace.satisfaction.inheritance]
/// [impl _trace.satisfaction.modifier-descendants]
fn compute_inherited_types(tree: &mut RequirementTree, errors: &mut ErrorCollector) {
    // Get all IDs sorted by depth (parents before children)
    let mut ids: Vec<String> = tree.requirements.keys().cloned().collect();
    ids.sort_by_key(|id| id.matches('.').count());

    for id in ids {
        let parent_types = if let Some(parent_id) = get_parent_id(&id) {
            tree.requirements
                .get(&parent_id)
                .map(|p| p.required_types.clone())
                .unwrap_or_else(default_required_types)
        } else {
            default_required_types()
        };

        let req = tree.requirements.get_mut(&id).unwrap();

        // Start with parent's types
        req.required_types = parent_types;

        // Apply modifiers if this is an explicit def
        if let Some(ref def) = req.definition {
            // Add types
            for add_type in &def.modifiers.add_types {
                // [impl _trace.types.def-required]
                if add_type == "def" {
                    errors.def_as_required(&def.location);
                    continue;
                }
                // [impl _trace.errors.add-existing]
                if req.required_types.contains(add_type) {
                    errors.add_existing(&def.location, add_type);
                }
                req.required_types.insert(add_type.clone());
            }

            // Remove types
            for remove_type in &def.modifiers.remove_types {
                // [impl _trace.errors.remove-missing]
                if !req.required_types.contains(remove_type) {
                    errors.remove_missing(&def.location, remove_type);
                }
                req.required_types.remove(remove_type);
            }
        }
    }
}

/// Build parent-child relationships
fn build_parent_child(tree: &mut RequirementTree) {
    let ids: Vec<String> = tree.requirements.keys().cloned().collect();

    for id in &ids {
        if let Some(parent_id) = get_parent_id(id) {
            // Add this as a child of parent
            if let Some(parent) = tree.requirements.get_mut(&parent_id) {
                if !parent.children.contains(id) {
                    parent.children.push(id.clone());
                }
            }
            // Set parent reference
            if let Some(req) = tree.requirements.get_mut(id) {
                req.parent = Some(parent_id);
            }
        } else {
            // This is a root
            if !tree.roots.contains(id) {
                tree.roots.push(id.clone());
            }
        }
    }

    // Sort children and roots for deterministic output
    for req in tree.requirements.values_mut() {
        req.children.sort();
    }
    tree.roots.sort();
}

/// Get the parent ID from a requirement ID
fn get_parent_id(id: &str) -> Option<String> {
    let parts: Vec<&str> = id.split('.').collect();
    if parts.len() > 1 {
        Some(parts[..parts.len() - 1].join("."))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Location;
    use std::path::PathBuf;

    fn make_def(id: &str) -> Annotation {
        Annotation {
            kind: "def".to_string(),
            id: id.to_string(),
            modifiers: Default::default(),
            location: Location::new(PathBuf::from("test.md"), 1, 1),
            context: String::new(),
        }
    }

    #[test]
    fn test_implicit_ancestors() {
        let mut errors = ErrorCollector::new();
        let annotations = vec![make_def("foo.bar.baz")];
        let tree = build_tree(annotations, &mut errors);

        assert!(tree.requirements.contains_key("foo"));
        assert!(tree.requirements.contains_key("foo.bar"));
        assert!(tree.requirements.contains_key("foo.bar.baz"));

        assert!(tree.requirements.get("foo").unwrap().implicit);
        assert!(tree.requirements.get("foo.bar").unwrap().implicit);
        assert!(!tree.requirements.get("foo.bar.baz").unwrap().implicit);
    }

    #[test]
    fn test_default_required_types() {
        let mut errors = ErrorCollector::new();
        let annotations = vec![make_def("foo")];
        let tree = build_tree(annotations, &mut errors);

        let req = tree.requirements.get("foo").unwrap();
        assert!(req.required_types.contains("impl"));
        assert!(req.required_types.contains("test"));
    }
}
