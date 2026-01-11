//! Non-fatal error collection
//! [impl _trace.errors]

use crate::model::Location;

/// A non-fatal error encountered during processing
#[derive(Debug, Clone)]
pub struct TraceError {
    pub kind: ErrorKind,
    pub location: Location,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorKind {
    /// [impl _trace.errors.duplicate-def]
    DuplicateDef,
    /// [impl _trace.errors.add-existing]
    AddExisting,
    /// [impl _trace.errors.remove-missing]
    RemoveMissing,
    /// [impl _trace.errors.unrequired-annotation]
    UnrequiredAnnotation,
    /// [impl _trace.types.def-required]
    DefAsRequired,
}

/// Collects non-fatal errors during processing
/// [impl _trace.errors]
#[derive(Debug, Default)]
pub struct ErrorCollector {
    pub errors: Vec<TraceError>,
}

impl ErrorCollector {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a duplicate definition error
    /// [impl _trace.errors.duplicate-def]
    pub fn duplicate_def(&mut self, location: &Location, id: &str) {
        self.errors.push(TraceError {
            kind: ErrorKind::DuplicateDef,
            location: location.clone(),
            message: format!("Duplicate definition for '{}'", id),
        });
    }

    /// Record an add-existing error
    /// [impl _trace.errors.add-existing]
    pub fn add_existing(&mut self, location: &Location, type_name: &str) {
        self.errors.push(TraceError {
            kind: ErrorKind::AddExisting,
            location: location.clone(),
            message: format!("Adding '{}' which is already required", type_name),
        });
    }

    /// Record a remove-missing error
    /// [impl _trace.errors.remove-missing]
    pub fn remove_missing(&mut self, location: &Location, type_name: &str) {
        self.errors.push(TraceError {
            kind: ErrorKind::RemoveMissing,
            location: location.clone(),
            message: format!("Removing '{}' which is not required", type_name),
        });
    }

    /// Record an unrequired annotation error
    /// [impl _trace.errors.unrequired-annotation]
    pub fn unrequired_annotation(&mut self, location: &Location, type_name: &str, id: &str) {
        self.errors.push(TraceError {
            kind: ErrorKind::UnrequiredAnnotation,
            location: location.clone(),
            message: format!(
                "Annotation type '{}' is not required for '{}'",
                type_name, id
            ),
        });
    }

    /// Record a def-as-required error
    /// [impl _trace.types.def-required]
    pub fn def_as_required(&mut self, location: &Location) {
        self.errors.push(TraceError {
            kind: ErrorKind::DefAsRequired,
            location: location.clone(),
            message: "Cannot require 'def' as a satisfaction type".to_string(),
        });
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn len(&self) -> usize {
        self.errors.len()
    }
}

impl std::fmt::Display for TraceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.location, self.message)
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        std::path::PathBuf,
    };

    /// [test _trace.errors]
    /// [test _trace.errors.duplicate-def]
    #[test]
    fn test_duplicate_def_error() {
        let mut errors = ErrorCollector::new();
        let loc = Location::new(PathBuf::from("test.md"), 10, 5);
        errors.duplicate_def(&loc, "my.requirement");

        assert_eq!(errors.len(), 1);
        assert_eq!(errors.errors[0].kind, ErrorKind::DuplicateDef);
        assert!(errors.errors[0].message.contains("my.requirement"));
    }

    /// [test _trace.errors.add-existing]
    #[test]
    fn test_add_existing_error() {
        let mut errors = ErrorCollector::new();
        let loc = Location::new(PathBuf::from("test.md"), 10, 5);
        errors.add_existing(&loc, "impl");

        assert_eq!(errors.len(), 1);
        assert_eq!(errors.errors[0].kind, ErrorKind::AddExisting);
        assert!(errors.errors[0].message.contains("impl"));
    }

    /// [test _trace.errors.remove-missing]
    #[test]
    fn test_remove_missing_error() {
        let mut errors = ErrorCollector::new();
        let loc = Location::new(PathBuf::from("test.md"), 10, 5);
        errors.remove_missing(&loc, "test");

        assert_eq!(errors.len(), 1);
        assert_eq!(errors.errors[0].kind, ErrorKind::RemoveMissing);
        assert!(errors.errors[0].message.contains("test"));
    }

    /// [test _trace.errors.unrequired-annotation]
    #[test]
    fn test_unrequired_annotation_error() {
        let mut errors = ErrorCollector::new();
        let loc = Location::new(PathBuf::from("test.md"), 10, 5);
        errors.unrequired_annotation(&loc, "doc", "my.req");

        assert_eq!(errors.len(), 1);
        assert_eq!(errors.errors[0].kind, ErrorKind::UnrequiredAnnotation);
        assert!(errors.errors[0].message.contains("doc"));
    }
}
