// ! Tests that use fixture files to verify git-zoom behavior

mod helpers;

use helpers::*;

#[test]
fn test_zoom_in_with_fixture() {
    // Load initial state from fixture
    let initial_yaml = include_str!("fixtures/basic_initial.yaml");
    let repo = TestRepo::from_yaml(initial_yaml);

    // Perform zoom-in operation
    repo.run_zoom(&["in", "src/lib"])
        .expect("zoom in should succeed");

    // Load expected state from fixture
    let expected_yaml = include_str!("fixtures/after_zoom_in_src_lib.yaml");
    let expected = git_snapshot::parse(expected_yaml).expect("failed to parse expected fixture");

    // Get actual resulting state
    let actual = repo.to_snapshot();

    // Compare snapshots
    assert_snapshots_equal(&actual, &expected);
}

/// Compare two snapshots, ignoring commit IDs (since they're non-deterministic)
/// but verifying structure, messages, trees, and parent relationships
fn assert_snapshots_equal(actual: &git_snapshot::Repository, expected: &git_snapshot::Repository) {
    // Compare HEAD state
    assert_eq!(actual.head(), expected.head(), "HEAD reference mismatch");

    // Compare number of commits
    let actual_commits: Vec<_> = actual.commits().collect();
    let expected_commits: Vec<_> = expected.commits().collect();

    assert_eq!(
        actual_commits.len(),
        expected_commits.len(),
        "Commit count mismatch: expected {}, got {}",
        expected_commits.len(),
        actual_commits.len()
    );

    // Build maps by message (assumes messages are unique enough to identify commits)
    let mut actual_by_msg: std::collections::HashMap<_, _> = std::collections::HashMap::new();
    for commit in &actual_commits {
        let msg_first_line = commit.message.lines().next().unwrap_or("");
        actual_by_msg.insert(msg_first_line, commit);
    }

    let mut expected_by_msg: std::collections::HashMap<_, _> = std::collections::HashMap::new();
    for commit in &expected_commits {
        let msg_first_line = commit.message.lines().next().unwrap_or("");
        expected_by_msg.insert(msg_first_line, commit);
    }

    // For each expected commit, find matching actual commit and compare details
    for expected_commit in &expected_commits {
        let expected_msg_first = expected_commit.message.lines().next().unwrap_or("");

        let actual_commit = actual_by_msg
            .get(expected_msg_first)
            .expect(&format!(
                "Actual snapshot is missing commit with message: {}",
                expected_msg_first
            ));

        // Compare commit details
        assert_eq!(
            actual_commit.message.trim(),
            expected_commit.message.trim(),
            "Message mismatch for commit: {}",
            expected_msg_first
        );

        assert_eq!(
            actual_commit.parents.len(),
            expected_commit.parents.len(),
            "Parent count mismatch for commit '{}': expected {}, got {}",
            expected_msg_first,
            expected_commit.parents.len(),
            actual_commit.parents.len()
        );

        assert_eq!(
            actual_commit.committer.name,
            expected_commit.committer.name,
            "Committer name mismatch for commit: {}",
            expected_msg_first
        );

        assert_eq!(
            actual_commit.committer.email,
            expected_commit.committer.email,
            "Committer email mismatch for commit: {}",
            expected_msg_first
        );

        // Compare tree contents
        let actual_paths: std::collections::HashSet<_> = actual_commit.tree.paths().collect();
        let expected_paths: std::collections::HashSet<_> = expected_commit.tree.paths().collect();

        assert_eq!(
            actual_paths, expected_paths,
            "Tree paths mismatch for commit '{}': expected {:?}, got {:?}",
            expected_msg_first, expected_paths, actual_paths
        );

        for path in &expected_paths {
            assert_eq!(
                actual_commit.tree.get(path),
                expected_commit.tree.get(path),
                "Tree content mismatch for path '{}' in commit '{}'",
                path,
                expected_msg_first
            );
        }
    }

    println!("✓ Snapshots match (validated {} commits)", expected_commits.len());
}
