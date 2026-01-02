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

    // Get actual resulting state
    let actual = repo.to_snapshot();

    // DEBUG: Print actual structure
    println!("\n=== Actual commits after zoom-in ===");
    for commit in actual.commits() {
        println!("Message: {}", commit.message.lines().next().unwrap_or(""));
        println!("  Parents: {}", commit.parents.len());
        println!("  Committer: {} <{}>", commit.committer.name, commit.committer.email);
        println!("  Tree paths: {}", commit.tree.paths().count());
        println!();
    }

    // Load expected state from fixture
    let expected_yaml = include_str!("fixtures/after_zoom_in_src_lib.yaml");
    let expected = git_snapshot::parse(expected_yaml).expect("failed to parse expected fixture");

    // Compare snapshots
    assert_snapshots_equal(&actual, &expected);
}

#[test]
fn test_complete_zoom_cycle() {
    // Load initial state
    let initial_yaml = include_str!("fixtures/complete_cycle_initial.yaml");
    let repo = TestRepo::from_yaml(initial_yaml);

    // Zoom in
    repo.run_zoom(&["in", "src"]).expect("zoom in failed");

    // Make modifications
    repo.write_file("lib.txt", "MODIFIED library code");
    repo.git_add_and_commit("Update lib");

    repo.write_file("new.txt", "new content");
    repo.git_add_and_commit("Add new file");

    // Zoom out
    repo.run_zoom(&["out"]).expect("zoom out failed");

    // Get actual state
    let actual = repo.to_snapshot();

    // Debug: print actual commits
    println!("\nActual commits ({}):", actual.commits().count());
    for commit in actual.commits() {
        println!(
            "  {} | parents:{} | {} | committer:{} <{}> | paths:{}",
            commit.id.to_hex().chars().take(7).collect::<String>(),
            commit.parents.len(),
            commit.message.lines().next().unwrap_or(""),
            commit.committer.name,
            commit.committer.email,
            commit.tree.paths().count()
        );
    }

    // Load expected state
    let expected_yaml = include_str!("fixtures/complete_cycle_final.yaml");
    let expected = git_snapshot::parse(expected_yaml).expect("failed to parse expected fixture");

    println!("\nExpected commits ({}):", expected.commits().count());
    for commit in expected.commits() {
        println!(
            "  {} | parents:{} | {} | committer:{} <{}> | paths:{}",
            commit.id.to_hex().chars().take(7).collect::<String>(),
            commit.parents.len(),
            commit.message.lines().next().unwrap_or(""),
            commit.committer.name,
            commit.committer.email,
            commit.tree.paths().count()
        );
    }

    // Compare
    assert_snapshots_equal(&actual, &expected);
}

#[test]
fn test_nested_directory_zoom() {
    let initial_yaml = include_str!("fixtures/nested_initial.yaml");
    let repo = TestRepo::from_yaml(initial_yaml);

    // Zoom into nested path
    repo.run_zoom(&["in", "src/core"]).expect("zoom in failed");

    // Verify working directory has only src/core contents at root
    assert!(repo.file_exists("engine/mod.rs"));
    assert!(repo.file_exists("types.rs"));
    assert!(!repo.file_exists("README.md"));
    assert!(!repo.file_exists("src"));

    let actual = repo.to_snapshot();
    let expected_yaml = include_str!("fixtures/nested_after_zoom.yaml");
    let expected = git_snapshot::parse(expected_yaml).expect("failed to parse expected");

    assert_snapshots_equal(&actual, &expected);
}

#[test]
fn test_empty_subtree_with_allow_empty() {
    let yaml = r#"
HEAD: refs/heads/main
refs:
  heads:
    main: 1
1:
  message: "Initial commit"
  tree:
    README.md: "readme"
"#;

    let repo = TestRepo::from_yaml(yaml);

    // Zoom into non-existent path with --allow-empty
    repo.run_zoom(&["in", "nonexistent", "--allow-empty"])
        .expect("zoom in with --allow-empty should succeed");

    // Should have empty working directory
    assert!(!repo.file_exists("README.md"));

    let snapshot = repo.to_snapshot();

    // Verify we have 3 commits: original, seed, zoom-in merge
    assert_eq!(snapshot.commits().count(), 3);

    // Verify HEAD commit is zoom-in
    let head = snapshot.head_commit().unwrap();
    assert!(head.message.contains("git-zoom-in:"));
    assert_eq!(head.committer.name, "🔎");

    // Verify tree is empty
    assert_eq!(head.tree.paths().count(), 0);
}

#[test]
fn test_zoom_without_allow_empty_fails() {
    let yaml = r#"
HEAD: refs/heads/main
refs:
  heads:
    main: 1
1:
  message: "Initial commit"
  tree:
    README.md: "readme"
"#;

    let repo = TestRepo::from_yaml(yaml);

    // Zoom into non-existent path WITHOUT --allow-empty should fail
    let result = repo.run_zoom(&["in", "nonexistent"]);
    assert!(result.is_err());
    let err_msg = result.unwrap_err();
    assert!(err_msg.contains("does not exist") || err_msg.contains("not found"));
}

#[test]
fn test_multiple_files_in_subtree() {
    let yaml = r#"
HEAD: refs/heads/main
refs:
  heads:
    main: 1
1:
  message: "Initial commit"
  tree:
    README.md: "root readme"
    lib/a.txt: "file a"
    lib/b.txt: "file b"
    lib/c.txt: "file c"
    lib/sub/d.txt: "file d"
"#;

    let repo = TestRepo::from_yaml(yaml);
    repo.run_zoom(&["in", "lib"]).expect("zoom in failed");

    // Verify all lib files are at root
    assert_eq!(repo.read_file("a.txt"), "file a");
    assert_eq!(repo.read_file("b.txt"), "file b");
    assert_eq!(repo.read_file("c.txt"), "file c");
    assert_eq!(repo.read_file("sub/d.txt"), "file d");
    assert!(!repo.file_exists("README.md"));

    // Modify multiple files
    repo.write_file("a.txt", "MODIFIED a");
    repo.write_file("c.txt", "MODIFIED c");
    repo.write_file("sub/d.txt", "MODIFIED d");
    repo.git_add_and_commit("Modify multiple files");

    // Zoom out
    repo.run_zoom(&["out"]).expect("zoom out failed");

    // Verify all modifications preserved
    assert_eq!(repo.read_file("lib/a.txt"), "MODIFIED a");
    assert_eq!(repo.read_file("lib/b.txt"), "file b"); // unchanged
    assert_eq!(repo.read_file("lib/c.txt"), "MODIFIED c");
    assert_eq!(repo.read_file("lib/sub/d.txt"), "MODIFIED d");
    assert_eq!(repo.read_file("README.md"), "root readme");
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
