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
///
/// Matches commits by structure: message + parents + committer + tree
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

    // Create a signature for each commit based on structural properties
    fn commit_signature(commit: &git_snapshot::Commit) -> String {
        format!(
            "{}|{}|{}|{}|{}",
            commit.message.trim(),
            commit.parents.len(),
            commit.committer.name,
            commit.committer.email,
            commit.tree.paths().count()
        )
    }

    // Build maps by signature
    let mut actual_by_sig: std::collections::HashMap<String, Vec<&git_snapshot::Commit>> =
        std::collections::HashMap::new();
    for commit in &actual_commits {
        let sig = commit_signature(commit);
        actual_by_sig.entry(sig).or_insert_with(Vec::new).push(commit);
    }

    let mut expected_by_sig: std::collections::HashMap<String, Vec<&git_snapshot::Commit>> =
        std::collections::HashMap::new();
    for commit in &expected_commits {
        let sig = commit_signature(commit);
        expected_by_sig.entry(sig).or_insert_with(Vec::new).push(commit);
    }

    // Check that each expected signature exists in actual
    for (sig, expected_commits_with_sig) in &expected_by_sig {
        let actual_commits_with_sig = actual_by_sig.get(sig).unwrap_or_else(|| {
            let sample = expected_commits_with_sig[0];
            panic!(
                "Missing commit in actual snapshot:\n\
                 Message: {}\n\
                 Parents: {}\n\
                 Committer: {} <{}>\n\
                 Tree paths: {}",
                sample.message.lines().next().unwrap_or(""),
                sample.parents.len(),
                sample.committer.name,
                sample.committer.email,
                sample.tree.paths().count()
            );
        });

        assert_eq!(
            actual_commits_with_sig.len(),
            expected_commits_with_sig.len(),
            "Different number of commits with same signature: {}",
            sig
        );

        // Compare tree contents for each matching pair
        for (actual_commit, expected_commit) in
            actual_commits_with_sig.iter().zip(expected_commits_with_sig.iter())
        {
            let actual_paths: std::collections::HashSet<_> = actual_commit.tree.paths().collect();
            let expected_paths: std::collections::HashSet<_> =
                expected_commit.tree.paths().collect();

            assert_eq!(
                actual_paths, expected_paths,
                "Tree paths mismatch for commit '{}'",
                expected_commit.message.lines().next().unwrap_or("")
            );

            for path in &expected_paths {
                assert_eq!(
                    actual_commit.tree.get(path),
                    expected_commit.tree.get(path),
                    "Tree content mismatch for path '{}' in commit '{}'",
                    path,
                    expected_commit.message.lines().next().unwrap_or("")
                );
            }
        }
    }

    println!("✓ Snapshots match (validated {} commits)", expected_commits.len());
}
