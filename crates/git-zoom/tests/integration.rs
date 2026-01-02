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
    let head_commit = snapshot.head_commit().unwrap();

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
    let head_commit = snapshot.head_commit().unwrap();
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
    let head = snapshot.head_commit().unwrap();
    verify_zoom_out_commit(head);

    // Verify the path is correct in trailer
    let path = extract_trailer(&head.message, "git-zoom-out");
    assert_eq!(path, Some("src/lib".to_string()));
}

// ============================================================================
// Category B: Path Variations
// ============================================================================

#[test]
fn test_zoom_nested_path() {
    let yaml = r#"
HEAD: refs/heads/main
refs:
  heads:
    main: 1
1:
  message: "Initial commit"
  tree:
    README.md: "root"
    src/lib/core/engine.rs: "engine code"
    src/lib/core/utils.rs: "utils code"
    src/lib/api.rs: "api code"
"#;

    let repo = TestRepo::from_yaml(yaml);

    // Zoom into deeply nested path
    repo.run_zoom(&["in", "src/lib/core"])
        .expect("zoom into nested path failed");

    // Verify files at root level
    assert!(repo.file_exists("engine.rs"));
    assert!(repo.file_exists("utils.rs"));
    assert_eq!(repo.read_file("engine.rs"), "engine code");

    // Verify other files don't exist
    assert!(!repo.file_exists("api.rs"));
    assert!(!repo.file_exists("README.md"));
    assert!(!repo.file_exists("src"));
}

#[test]
fn test_zoom_sibling_paths() {
    let yaml = r#"
HEAD: refs/heads/main
refs:
  heads:
    main: 1
1:
  message: "Initial commit"
  tree:
    src/lib/lib.txt: "library"
    src/bin/main.txt: "binary"
    docs/guide.md: "docs"
"#;

    let repo = TestRepo::from_yaml(yaml);

    // Zoom into first sibling
    repo.run_zoom(&["in", "src/lib"]).unwrap();
    repo.write_file("lib.txt", "library v2");
    repo.git_add_and_commit("Update lib");
    repo.run_zoom(&["out"]).unwrap();

    // Zoom into second sibling
    repo.run_zoom(&["in", "src/bin"]).unwrap();
    repo.write_file("main.txt", "binary v2");
    repo.git_add_and_commit("Update bin");
    repo.run_zoom(&["out"]).unwrap();

    // Verify both modifications preserved
    assert_eq!(repo.read_file("src/lib/lib.txt"), "library v2");
    assert_eq!(repo.read_file("src/bin/main.txt"), "binary v2");
    assert_eq!(repo.read_file("docs/guide.md"), "docs");
}

#[test]
fn test_zoom_path_normalization() {
    let yaml = r#"
HEAD: refs/heads/main
refs:
  heads:
    main: 1
1:
  tree:
    src/code.txt: "content"
"#;

    let repo = TestRepo::from_yaml(yaml);

    // Test with trailing slash
    repo.run_zoom(&["in", "src/"]).expect("trailing slash should work");

    assert!(repo.file_exists("code.txt"));
    assert_eq!(repo.read_file("code.txt"), "content");
}

// ============================================================================
// Category C: Multiple Cycles
// ============================================================================

#[test]
fn test_multiple_zoom_cycles() {
    let yaml = r#"
HEAD: refs/heads/main
refs:
  heads:
    main: 1
1:
  tree:
    README.md: "root"
    src/file.txt: "original"
"#;

    let repo = TestRepo::from_yaml(yaml);

    // First cycle
    repo.run_zoom(&["in", "src"]).unwrap();
    repo.write_file("file.txt", "v1");
    repo.git_add_and_commit("Update 1");
    repo.run_zoom(&["out"]).unwrap();

    // Second cycle
    repo.run_zoom(&["in", "src"]).unwrap();
    repo.write_file("file.txt", "v2");
    repo.git_add_and_commit("Update 2");
    repo.run_zoom(&["out"]).unwrap();

    // Third cycle
    repo.run_zoom(&["in", "src"]).unwrap();
    assert_eq!(repo.read_file("file.txt"), "v2");
    repo.write_file("file.txt", "v3");
    repo.git_add_and_commit("Update 3");
    repo.run_zoom(&["out"]).unwrap();

    // Verify final state
    assert_eq!(repo.read_file("src/file.txt"), "v3");
    assert_eq!(repo.read_file("README.md"), "root");
}

#[test]
fn test_nested_zoom() {
    let yaml = r#"
HEAD: refs/heads/main
refs:
  heads:
    main: 1
1:
  tree:
    root.txt: "root"
    src/lib/core/mod.rs: "core module"
    src/lib/lib.rs: "lib root"
"#;

    let repo = TestRepo::from_yaml(yaml);

    // First zoom: src/lib
    repo.run_zoom(&["in", "src/lib"]).unwrap();
    assert!(repo.file_exists("core/mod.rs"));
    assert!(repo.file_exists("lib.rs"));
    assert!(!repo.file_exists("root.txt"));

    // Second zoom: core (nested within first zoom)
    repo.run_zoom(&["in", "core"]).unwrap();
    assert!(repo.file_exists("mod.rs"));
    assert!(!repo.file_exists("lib.rs"));
    assert_eq!(repo.read_file("mod.rs"), "core module");

    // Modify
    repo.write_file("mod.rs", "modified core");
    repo.git_add_and_commit("Update core");

    // First zoom out (back to src/lib view)
    repo.run_zoom(&["out"]).unwrap();
    assert_eq!(repo.read_file("core/mod.rs"), "modified core");
    assert!(repo.file_exists("lib.rs"));

    // Second zoom out (back to full tree)
    repo.run_zoom(&["out"]).unwrap();
    assert_eq!(repo.read_file("src/lib/core/mod.rs"), "modified core");
    assert_eq!(repo.read_file("root.txt"), "root");
}

#[test]
fn test_return_to_previous_zoom() {
    let yaml = r#"
HEAD: refs/heads/main
refs:
  heads:
    main: 1
1:
  tree:
    root.txt: "root"
    src/file.txt: "original"
"#;

    let repo = TestRepo::from_yaml(yaml);

    // Zoom in
    repo.run_zoom(&["in", "src"]).unwrap();
    repo.write_file("file.txt", "modified");
    repo.git_add_and_commit("Update");

    // Zoom out
    repo.run_zoom(&["out"]).unwrap();
    assert_eq!(repo.read_file("src/file.txt"), "modified");

    // Zoom back in without specifying path (should find previous zoom)
    repo.run_zoom(&["in"]).expect("zoom in without path should work");
    assert_eq!(repo.read_file("file.txt"), "modified");
    assert!(!repo.file_exists("root.txt"));
}
