//! git zoom out implementation.

use crate::git;
use crate::scan;
use crate::tree;

const COMMITTER_NAME: &str = "🔍";
const COMMITTER_EMAIL: &str = "git-zoom-out@localhost";

/// Normalize a path: strip trailing/leading slashes.
fn normalize_path(path: &str) -> String {
    path.trim_matches('/').to_string()
}

/// Parse target[:path] argument.
fn parse_target_path(arg: Option<&str>) -> (Option<String>, Option<String>) {
    match arg {
        None => (None, None),
        Some(s) => {
            if let Some((target, path)) = s.split_once(':') {
                // Handle ":path" case (empty target means None)
                let target_opt = if target.is_empty() {
                    None
                } else {
                    Some(target.to_string())
                };
                // Normalize path to match zoom_in behavior
                let path_normalized = normalize_path(path);
                let path_opt = if path_normalized.is_empty() {
                    None
                } else {
                    Some(path_normalized)
                };
                (target_opt, path_opt)
            } else {
                // No colon - treat as target only
                (Some(s.to_string()), None)
            }
        }
    }
}

/// Execute git zoom out.
pub fn zoom_out(target_and_path: Option<&str>, deny_empty: bool) -> git::Result<()> {
    // 1. Parse target and path from argument
    let (explicit_target, explicit_path) = parse_target_path(target_and_path);

    // 2. Scan for git-zoom-in trailer (filtered by path if specified)
    let found = scan::scan_for_zoom_in(explicit_path.as_deref())?;
    let found = found.ok_or_else(|| git::Error {
        command: "zoom out".to_string(),
        message: if let Some(p) = &explicit_path {
            format!("no zoom-in commit found in history for path '{}'", p)
        } else {
            "no zoom-in commit found in history".to_string()
        },
    })?;

    // 3. Determine path
    let path = explicit_path.unwrap_or(found.path);

    // 4. Determine base commit (where we zoomed in from)
    // found.second_parent = full-tree commit we zoomed in from
    let base_commit = found.second_parent.ok_or_else(|| git::Error {
        command: "zoom out".to_string(),
        message: "zoom-in commit has no second parent".to_string(),
    })?;

    // 5. Determine target commit (where to merge into full-tree lineage)
    let target_commit = match explicit_target {
        Some(t) => git::rev_parse(&t)?,
        None => base_commit.clone(),
    };

    // 6. Get current HEAD tree (subtree content)
    let head_commit = git::head()?;
    let head_tree = git::rev_parse("HEAD^{tree}")?;

    // 7. Check for empty subtree if --deny-empty
    if deny_empty {
        let entries = git::ls_tree(&head_tree)?;
        if entries.is_empty() {
            return Err(git::Error {
                command: "zoom out".to_string(),
                message: "subtree is empty (use without --deny-empty to allow)".to_string(),
            });
        }
    }

    // 8. Build full tree: target's tree with path replaced by head's tree
    let target_tree = git::rev_parse(&format!("{}^{{tree}}", target_commit))?;
    let new_full_tree = tree::replace_subtree(&target_tree, &path, &head_tree)?;

    // 9. Create merge commit
    let merge_msg = format!("Merge to '{}'\n\ngit-zoom-out: {}", path, path);
    let merge_commit = git::commit_tree(
        &new_full_tree,
        &[&target_commit, &head_commit],
        &merge_msg,
        COMMITTER_NAME,
        COMMITTER_EMAIL,
    )?;

    // 10. Update HEAD and reset
    git::update_ref_head(&merge_commit)?;
    git::reset_hard()?;

    eprintln!("Zoomed out from '{}' at {}", path, &merge_commit[..8]);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::zoom_in;
    use std::fs;
    use std::process::Command;
    use tempfile::TempDir;

    fn setup_test_repo() -> TempDir {
        let dir = TempDir::new().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        Command::new("git")
            .args(["init"])
            .current_dir(dir.path())
            .output()
            .unwrap();

        Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(dir.path())
            .output()
            .unwrap();

        Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(dir.path())
            .output()
            .unwrap();

        dir
    }

    #[test]
    fn test_zoom_out_basic() {
        let dir = setup_test_repo();

        // Create initial structure
        fs::create_dir_all(dir.path().join("src/lib")).unwrap();
        fs::write(dir.path().join("src/lib/foo.txt"), "original").unwrap();
        fs::write(dir.path().join("root.txt"), "root").unwrap();

        Command::new("git")
            .args(["add", "."])
            .current_dir(dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "initial"])
            .current_dir(dir.path())
            .output()
            .unwrap();

        let original_head = git::head().unwrap();

        // Zoom in to src/lib
        zoom_in::zoom_in(Some("src/lib"), false).unwrap();

        // Make a change in the subtree
        fs::write(dir.path().join("foo.txt"), "modified").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "modify foo"])
            .current_dir(dir.path())
            .output()
            .unwrap();

        // Zoom out
        zoom_out(None, false).unwrap();

        // Verify: working directory should have full tree structure
        assert!(dir.path().join("src/lib/foo.txt").exists());
        assert!(dir.path().join("root.txt").exists());

        // Verify: src/lib/foo.txt should have modified content
        let content = fs::read_to_string(dir.path().join("src/lib/foo.txt")).unwrap();
        assert_eq!(content, "modified");

        // Verify: HEAD should have git-zoom-out trailer
        let body = git::commit_body("HEAD").unwrap();
        assert!(body.contains("git-zoom-out: src/lib"));

        // Verify: should have 2 parents (original and subtree work)
        let parents = git::parents("HEAD").unwrap();
        assert_eq!(parents.len(), 2);
        assert_eq!(parents[0], original_head);
    }

    #[test]
    fn test_zoom_cycle() {
        let dir = setup_test_repo();

        // Create initial structure
        fs::create_dir_all(dir.path().join("src/lib")).unwrap();
        fs::write(dir.path().join("src/lib/foo.txt"), "v1").unwrap();
        fs::write(dir.path().join("root.txt"), "root").unwrap();

        Command::new("git")
            .args(["add", "."])
            .current_dir(dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "initial"])
            .current_dir(dir.path())
            .output()
            .unwrap();

        // First cycle: zoom in, modify, zoom out
        zoom_in::zoom_in(Some("src/lib"), false).unwrap();
        fs::write(dir.path().join("foo.txt"), "v2").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "v2"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        zoom_out(None, false).unwrap();

        // Second cycle: zoom in again (should continue from v2), modify, zoom out
        zoom_in::zoom_in(None, false).unwrap(); // No path - should use previous

        // Verify we're continuing from v2
        let content = fs::read_to_string(dir.path().join("foo.txt")).unwrap();
        assert_eq!(content, "v2");

        fs::write(dir.path().join("foo.txt"), "v3").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "v3"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        zoom_out(None, false).unwrap();

        // Verify final state
        let content = fs::read_to_string(dir.path().join("src/lib/foo.txt")).unwrap();
        assert_eq!(content, "v3");
    }

    #[test]
    fn test_parse_target_path() {
        assert_eq!(parse_target_path(None), (None, None));
        assert_eq!(
            parse_target_path(Some("abc123")),
            (Some("abc123".to_string()), None)
        );
        assert_eq!(
            parse_target_path(Some("abc123:src/lib")),
            (Some("abc123".to_string()), Some("src/lib".to_string()))
        );
        assert_eq!(
            parse_target_path(Some(":src/lib")),
            (None, Some("src/lib".to_string()))
        );
        // Path normalization - trailing slashes stripped
        assert_eq!(
            parse_target_path(Some(":src/lib/")),
            (None, Some("src/lib".to_string()))
        );
        assert_eq!(
            parse_target_path(Some("abc123:/src/lib/")),
            (Some("abc123".to_string()), Some("src/lib".to_string()))
        );
        // Empty path after normalization becomes None
        assert_eq!(parse_target_path(Some(":")), (None, None));
        assert_eq!(parse_target_path(Some(":/")), (None, None));
    }

    #[test]
    fn test_multiple_paths() {
        let dir = setup_test_repo();

        // Create structure with two subtrees
        fs::create_dir_all(dir.path().join("src/lib")).unwrap();
        fs::create_dir_all(dir.path().join("src/bin")).unwrap();
        fs::write(dir.path().join("src/lib/lib.txt"), "library").unwrap();
        fs::write(dir.path().join("src/bin/main.txt"), "binary").unwrap();

        Command::new("git")
            .args(["add", "."])
            .current_dir(dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "initial"])
            .current_dir(dir.path())
            .output()
            .unwrap();

        // Zoom into lib, modify, zoom out
        zoom_in::zoom_in(Some("src/lib"), false).unwrap();
        fs::write(dir.path().join("lib.txt"), "modified library").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "modify lib"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        zoom_out(None, false).unwrap();

        // Zoom into bin, modify, zoom out
        zoom_in::zoom_in(Some("src/bin"), false).unwrap();
        fs::write(dir.path().join("main.txt"), "modified binary").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "modify bin"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        zoom_out(None, false).unwrap();

        // Verify both changes exist
        let lib_content = fs::read_to_string(dir.path().join("src/lib/lib.txt")).unwrap();
        let bin_content = fs::read_to_string(dir.path().join("src/bin/main.txt")).unwrap();
        assert_eq!(lib_content, "modified library");
        assert_eq!(bin_content, "modified binary");
    }

    #[test]
    fn test_deny_empty() {
        let dir = setup_test_repo();

        fs::create_dir_all(dir.path().join("src/lib")).unwrap();
        fs::write(dir.path().join("src/lib/foo.txt"), "hello").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "initial"])
            .current_dir(dir.path())
            .output()
            .unwrap();

        // Zoom in with allow-empty
        zoom_in::zoom_in(Some("empty/path"), true).unwrap();

        // Try to zoom out with --deny-empty (should fail)
        let result = zoom_out(None, true);
        assert!(result.is_err());

        // Without --deny-empty should work
        let result = zoom_out(None, false);
        assert!(result.is_ok());
    }
}
