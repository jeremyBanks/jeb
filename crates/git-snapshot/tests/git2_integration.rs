//! Integration tests for git2 repository reading and writing

use git_snapshot::{GitError, HeadState, RefName, Repository, UnsupportedFeature};
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
    README.md: "# Test"
    src/main.rs: "fn main() {}"
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
    for commit in original.commits() {
        let rt_commit = roundtrip.get_commit(&commit.id).unwrap();
        assert_eq!(commit.message, rt_commit.message);
        assert_eq!(commit.parents, rt_commit.parents);
        assert_eq!(commit.tree.entries, rt_commit.tree.entries);
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
    repo.set_head("refs/heads/main").unwrap();
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
    a/b/c/file.txt: "deep"
    a/b/other.txt: "mid"
    a/top.txt: "shallow"
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
