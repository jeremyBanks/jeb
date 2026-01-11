//! Data structures for _trace
//! [impl _trace.location]
//! [impl _trace.types]

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// Location of an annotation in a file
/// [impl _trace.location.file]
/// [impl _trace.location.line]
/// [impl _trace.location.column]
#[derive(Debug, Clone)]
pub struct Location {
    pub file: PathBuf,
    pub line: usize,   // 1-indexed
    pub column: usize, // 1-indexed
}

impl Location {
    pub fn new(file: PathBuf, line: usize, column: usize) -> Self {
        Self { file, line, column }
    }
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.file.display(), self.line, self.column)
    }
}

/// Satisfaction mode for a requirement
/// [impl _trace.satisfaction.mode]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SatisfactionMode {
    /// Requirement must be satisfied by annotations on this exact ID
    Self_,
    /// Requirement is satisfied when all children are satisfied
    Children,
    /// Either self or children satisfaction works (default)
    #[default]
    Either,
}

/// Modifiers parsed from a def annotation
/// [impl _trace.satisfaction.modifiers]
#[derive(Debug, Clone, Default)]
pub struct Modifiers {
    pub add_types: Vec<String>,
    pub remove_types: Vec<String>,
    pub mode: Option<SatisfactionMode>,
}

/// A parsed annotation
/// [impl _trace.syntax]
#[derive(Debug, Clone)]
pub struct Annotation {
    /// The annotation type (def, impl, test, etc.)
    pub kind: String,
    /// The requirement ID
    pub id: String,
    /// Modifiers (only meaningful for def annotations)
    pub modifiers: Modifiers,
    /// Location in source file
    pub location: Location,
    /// Extracted context text
    pub context: String,
}

/// A requirement (defined or implicit)
/// [impl _trace.hierarchy]
#[derive(Debug, Clone)]
pub struct Requirement {
    /// The requirement ID
    pub id: String,
    /// The definition annotation, if explicitly defined
    pub definition: Option<Annotation>,
    /// Whether this requirement was implicitly created
    pub implicit: bool,
    /// Required annotation types for satisfaction
    pub required_types: HashSet<String>,
    /// Satisfaction mode
    pub mode: SatisfactionMode,
    /// Annotations by type (excluding def)
    pub annotations: HashMap<String, Vec<Annotation>>,
    /// Direct child requirement IDs
    pub children: Vec<String>,
    /// Parent requirement ID, if any
    pub parent: Option<String>,
}

impl Requirement {
    pub fn new(id: String) -> Self {
        Self {
            id,
            definition: None,
            implicit: true,
            required_types: HashSet::new(),
            mode: SatisfactionMode::Either,
            annotations: HashMap::new(),
            children: Vec::new(),
            parent: None,
        }
    }

    /// Get the parent ID from this requirement's ID
    /// [impl _trace.syntax.id.hierarchy]
    pub fn parent_id(&self) -> Option<String> {
        let parts: Vec<&str> = self.id.split('.').collect();
        if parts.len() > 1 {
            Some(parts[..parts.len() - 1].join("."))
        } else {
            None
        }
    }
}

/// The complete requirement tree
#[derive(Debug, Default)]
pub struct RequirementTree {
    /// All requirements by ID
    pub requirements: HashMap<String, Requirement>,
    /// Root requirement IDs (those with no parent)
    pub roots: Vec<String>,
}

impl RequirementTree {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get all requirement IDs in hierarchical order (depth-first)
    pub fn iter_depth_first(&self) -> Vec<&String> {
        let mut result = Vec::new();
        let mut sorted_roots: Vec<_> = self.roots.iter().collect();
        sorted_roots.sort();
        for root in sorted_roots {
            self.collect_depth_first(root, &mut result);
        }
        result
    }

    fn collect_depth_first<'a>(&'a self, id: &'a String, result: &mut Vec<&'a String>) {
        result.push(id);
        if let Some(req) = self.requirements.get(id) {
            let mut sorted_children: Vec<_> = req.children.iter().collect();
            sorted_children.sort();
            for child_id in sorted_children {
                self.collect_depth_first(child_id, result);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [test _trace.location]
    /// [test _trace.location.file]
    /// [test _trace.location.line]
    /// [test _trace.location.column]
    #[test]
    fn test_location_display() {
        let loc = Location::new(PathBuf::from("src/main.rs"), 42, 10);
        let display = format!("{}", loc);
        assert!(display.contains("src/main.rs"));
        assert!(display.contains("42"));
        assert!(display.contains("10"));
        assert_eq!(display, "src/main.rs:42:10");
    }

    /// [test _trace.location.line]
    /// [test _trace.location.column]
    #[test]
    fn test_location_is_1_indexed() {
        let loc = Location::new(PathBuf::from("test.rs"), 1, 1);
        // Line and column should be 1-indexed (first line is 1, not 0)
        assert_eq!(loc.line, 1);
        assert_eq!(loc.column, 1);
    }

    /// [test _trace.types]
    /// [test _trace.types.custom]
    #[test]
    fn test_custom_annotation_types() {
        // Annotations can have any type string
        let annotation = Annotation {
            kind: "custom-type".to_string(),
            id: "some.req".to_string(),
            modifiers: Modifiers::default(),
            location: Location::new(PathBuf::from("test.md"), 1, 1),
            context: String::new(),
        };
        assert_eq!(annotation.kind, "custom-type");
    }

    /// [test _trace.types.def]
    #[test]
    fn test_def_type() {
        let def = Annotation {
            kind: "def".to_string(),
            id: "my.requirement".to_string(),
            modifiers: Modifiers::default(),
            location: Location::new(PathBuf::from("spec.md"), 10, 1),
            context: "This is the requirement definition".to_string(),
        };
        assert_eq!(def.kind, "def");
    }

    /// [test _trace.syntax.id.hierarchy]
    #[test]
    fn test_parent_id() {
        let req = Requirement::new("a.b.c".to_string());
        assert_eq!(req.parent_id(), Some("a.b".to_string()));

        let root = Requirement::new("root".to_string());
        assert_eq!(root.parent_id(), None);
    }
}
