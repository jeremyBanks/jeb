//! Integration tests for git2 repository reading and writing

use git_snapshot::{GitError, HeadState, Repository, UnsupportedFeature};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn test_empty_repository() {
    // Create empty git repo
    let temp = TempDir::new().unwrap();
    git2::Repository::init(temp.path()).unwrap();

    // Read snapshot
    let snapshot = Repository::from_git_dir(temp.path()).unwrap();

    // Verify structure
    assert_eq!(snapshot.commits().count(), 0);
    assert_eq!(snapshot.refs().count(), 0);
    assert!(matches!(snapshot.head(), HeadState::Symbolic(_)));
}

#[test]
fn test_single_commit() {
    // Create repo with one commit
    let temp = TempDir::new().unwrap();
    let repo = git2::Repository::init(temp.path()).unwrap();

    // Create file and commit
    fs::write(temp.path().join("README.md"), "# Hello").unwrap();
    let mut index = repo.index().unwrap();
    index.add_path(Path::new("README.md")).unwrap();
    index.write().unwrap();

    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let sig = git2::Signature::now("Test", "test@example.com").unwrap();

    repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
        .unwrap();

    // Read snapshot
    let snapshot = Repository::from_git_dir(temp.path()).unwrap();

    // Verify
    assert_eq!(snapshot.commits().count(), 1);
    let commit = snapshot.commits().next().unwrap();
    assert_eq!(commit.message, "Initial commit");
    assert_eq!(commit.tree.get("README.md"), Some("# Hello"));
}

#[test]
fn test_roundtrip_through_temporary() {
    // Parse YAML snapshot
    let yaml = r#"
HEAD: refs/heads/main
refs:
  heads:
    main: 1
1:
  parents: []
  message: test commit
  tree:
    README.md: "test content"
    src:
      main.rs: "code content"
"#;
    let original = git_snapshot::parse(yaml).unwrap();

    // Materialize to temp repo
    let temp_repo = original.to_temporary_repository().unwrap();

    // Read back
    let roundtrip = temp_repo.to_snapshot().unwrap();

    // Compare
    assert_eq!(original.commits().count(), roundtrip.commits().count());
    assert_eq!(original.refs().count(), roundtrip.refs().count());
    assert_eq!(original.head(), roundtrip.head());

    // Deep comparison of commit contents
    // Note: commit IDs will differ (pseudo IDs like "1" become real git SHA-1s)
    // so we compare by finding commits with matching messages
    for commit in original.commits() {
        let rt_commit = roundtrip
            .commits()
            .find(|c| c.message == commit.message)
            .expect("Should find commit with matching message");

        // Compare number of parents (can't compare parent IDs directly due to ID changes)
        assert_eq!(commit.parents.len(), rt_commit.parents.len());

        // Compare tree paths
        for path in commit.tree.paths() {
            assert_eq!(
                rt_commit.tree.get(path),
                commit.tree.get(path),
                "Path {} content differs",
                path
            );
        }
    }
}

#[test]
#[cfg(unix)]
fn test_reject_executable_bit() {
    use std::os::unix::fs::PermissionsExt;

    let temp = TempDir::new().unwrap();
    let repo = git2::Repository::init(temp.path()).unwrap();

    // Create executable file
    let script_path = temp.path().join("script.sh");
    fs::write(&script_path, "#!/bin/sh\n").unwrap();
    fs::set_permissions(&script_path, fs::Permissions::from_mode(0o755)).unwrap();

    let mut index = repo.index().unwrap();
    index.add_path(Path::new("script.sh")).unwrap();
    index.write().unwrap();

    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let sig = git2::Signature::now("Test", "test@example.com").unwrap();

    repo.commit(Some("HEAD"), &sig, &sig, "Add script", &tree, &[])
        .unwrap();

    // Should fail with UnsupportedFeature error
    let result = Repository::from_git_dir(temp.path());
    assert!(result.is_err());
    if let Err(GitError::UnsupportedFeature { feature, path }) = result {
        assert_eq!(feature, UnsupportedFeature::ExecutableBit);
        assert_eq!(path, "script.sh");
    } else {
        panic!("Expected UnsupportedFeature error");
    }
}

