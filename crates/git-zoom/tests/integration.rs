//! Integration tests for git-zoom using git-snapshot
//!
//! These tests verify complete zoom in/out cycles using declarative YAML repository
//! snapshots for test setup and verification.

mod helpers;

use helpers::*;

// ============================================================================
// Category A: Basic Operations
// ============================================================================

#[test]
fn test_basic_zoom_in() {
    let yaml = r#"
HEAD: refs/heads/main
refs:
  heads:
    main: 1
1:
  message: "Initial commit"
  tree:
    README.md: "root readme"
    src/lib/foo.txt: "library code"
    src/lib/bar.txt: "more code"
"#;

    let repo = TestRepo::from_yaml(yaml);

    // Zoom into src/lib
    repo.run_zoom(&["in", "src/lib"]).expect("zoom in failed");

    // Verify working directory shows subtree at root
    assert!(repo.file_exists("foo.txt"), "foo.txt should exist at root");
    assert!(repo.file_exists("bar.txt"), "bar.txt should exist at root");
    assert_eq!(repo.read_file("foo.txt"), "library code");
    assert_eq!(repo.read_file("bar.txt"), "more code");

    // Verify files from outside subtree are gone
    assert!(
        !repo.file_exists("README.md"),
        "README.md should not exist after zoom"
    );
    assert!(
        !repo.file_exists("src"),
        "src directory should not exist after zoom"
    );

    // Verify commit structure
    let snapshot = repo.to_snapshot();
    let head_commit = snapshot.get_head_commit().unwrap();

    // Should be a merge commit with zoom-in trailer
    verify_zoom_in_commit(head_commit);

    // Check trailer has correct path
    let path = extract_trailer(&head_commit.message, "git-zoom-in");
    assert_eq!(path, Some("src/lib".to_string()));
}

#[test]
fn test_basic_zoom_out() {
    let yaml = r#"
HEAD: refs/heads/main
refs:
  heads:
    main: 1
1:
  message: "Initial commit"
  tree:
    README.md: "root readme"
    src/lib/foo.txt: "original"
"#;

    let repo = TestRepo::from_yaml(yaml);

    // Zoom in
    repo.run_zoom(&["in", "src/lib"]).unwrap();
    assert_eq!(repo.read_file("foo.txt"), "original");

    // Make a modification
    repo.write_file("foo.txt", "modified");
    repo.git_add_and_commit("Update foo.txt");

    // Zoom out
    repo.run_zoom(&["out"]).expect("zoom out failed");

    // Verify modification is preserved in correct location
    assert_eq!(repo.read_file("src/lib/foo.txt"), "modified");

    // Verify root file is still there
    assert_eq!(repo.read_file("README.md"), "root readme");

    // Verify commit structure
    let snapshot = repo.to_snapshot();
    let head_commit = snapshot.get_head_commit().unwrap();
    verify_zoom_out_commit(head_commit);
}

#[test]
fn test_complete_zoom_cycle() {
    let yaml = r#"
HEAD: refs/heads/main
refs:
  heads:
    main: 1
1:
  message: "Initial commit"
  tree:
    README.md: "root readme"
    other.txt: "other file"
    src/lib/foo.txt: "original"
    src/lib/bar.txt: "bar"
"#;

    let repo = TestRepo::from_yaml(yaml);

    // ZOOM IN
    repo.run_zoom(&["in", "src/lib"]).unwrap();

    // Verify working directory after zoom in
    assert_eq!(repo.read_file("foo.txt"), "original");
    assert_eq!(repo.read_file("bar.txt"), "bar");
    assert!(!repo.file_exists("README.md"));
    assert!(!repo.file_exists("other.txt"));

    // MODIFY - change existing file and add new file
    repo.write_file("foo.txt", "modified");
    repo.write_file("new.txt", "new file");
    repo.git_add_and_commit("Update library");

    // ZOOM OUT
    repo.run_zoom(&["out"]).unwrap();

    // Verify all changes are preserved
    assert_eq!(repo.read_file("src/lib/foo.txt"), "modified");
    assert_eq!(repo.read_file("src/lib/bar.txt"), "bar");
    assert_eq!(repo.read_file("src/lib/new.txt"), "new file");

    // Verify root files are preserved
    assert_eq!(repo.read_file("README.md"), "root readme");
    assert_eq!(repo.read_file("other.txt"), "other file");

    // Verify commit structure
    let snapshot = repo.to_snapshot();
    let head = snapshot.get_head_commit().unwrap();
    verify_zoom_out_commit(head);

    // Verify the path is correct in trailer
    let path = extract_trailer(&head.message, "git-zoom-out");
    assert_eq!(path, Some("src/lib".to_string()));
}
