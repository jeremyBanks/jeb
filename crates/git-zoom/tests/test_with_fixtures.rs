// ! Additional edge-case tests for git-zoom behavior

mod common;

use common::helpers::*;

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
    lib:
      a.txt: "file a"
      b.txt: "file b"
      c.txt: "file c"
      sub:
        d.txt: "file d"
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