#[test]
fn test_multiple_commits_with_merge() {
    let temp = TempDir::new().unwrap();
    let repo = git2::Repository::init(temp.path()).unwrap();
    let sig = git2::Signature::now("Test", "test@example.com").unwrap();

    // Create initial commit
    fs::write(temp.path().join("file.txt"), "initial").unwrap();
    let mut index = repo.index().unwrap();
    index.add_path(Path::new("file.txt")).unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let commit1 = repo
        .commit(Some("HEAD"), &sig, &sig, "Initial", &tree, &[])
        .unwrap();
    let commit1_obj = repo.find_commit(commit1).unwrap();

    // Create second commit
    fs::write(temp.path().join("file.txt"), "second").unwrap();
    index.add_path(Path::new("file.txt")).unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let commit2 = repo
        .commit(Some("HEAD"), &sig, &sig, "Second", &tree, &[&commit1_obj])
        .unwrap();
    let commit2_obj = repo.find_commit(commit2).unwrap();

    // Create branch
    repo.branch("feature", &commit1_obj, false).unwrap();
    repo.set_head("refs/heads/feature").unwrap();

    // Create commit on branch
    fs::write(temp.path().join("file.txt"), "branch").unwrap();
    index.add_path(Path::new("file.txt")).unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let commit3 = repo
        .commit(
            Some("refs/heads/feature"),
            &sig,
            &sig,
            "Branch",
            &tree,
            &[&commit1_obj],
        )
        .unwrap();
    let commit3_obj = repo.find_commit(commit3).unwrap();

    // Merge (create merge commit)
    repo.set_head("refs/heads/master").unwrap();
    fs::write(temp.path().join("file.txt"), "merged").unwrap();
    index.add_path(Path::new("file.txt")).unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        "Merge",
        &tree,
        &[&commit2_obj, &commit3_obj],
    )
    .unwrap();

    // Read snapshot
    let snapshot = Repository::from_git_dir(temp.path()).unwrap();

    // Verify
    assert_eq!(snapshot.commits().count(), 4);
    assert_eq!(snapshot.refs().count(), 2);

    // Find merge commit (has 2 parents)
    let merge_commit = snapshot
        .commits()
        .find(|c| c.parents.len() == 2)
        .expect("Should have merge commit");
    assert_eq!(merge_commit.message, "Merge");
}

#[test]
fn test_nested_directories() {
    let yaml = r#"
HEAD: refs/heads/main
refs:
  heads:
    main: 1
1:
  parents: []
  tree:
    a:
      b:
        c:
          file.txt: "deep"
        other.txt: "mid"
      top.txt: "shallow"
    root.txt: "root"
"#;
    let original = git_snapshot::parse(yaml).unwrap();

    // Materialize and read back
    let temp_repo = original.to_temporary_repository().unwrap();
    let roundtrip = temp_repo.to_snapshot().unwrap();

    // Verify all files preserved
    let commit = roundtrip.commits().next().unwrap();
    assert_eq!(commit.tree.get("a/b/c/file.txt"), Some("deep"));
    assert_eq!(commit.tree.get("a/b/other.txt"), Some("mid"));
    assert_eq!(commit.tree.get("a/top.txt"), Some("shallow"));
    assert_eq!(commit.tree.get("root.txt"), Some("root"));
}

#[test]
fn test_commit_hash_stability_plain_message() {
    // CRITICAL BUG TEST: Commit hash should remain stable when round-tripped through git
    // This tests for the bug where plain scalar messages produce different hashes
    // than block scalar messages due to inconsistent trailing newline handling.

    let yaml = r#"
HEAD: refs/heads/main
refs:
  heads:
    main: 1
1:
  message: Test commit
  tree:
    file.txt: content
"#;

    // Parse original
    let original = git_snapshot::parse(yaml).unwrap();
    let original_commit = original.commits().next().unwrap();
    let hash_before = original_commit.id.clone();

    // Write to git and read back
    let temp_repo = original.to_temporary_repository().unwrap();
    let roundtrip = Repository::from_git_dir(temp_repo.path()).unwrap();
    let roundtrip_commit = roundtrip.commits().next().unwrap();
    let hash_after = roundtrip_commit.id.clone();

    // Hash should be IDENTICAL - this tests that message normalization is consistent
    assert_eq!(
        hash_before, hash_after,
        "Commit hash changed after git round-trip! Message handling is inconsistent.\n\
         Before: {:?}\n\
         After:  {:?}\n\
         This indicates message newline normalization is not deterministic.",
        hash_before, hash_after
    );
}

#[test]
fn test_commit_hash_stability_block_message() {
    // Test hash stability with block scalar messages (which have trailing newlines)
    let yaml = r#"
HEAD: refs/heads/main
refs:
  heads:
    main: 1
1:
  message: |
    Test commit
  tree:
    file.txt: content
"#;

    let original = git_snapshot::parse(yaml).unwrap();
    let original_commit = original.commits().next().unwrap();
    let hash_before = original_commit.id.clone();

    let temp_repo = original.to_temporary_repository().unwrap();
    let roundtrip = Repository::from_git_dir(temp_repo.path()).unwrap();
    let roundtrip_commit = roundtrip.commits().next().unwrap();
    let hash_after = roundtrip_commit.id.clone();

    assert_eq!(
        hash_before, hash_after,
        "Commit hash changed with block message! Before: {:?} After: {:?}",
        hash_before, hash_after
    );
}

#[test]
fn test_commit_hash_stability_multiline_message() {
    // Test hash stability with multi-line commit messages
    let yaml = r#"
HEAD: refs/heads/main
refs:
  heads:
    main: 1
1:
  message: |
    Subject line

    Body paragraph with details
  tree:
    file.txt: content
"#;

    let original = git_snapshot::parse(yaml).unwrap();
    let original_commit = original.commits().next().unwrap();
    let hash_before = original_commit.id.clone();

    let temp_repo = original.to_temporary_repository().unwrap();
    let roundtrip = Repository::from_git_dir(temp_repo.path()).unwrap();
    let roundtrip_commit = roundtrip.commits().next().unwrap();
    let hash_after = roundtrip_commit.id.clone();

    assert_eq!(
        hash_before, hash_after,
        "Commit hash changed with multiline message! Before: {:?} After: {:?}",
        hash_before, hash_after
    );
}
