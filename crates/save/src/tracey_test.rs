//! Test module for verifying tracey integration

use crate::git2::TemporaryRepository;

/// Greets a user with a friendly message
/// [impl test.hello]
pub fn hello(name: &str) -> String {
    format!("Hello, {}!", name)
}

/// Says goodbye to a user
/// [impl test.goodbye]
pub fn goodbye(name: &str) -> String {
    format!("Goodbye, {}!", name)
}

/// Adds two numbers together
/// [impl test.math.add]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// Multiplies two numbers together
/// [impl test.math.multiply]
pub fn multiply(a: i32, b: i32) -> i32 {
    a * b
}

/// Creates a temporary repository for testing
/// [impl test.repo.temporary]
/// [impl test.repo.cleanup]
pub fn create_temp_repo() -> TemporaryRepository {
    TemporaryRepository::new().expect("failed to create temporary repository")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that hello function works correctly
    /// [verify test.hello]
    #[test]
    fn test_hello() {
        assert_eq!(hello("World"), "Hello, World!");
    }

    /// Test that goodbye function works correctly
    /// [verify test.goodbye]
    #[test]
    fn test_goodbye() {
        assert_eq!(goodbye("World"), "Goodbye, World!");
    }

    /// Test that add function works correctly
    /// [verify test.math.add]
    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
        assert_eq!(add(-1, 1), 0);
    }

    /// Test that multiply function works correctly
    /// [verify test.math.multiply]
    #[test]
    fn test_multiply() {
        assert_eq!(multiply(2, 3), 6);
        assert_eq!(multiply(-2, 3), -6);
    }

    /// Test that temporary repository creation works
    /// [verify test.repo.temporary]
    /// [verify test.repo.cleanup]
    #[test]
    fn test_temp_repo() {
        let repo = create_temp_repo();
        assert!(repo.path().exists());
        // Repo will be automatically cleaned up when dropped
    }
}
