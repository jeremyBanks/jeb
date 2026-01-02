//! Test helper utilities for git-zoom integration tests
//!
//! This module provides a `TestRepo` wrapper around git-snapshot's `TemporaryRepository`
//! with git-zoom-specific utilities for running zoom commands, manipulating files, and
//! verifying results.

use git_snapshot::{parse, serialize, Commit, CommitIdStyle, HeadState, Repository, SerializationOptions};
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;

/// Global mutex to synchronize directory changes across tests
/// This prevents parallel tests from interfering with each other when changing
/// the process's current directory.
static DIR_MUTEX: Mutex<()> = Mutex::new(());

/// Wrapper around git-snapshot's TemporaryRepository with git-zoom-specific utilities
pub struct TestRepo {
    pub temp_repo: git_snapshot::TemporaryRepository,
    /// If set, TestRepo will compare final state against this fixture on drop
    expected_fixture_path: Option<PathBuf>,
}

impl TestRepo {
    /// Create a test repository from YAML string
    pub fn from_yaml(yaml: &str) -> Self {
        let snapshot = parse(yaml).expect("Failed to parse YAML");
        Self::from_snapshot(snapshot)
    }

    /// Create a test repository from a git-snapshot Repository
    pub fn from_snapshot(snapshot: Repository) -> Self {
        let temp_repo = snapshot
            .to_temporary_repository()
            .expect("Failed to create temporary repository");

        let test_repo = Self {
            temp_repo,
            expected_fixture_path: None,
        };

        // Reset working tree to match HEAD
        // git-snapshot creates commits but doesn't populate the working directory
        let _guard = DIR_MUTEX.lock().unwrap();
        let original_dir = env::current_dir().unwrap();
        env::set_current_dir(test_repo.workdir()).unwrap();

        let _reset = Command::new("git")
            .args(&["reset", "--hard", "HEAD"])
            .output()
            .expect("Failed to reset working tree");

        env::set_current_dir(original_dir).unwrap();
        drop(_guard);

        test_repo
    }

    /// Set the expected fixture path for final state comparison
    pub fn set_expected_fixture(&mut self, path: PathBuf) {
        self.expected_fixture_path = Some(path);
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
        let _guard = DIR_MUTEX.lock().unwrap();
        let original_dir = env::current_dir().unwrap();
        env::set_current_dir(self.workdir()).unwrap();

        let result = Command::new(env!("CARGO_BIN_EXE_git-zoom"))
            .args(args)
            .output();

        env::set_current_dir(original_dir).unwrap();
        drop(_guard);

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
        let _guard = DIR_MUTEX.lock().unwrap();
        let original_dir = env::current_dir().unwrap();
        env::set_current_dir(self.workdir()).unwrap();

        // Git add all
        let add_result = Command::new("git").args(&["add", "."]).output();

        // Get current HEAD timestamp to derive from
        let timestamp_result = Command::new("git")
            .args(&["show", "-s", "--format=%aI", "HEAD"])
            .output();

        let timestamp = if let Ok(output) = timestamp_result {
            if output.status.success() {
                String::from_utf8_lossy(&output.stdout).trim().to_string()
            } else {
                // No HEAD commit yet, use default
                "2024-12-06T06:12:24-06:24".to_string()
            }
        } else {
            "2024-12-06T06:12:24-06:24".to_string()
        };

        // Git commit (with explicit author/committer matching git-snapshot defaults)
        // Use deterministic timestamp derived from parent
        let commit_result = Command::new("git")
            .args(&[
                "-c",
                "user.name=User",
                "-c",
                "user.email=user@localhost",
                "commit",
                "-m",
                message,
            ])
            .env("GIT_AUTHOR_DATE", &timestamp)
            .env("GIT_COMMITTER_DATE", &timestamp)
            .output();

        env::set_current_dir(original_dir).unwrap();
        drop(_guard);

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
        HeadState::Symbolic(ref_name) => *snapshot
            .get_ref(ref_name)
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

/// Drop implementation for TestRepo to handle fixture comparison
impl Drop for TestRepo {
    fn drop(&mut self) {
        // Only compare if an expected fixture path was set
        if let Some(ref expected_path) = self.expected_fixture_path {
            // Capture final snapshot
            let snapshot = self.to_snapshot();

            // Serialize with default normalization style
            let actual_yaml = serialize(
                &snapshot,
                CommitIdStyle::Hex,
                SerializationOptions::default(),
            );

            // Use the comparison function from fixtures module
            super::fixtures::compare_or_update_fixture(expected_path, &actual_yaml);
        }
    }
}
