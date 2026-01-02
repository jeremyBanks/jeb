//! Test helper utilities for git-zoom integration tests
//!
//! This module provides a `TestRepo` wrapper around git-snapshot's `TemporaryRepository`
//! with git-zoom-specific utilities for running zoom commands, manipulating files, and
//! verifying results.

use git_snapshot::{parse, Commit, HeadState, Repository};
use std::env;
use std::path::Path;
use std::process::Command;

/// Wrapper around git-snapshot's TemporaryRepository with git-zoom-specific utilities
pub struct TestRepo {
    pub temp_repo: git_snapshot::TemporaryRepository,
}

impl TestRepo {
    /// Create a test repository from YAML string
    pub fn from_yaml(yaml: &str) -> Self {
        let snapshot = parse(yaml).expect("Failed to parse YAML");
        let temp_repo = snapshot
            .to_temporary_repository()
            .expect("Failed to create temporary repository");

        Self { temp_repo }
    }

    /// Get the working directory path
    pub fn workdir(&self) -> &Path {
        self.temp_repo
            .workdir()
            .expect("No working directory in repository")
    }

    /// Run a git-zoom command with the given arguments
    ///
    /// Changes to the repository's working directory, runs the git-zoom binary,
    /// then restores the original directory. Returns Ok(()) on success or
    /// Err(stderr) on failure.
    pub fn run_zoom(&self, args: &[&str]) -> Result<(), String> {
        let original_dir = env::current_dir().unwrap();
        env::set_current_dir(self.workdir()).unwrap();

        let result = Command::new(env!("CARGO_BIN_EXE_git-zoom"))
            .args(args)
            .output();

        env::set_current_dir(original_dir).unwrap();

        match result {
            Ok(output) if output.status.success() => Ok(()),
            Ok(output) => Err(String::from_utf8_lossy(&output.stderr).to_string()),
            Err(e) => Err(format!("Failed to execute git-zoom: {}", e)),
        }
    }

    /// Read the current repository state back to a snapshot
    pub fn to_snapshot(&self) -> Repository {
        self.temp_repo
            .to_snapshot()
            .expect("Failed to read snapshot")
    }

    /// Read a file's contents from the working directory
    pub fn read_file(&self, path: &str) -> String {
        std::fs::read_to_string(self.workdir().join(path))
            .unwrap_or_else(|_| panic!("Failed to read file: {}", path))
    }

    /// Check if a file exists in the working directory
    pub fn file_exists(&self, path: &str) -> bool {
        self.workdir().join(path).exists()
    }

    /// Write content to a file in the working directory
    pub fn write_file(&self, path: &str, content: &str) {
        let full_path = self.workdir().join(path);

        // Create parent directories if needed
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)
                .unwrap_or_else(|_| panic!("Failed to create parent directories for: {}", path));
        }

        std::fs::write(&full_path, content)
            .unwrap_or_else(|_| panic!("Failed to write file: {}", path));
    }

    /// Add all changes and create a commit with the given message
    pub fn git_add_and_commit(&self, message: &str) {
        let original_dir = env::current_dir().unwrap();
        env::set_current_dir(self.workdir()).unwrap();

        // Git add all
        let add_result = Command::new("git").args(&["add", "."]).output();

        // Git commit
        let commit_result = Command::new("git")
            .args(&["commit", "-m", message])
            .output();

        env::set_current_dir(original_dir).unwrap();

        add_result.expect("Failed to run git add");
        commit_result.expect("Failed to run git commit");
    }
}

/// Verify that a commit has the structure of a zoom-in commit
///
/// Zoom-in commits should:
/// - Be merge commits (2 parents)
/// - Have "git-zoom-in:" in the message
/// - Have committer name "🔎"
pub fn verify_zoom_in_commit(commit: &Commit) {
    assert_eq!(
        commit.parents.len(),
        2,
        "Zoom-in commit should be a merge commit with 2 parents, got {}",
        commit.parents.len()
    );

    assert!(
        commit.message.contains("git-zoom-in:"),
        "Zoom-in commit should have 'git-zoom-in:' trailer in message"
    );

    assert_eq!(
        commit.committer.name, "🔎",
        "Zoom-in commit should have committer name '🔎'"
    );
}

/// Verify that a commit has the structure of a zoom-out commit
///
/// Zoom-out commits should:
/// - Have "git-zoom-out:" in the message
/// - Have committer name "🔍"
pub fn verify_zoom_out_commit(commit: &Commit) {
    assert!(
        commit.message.contains("git-zoom-out:"),
        "Zoom-out commit should have 'git-zoom-out:' trailer in message"
    );

    assert_eq!(
        commit.committer.name, "🔍",
        "Zoom-out commit should have committer name '🔍'"
    );
}

/// Extract a trailer value from a commit message
///
/// Returns the value after "trailer-name: " if found, None otherwise.
pub fn extract_trailer(message: &str, trailer_name: &str) -> Option<String> {
    let prefix = format!("{}: ", trailer_name);
    message
        .lines()
        .find(|line| line.starts_with(&prefix))
        .map(|line| line.strip_prefix(&prefix).unwrap().trim().to_string())
}

/// Verify that the first-parent lineage matches expected commit messages
///
/// Walks the first-parent history from HEAD and checks that the commit
/// messages (first line only) match the expected sequence.
pub fn verify_first_parent_lineage(snapshot: &Repository, expected_messages: &[&str]) {
    // Get starting commit ID from HEAD
    let mut current_id = match snapshot.head() {
        HeadState::Symbolic(ref_name) => snapshot
            .resolve_ref(ref_name)
            .expect("HEAD reference not found"),
        HeadState::Detached(id) => *id,
    };

    let mut messages = Vec::new();

    loop {
        let commit = snapshot
            .get_commit(&current_id)
            .expect("Commit not found in repository");

        // Get first line of commit message
        if let Some(first_line) = commit.message.lines().next() {
            messages.push(first_line.to_string());
        }

        if commit.parents.is_empty() {
            break;
        }

        // Follow first parent
        current_id = commit.parents[0];
    }

    // Reverse to get root-to-HEAD order
    messages.reverse();

    assert_eq!(
        messages.len(),
        expected_messages.len(),
        "Expected {} commits in first-parent lineage, found {}: {:?}",
        expected_messages.len(),
        messages.len(),
        messages
    );

    for (i, (actual, expected)) in messages.iter().zip(expected_messages.iter()).enumerate() {
        assert_eq!(
            actual, expected,
            "Commit {} message mismatch. Expected '{}', got '{}'",
            i, expected, actual
        );
    }
}
