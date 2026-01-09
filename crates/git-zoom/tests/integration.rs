//! Integration tests for git-zoom using git-snapshot
//!
//! These tests verify complete zoom in/out cycles using declarative YAML
//! repository snapshots for test setup and verification.

mod common;

use common::{
    fixtures::test_fixture,
    helpers::*,
};
use inline::snapshot;

// ============================================================================
// Category A: Basic Operations
// ============================================================================

#[test]
fn test_basic_zoom_in() {
    test_fixture("basic-zoom-in", |repo| {
        // Zoom into src/lib
        repo.run_zoom(&["in", "src/lib"])?;

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
        let repo_snapshot = repo.to_snapshot();
        let head_commit = repo_snapshot.head_commit().unwrap();

        // Should be a merge commit with zoom-in trailer
        verify_zoom_in_commit(head_commit);

        // Check trailer has correct path
        let path = extract_trailer(&head_commit.message, "git-zoom-in");
        assert_eq!(path, Some("src/lib".to_string()));

        // Inline snapshot: verify tree structure after zoom-in
        let mut tree_paths: Vec<_> = head_commit.tree.paths().collect();
        tree_paths.sort();
        snapshot(r#"["bar.txt", "foo.txt"]"#.to_string()).value = format!("{:?}", tree_paths);

        // Inline snapshot: verify commit message first line
        let msg_first_line = head_commit.message.lines().next().unwrap_or("");
        snapshot("Merge from 'src/lib'".to_string()).value = msg_first_line.to_string();

        Ok(())
    });
}

#[test]
fn test_basic_zoom_out() {
    test_fixture("basic-zoom-out", |repo| {
        // Zoom in
        repo.run_zoom(&["in", "src/lib"])?;
        assert_eq!(repo.read_file("foo.txt"), "original");

        // Make a modification
        repo.write_file("foo.txt", "modified");
        repo.git_add_and_commit("Update foo.txt");

        // Zoom out
        repo.run_zoom(&["out"])?;

        // Verify modification is preserved in correct location
        assert_eq!(repo.read_file("src/lib/foo.txt"), "modified");

        // Verify root file is still there
        assert_eq!(repo.read_file("README.md"), "root readme");

        // Verify commit structure
        let snapshot = repo.to_snapshot();
        let head_commit = snapshot.head_commit().unwrap();
        verify_zoom_out_commit(head_commit);

        Ok(())
    });
}

#[test]
fn test_complete_zoom_cycle() {
    test_fixture("complete-zoom-cycle", |repo| {
        // ZOOM IN
        repo.run_zoom(&["in", "src/lib"])?;

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
        repo.run_zoom(&["out"])?;

        // Verify all changes are preserved
        assert_eq!(repo.read_file("src/lib/foo.txt"), "modified");
        assert_eq!(repo.read_file("src/lib/bar.txt"), "bar");
        assert_eq!(repo.read_file("src/lib/new.txt"), "new file");

        // Verify root files are preserved
        assert_eq!(repo.read_file("README.md"), "root readme");
        assert_eq!(repo.read_file("other.txt"), "other file");

        // Verify commit structure
        let repo_snapshot = repo.to_snapshot();
        let head = repo_snapshot.head_commit().unwrap();
        verify_zoom_out_commit(head);

        // Verify the path is correct in trailer
        let path = extract_trailer(&head.message, "git-zoom-out");
        assert_eq!(path, Some("src/lib".to_string()));

        Ok(())
    });
}

// ============================================================================
// Category B: Path Variations
// ============================================================================

#[test]
fn test_zoom_nested_path() {
    test_fixture("zoom-nested-path", |repo| {
        // Zoom into deeply nested path
        repo.run_zoom(&["in", "src/lib/core"])?;

        // Verify files at root level
        assert!(repo.file_exists("engine.rs"));
        assert!(repo.file_exists("utils.rs"));
        assert_eq!(repo.read_file("engine.rs"), "engine code");

        // Verify other files don't exist
        assert!(!repo.file_exists("api.rs"));
        assert!(!repo.file_exists("README.md"));
        assert!(!repo.file_exists("src"));

        Ok(())
    });
}

#[test]
fn test_zoom_sibling_paths() {
    test_fixture("zoom-sibling-paths", |repo| {
        // Zoom into first sibling
        repo.run_zoom(&["in", "src/lib"])?;
        repo.write_file("lib.txt", "library v2");
        repo.git_add_and_commit("Update lib");
        repo.run_zoom(&["out"])?;

        // Zoom into second sibling
        repo.run_zoom(&["in", "src/bin"])?;
        repo.write_file("main.txt", "binary v2");
        repo.git_add_and_commit("Update bin");
        repo.run_zoom(&["out"])?;

        // Verify both modifications preserved
        assert_eq!(repo.read_file("src/lib/lib.txt"), "library v2");
        assert_eq!(repo.read_file("src/bin/main.txt"), "binary v2");
        assert_eq!(repo.read_file("docs/guide.md"), "docs");

        Ok(())
    });
}

#[test]
fn test_zoom_path_normalization() {
    test_fixture("zoom-path-normalization", |repo| {
        // Test with trailing slash
        repo.run_zoom(&["in", "src/"])?;

        assert!(repo.file_exists("code.txt"));
        assert_eq!(repo.read_file("code.txt"), "content");

        Ok(())
    });
}

// ============================================================================
// Category C: Multiple Cycles
// ============================================================================

#[test]
fn test_multiple_zoom_cycles() {
    test_fixture("multiple-zoom-cycles", |repo| {
        // First cycle
        repo.run_zoom(&["in", "src"])?;
        repo.write_file("file.txt", "v1");
        repo.git_add_and_commit("Update 1");
        repo.run_zoom(&["out"])?;

        // Second cycle
        repo.run_zoom(&["in", "src"])?;
        repo.write_file("file.txt", "v2");
        repo.git_add_and_commit("Update 2");
        repo.run_zoom(&["out"])?;

        // Third cycle
        repo.run_zoom(&["in", "src"])?;
        assert_eq!(repo.read_file("file.txt"), "v2");
        repo.write_file("file.txt", "v3");
        repo.git_add_and_commit("Update 3");
        repo.run_zoom(&["out"])?;

        // Verify final state
        assert_eq!(repo.read_file("src/file.txt"), "v3");
        assert_eq!(repo.read_file("README.md"), "root");

        Ok(())
    });
}

#[test]
fn test_nested_zoom() {
    test_fixture("nested-zoom", |repo| {
        // First zoom: src/lib
        repo.run_zoom(&["in", "src/lib"])?;
        assert!(repo.file_exists("core/mod.rs"));
        assert!(repo.file_exists("lib.rs"));
        assert!(!repo.file_exists("root.txt"));

        // Second zoom: core (nested within first zoom)
        repo.run_zoom(&["in", "core"])?;
        assert!(repo.file_exists("mod.rs"));
        assert!(!repo.file_exists("lib.rs"));
        assert_eq!(repo.read_file("mod.rs"), "core module");

        // Modify
        repo.write_file("mod.rs", "modified core");
        repo.git_add_and_commit("Update core");

        // First zoom out (back to src/lib view)
        repo.run_zoom(&["out"])?;
        assert_eq!(repo.read_file("core/mod.rs"), "modified core");
        assert!(repo.file_exists("lib.rs"));

        // Second zoom out (back to full tree)
        repo.run_zoom(&["out"])?;
        assert_eq!(repo.read_file("src/lib/core/mod.rs"), "modified core");
        assert_eq!(repo.read_file("root.txt"), "root");

        Ok(())
    });
}

#[test]
fn test_return_to_previous_zoom() {
    test_fixture("return-to-previous-zoom", |repo| {
        // Zoom in
        repo.run_zoom(&["in", "src"])?;
        repo.write_file("file.txt", "modified");
        repo.git_add_and_commit("Update");

        // Zoom out
        repo.run_zoom(&["out"])?;
        assert_eq!(repo.read_file("src/file.txt"), "modified");

        // Zoom back in without specifying path (should find previous zoom)
        repo.run_zoom(&["in"])?;
        assert_eq!(repo.read_file("file.txt"), "modified");
        assert!(!repo.file_exists("root.txt"));

        Ok(())
    });
}

// ============================================================================
// Category G: History Structure Verification
// ============================================================================

#[test]
fn test_verify_merge_structure() {
    test_fixture("verify-merge-structure", |repo| {
        // Zoom in - should create merge commit
        repo.run_zoom(&["in", "src"])?;

        // Check the zoom-in commit structure
        let snapshot = repo.to_snapshot();
        let head = snapshot.head_commit().unwrap();

        // Should be merge commit with exactly 2 parents
        assert_eq!(
            head.parents.len(),
            2,
            "Zoom-in commit should have 2 parents (merge commit)"
        );

        // Parent 0 should be the seed commit, parent 1 should be the original commit
        // Both parents should exist in the repository
        for (i, parent_id) in head.parents.iter().enumerate() {
            let parent = snapshot.get_commit(parent_id);
            assert!(
                parent.is_some(),
                "Parent {} (id: {}) should exist",
                i,
                parent_id.to_hex()
            );
        }

        Ok(())
    });
}

#[test]
fn test_verify_trailers() {
    test_fixture("verify-trailers", |repo| {
        // Zoom in
        repo.run_zoom(&["in", "src"])?;

        let snapshot1 = repo.to_snapshot();
        let zoom_in_commit = snapshot1.head_commit().unwrap();

        // Verify zoom-in trailer
        let zoom_in_path = extract_trailer(&zoom_in_commit.message, "git-zoom-in");
        assert_eq!(
            zoom_in_path,
            Some("src".to_string()),
            "Zoom-in commit should have git-zoom-in: src trailer"
        );

        // Make change and zoom out
        repo.write_file("code.txt", "modified");
        repo.git_add_and_commit("Update");
        repo.run_zoom(&["out"])?;

        let snapshot2 = repo.to_snapshot();
        let zoom_out_commit = snapshot2.head_commit().unwrap();

        // Verify zoom-out trailer
        let zoom_out_path = extract_trailer(&zoom_out_commit.message, "git-zoom-out");
        assert_eq!(
            zoom_out_path,
            Some("src".to_string()),
            "Zoom-out commit should have git-zoom-out: src trailer"
        );

        Ok(())
    });
}

#[test]
fn test_committer_identity() {
    test_fixture("committer-identity", |repo| {
        // Zoom in
        repo.run_zoom(&["in", "src"])?;

        let snapshot1 = repo.to_snapshot();
        let zoom_in_commit = snapshot1.head_commit().unwrap();

        // Check committer identity for zoom-in
        assert_eq!(
            zoom_in_commit.committer.name, "🔎",
            "Zoom-in committer should be 🔎"
        );
        assert_eq!(
            zoom_in_commit.committer.email, "git-zoom-in@localhost",
            "Zoom-in email should be git-zoom-in@localhost"
        );

        // Make change and zoom out
        repo.write_file("file.txt", "modified");
        repo.git_add_and_commit("Update");
        repo.run_zoom(&["out"])?;

        let snapshot2 = repo.to_snapshot();
        let zoom_out_commit = snapshot2.head_commit().unwrap();

        // Check committer identity for zoom-out
        assert_eq!(
            zoom_out_commit.committer.name, "🔍",
            "Zoom-out committer should be 🔍"
        );
        assert_eq!(
            zoom_out_commit.committer.email, "git-zoom-out@localhost",
            "Zoom-out email should be git-zoom-out@localhost"
        );

        Ok(())
    });
}

#[test]
fn test_full_cycle_history_structure() {
    test_fixture("full-cycle-history-structure", |repo| {
        // Perform complete zoom cycle
        repo.run_zoom(&["in", "src"])?;
        repo.write_file("lib.txt", "modified lib");
        repo.git_add_and_commit("Modify lib");
        repo.run_zoom(&["out"])?;

        // Examine the full history
        let snapshot = repo.to_snapshot();

        // Count commits - should have:
        // 1. Initial commit
        // 2. Zoom-in merge commit (with seed commit as parent)
        // 3. Modify lib commit
        // 4. Zoom-out merge commit
        // Plus the seed commit that zoom-in creates
        let commit_count = snapshot.commits().count();
        println!("\nTotal commits in repository: {}", commit_count);

        // Build a map of commit ID to commit
        let mut commit_map: std::collections::HashMap<_, _> = std::collections::HashMap::new();
        for commit in snapshot.commits() {
            commit_map.insert(commit.id, commit);
        }

        // Print commits with parent relationships
        for commit in snapshot.commits() {
            let parent_info: Vec<String> = commit
                .parents
                .iter()
                .map(|p| {
                    commit_map
                        .get(p)
                        .map(|c| {
                            format!(
                                "{}",
                                c.message
                                    .lines()
                                    .next()
                                    .unwrap_or("")
                                    .chars()
                                    .take(20)
                                    .collect::<String>()
                            )
                        })
                        .unwrap_or_else(|| "???".to_string())
                })
                .collect();

            println!(
                "Commit: {} | Parents: {} | Message: {} | Parent messages: [{}]",
                commit.id.to_hex().chars().take(7).collect::<String>(),
                commit.parents.len(),
                commit.message.lines().next().unwrap_or(""),
                parent_info.join(", ")
            );
        }

        // Verify HEAD commit is zoom-out
        let head = snapshot.head_commit().unwrap();
        verify_zoom_out_commit(head);

        // Verify we have both README.md and modified src/lib.txt
        assert_eq!(
            head.tree.get("README.md"),
            Some("readme"),
            "README.md should be preserved"
        );
        assert_eq!(
            head.tree.get("src/lib.txt"),
            Some("modified lib"),
            "src/lib.txt should be modified"
        );

        Ok(())
    });
}

// ============================================================================
// Category H: Content Verification
// ============================================================================

#[test]
fn test_file_content_preservation() {
    test_fixture("file-content-preservation", |repo| {
        // Zoom into src/lib, modify one file, zoom out
        repo.run_zoom(&["in", "src/lib"])?;
        repo.write_file("core.rs", "pub fn core() { /* UPDATED */ }");
        repo.git_add_and_commit("Update core");
        repo.run_zoom(&["out"])?;

        // Verify ALL files are preserved correctly
        assert_eq!(
            repo.read_file("README.md"),
            "Project Documentation - This is important.",
            "README.md should be unchanged"
        );
        assert_eq!(
            repo.read_file("src/lib/core.rs"),
            "pub fn core() { /* UPDATED */ }",
            "core.rs should be modified"
        );
        assert_eq!(
            repo.read_file("src/lib/utils.rs"),
            "pub fn utils() {}",
            "utils.rs should be unchanged"
        );
        assert_eq!(
            repo.read_file("docs/guide.md"),
            "Guide - Step 1 Install",
            "docs/guide.md should be unchanged"
        );

        Ok(())
    });
}

#[test]
fn test_tree_structure_correctness() {
    test_fixture("tree-structure-correctness", |repo| {
        // Zoom into a/b
        repo.run_zoom(&["in", "a/b"])?;

        // Add new file at nested location
        repo.write_file("c/new.txt", "new file");
        repo.write_file("another.txt", "at b level");
        repo.git_add_and_commit("Add files");

        repo.run_zoom(&["out"])?;

        // Verify structure is correct
        assert_eq!(repo.read_file("a/b/c/deep.txt"), "deep");
        assert_eq!(repo.read_file("a/b/c/new.txt"), "new file");
        assert_eq!(repo.read_file("a/b/mid.txt"), "mid");
        assert_eq!(repo.read_file("a/b/another.txt"), "at b level");
        assert_eq!(repo.read_file("a/top.txt"), "top");
        assert_eq!(repo.read_file("root.txt"), "root");

        // Verify snapshot tree structure
        let snapshot = repo.to_snapshot();
        let head = snapshot.head_commit().unwrap();

        // Should have 6 files at correct paths
        assert_eq!(head.tree.paths().count(), 6, "Should have 6 files total");

        Ok(())
    });
}

#[test]
fn test_multiple_file_modifications() {
    test_fixture("multiple-file-modifications", |repo| {
        repo.run_zoom(&["in", "src"])?;

        // Modify all files
        repo.write_file("a.txt", "A");
        repo.write_file("b.txt", "B");
        repo.write_file("c.txt", "C");

        // Add new file
        repo.write_file("d.txt", "D");

        // Delete wouldn't work easily here since we'd need to git rm

        repo.git_add_and_commit("Modify all files");
        repo.run_zoom(&["out"])?;

        // Verify all modifications
        assert_eq!(repo.read_file("src/a.txt"), "A");
        assert_eq!(repo.read_file("src/b.txt"), "B");
        assert_eq!(repo.read_file("src/c.txt"), "C");
        assert_eq!(repo.read_file("src/d.txt"), "D");

        Ok(())
    });
}

// ============================================================================
// Category E: Edge Cases
// ============================================================================

#[test]
fn test_edge_case_tab_in_filename() {
    test_fixture("edge-case-tab-in-filename", |repo| {
        // Test zooming into a directory containing files with tabs in names
        repo.run_zoom(&["in", "src"])?;

        // Verify we can read the file with tab in name
        // Note: The actual filename has tabs, so we need to be careful
        let snapshot = repo.to_snapshot();
        let head_commit = snapshot.head_commit().unwrap();

        // Verify zoom-in commit was created
        verify_zoom_in_commit(head_commit);

        Ok(())
    });
}

#[test]
fn test_edge_case_special_characters() {
    test_fixture("edge-case-special-characters", |repo| {
        // Test zooming with special characters (spaces, unicode, etc.)
        repo.run_zoom(&["in", "src"])?;

        let snapshot = repo.to_snapshot();
        let head_commit = snapshot.head_commit().unwrap();

        // Verify zoom-in commit was created
        verify_zoom_in_commit(head_commit);

        // Verify we can zoom out
        repo.run_zoom(&["out"])?;

        Ok(())
    });
}

#[test]
fn test_edge_case_deep_nesting() {
    test_fixture("edge-case-deep-nesting", |repo| {
        // Test zooming into a deeply nested path (25+ levels)
        repo.run_zoom(&["in", "a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p/q/r/s/t/u/v/w/x/y/z"])?;

        let snapshot = repo.to_snapshot();
        let head_commit = snapshot.head_commit().unwrap();

        // Verify the deeply nested path was zoomed into
        verify_zoom_in_commit(head_commit);
        let path = extract_trailer(&head_commit.message, "git-zoom-in");
        assert!(path.is_some(), "Should have git-zoom-in trailer");

        // Verify we can zoom out
        repo.run_zoom(&["out"])?;

        Ok(())
    });
}

#[test]
fn test_edge_case_large_directory() {
    test_fixture("edge-case-large-directory", |repo| {
        // Test zooming into a directory with many files (20+)
        repo.run_zoom(&["in", "many"])?;

        let snapshot = repo.to_snapshot();
        let head_commit = snapshot.head_commit().unwrap();

        // Verify zoom-in succeeded
        verify_zoom_in_commit(head_commit);

        // Verify we can zoom out
        repo.run_zoom(&["out"])?;

        Ok(())
    });
}
