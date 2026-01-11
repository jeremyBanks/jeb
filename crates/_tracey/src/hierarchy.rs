//! Requirement hierarchy building
//! [impl _trace.hierarchy]

use {
    crate::{
        errors::ErrorCollector,
        model::{
            Annotation,
            Requirement,
            RequirementTree,
            SatisfactionMode,
        },
    },
    std::collections::{
        HashMap,
        HashSet,
    },
};

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
    // [impl _trace.types.def]
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
/// [impl _trace.hierarchy.inheritance]
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
    use {
        super::*,
        crate::model::Location,
        std::path::PathBuf,
    };

    fn make_def(id: &str) -> Annotation {
        Annotation {
            kind: "def".to_string(),
            id: id.to_string(),
            modifiers: Default::default(),
            location: Location::new(PathBuf::from("test.md"), 1, 1),
            context: String::new(),
        }
    }

    fn make_def_with_modifiers(id: &str, add: Vec<&str>, remove: Vec<&str>) -> Annotation {
        Annotation {
            kind: "def".to_string(),
            id: id.to_string(),
            modifiers: crate::model::Modifiers {
                add_types: add.into_iter().map(String::from).collect(),
                remove_types: remove.into_iter().map(String::from).collect(),
                mode: None,
            },
            location: Location::new(PathBuf::from("test.md"), 1, 1),
            context: String::new(),
        }
    }

    /// [test _trace.hierarchy]
    /// [test _trace.hierarchy.implicit-ancestors]
    #[test]
    fn test_implicit_ancestors() {
        let mut errors = ErrorCollector::new();
        let annotations = vec![make_def("req.sub.leaf")];
        let tree = build_tree(annotations, &mut errors);

        assert!(tree.requirements.contains_key("req"));
        assert!(tree.requirements.contains_key("req.sub"));
        assert!(tree.requirements.contains_key("req.sub.leaf"));

        assert!(tree.requirements.get("req").unwrap().implicit);
        assert!(tree.requirements.get("req.sub").unwrap().implicit);
        assert!(!tree.requirements.get("req.sub.leaf").unwrap().implicit);
    }

    /// [test _trace.satisfaction.defaults]
    #[test]
    fn test_default_required_types() {
        let mut errors = ErrorCollector::new();
        let annotations = vec![make_def("myreq")];
        let tree = build_tree(annotations, &mut errors);

        let req = tree.requirements.get("myreq").unwrap();
        assert!(req.required_types.contains("impl"));
        assert!(req.required_types.contains("test"));
    }

    /// [test _trace.hierarchy.inheritance]
    /// [test _trace.satisfaction.inheritance]
    #[test]
    fn test_type_inheritance() {
        let mut errors = ErrorCollector::new();
        // Parent adds +doc, child should inherit it
        let annotations = vec![
            make_def_with_modifiers("parent", vec!["doc"], vec![]),
            make_def("parent.child"),
        ];
        let tree = build_tree(annotations, &mut errors);

        let parent = tree.requirements.get("parent").unwrap();
        assert!(parent.required_types.contains("doc"));

        let child = tree.requirements.get("parent.child").unwrap();
        assert!(
            child.required_types.contains("doc"),
            "Child should inherit +doc from parent"
        );
    }

    /// [test _trace.satisfaction.modifier-descendants]
    #[test]
    fn test_modifier_descendants() {
        let mut errors = ErrorCollector::new();
        // Parent removes -test, child should inherit that removal
        let annotations = vec![
            make_def_with_modifiers("parent", vec![], vec!["test"]),
            make_def("parent.child"),
        ];
        let tree = build_tree(annotations, &mut errors);

        let child = tree.requirements.get("parent.child").unwrap();
        assert!(
            !child.required_types.contains("test"),
            "Child should inherit -test from parent"
        );
        assert!(
            child.required_types.contains("impl"),
            "Child should still have impl"
        );
    }

    /// [test _trace.hierarchy.completion]
    #[test]
    fn test_parent_child_relationships() {
        let mut errors = ErrorCollector::new();
        let annotations = vec![make_def("root"), make_def("root.a"), make_def("root.b")];
        let tree = build_tree(annotations, &mut errors);

        let root = tree.requirements.get("root").unwrap();
        assert!(root.children.contains(&"root.a".to_string()));
        assert!(root.children.contains(&"root.b".to_string()));

        let child_a = tree.requirements.get("root.a").unwrap();
        assert_eq!(child_a.parent, Some("root".to_string()));
    }

    /// [test _trace.types.def-required]
    #[test]
    fn test_def_as_required_type_is_error() {
        use crate::errors::ErrorKind;

        let mut errors = ErrorCollector::new();
        // Try to add +def as a required type - should produce an error
        let annotations = vec![Annotation {
            kind: "def".to_string(),
            id: "bad.requirement".to_string(),
            modifiers: crate::model::Modifiers {
                add_types: vec!["def".to_string()],
                remove_types: vec![],
                mode: None,
            },
            location: Location::new(PathBuf::from("test.md"), 1, 1),
            context: String::new(),
        }];
        let _tree = build_tree(annotations, &mut errors);

        assert!(!errors.is_empty(), "Should have an error for +def");
        assert!(
            errors
                .errors
                .iter()
                .any(|e| e.kind == ErrorKind::DefAsRequired),
            "Error should be DefAsRequired"
        );
    }
}
