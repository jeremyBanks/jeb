//! git zoom in implementation.

use crate::{
    git,
    scan,
};

const COMMITTER_NAME: &str = "🔎";
const COMMITTER_EMAIL: &str = "git-zoom-in@localhost";

/// Normalize a path: strip trailing slashes, remove `.` components, reject
/// `..`.
fn normalize_path(path: &str) -> git::Result<String> {
    let parts: Vec<&str> = path
        .split('/')
        .filter(|s| !s.is_empty() && *s != ".")
        .collect();

    if parts.is_empty() {
        return Err(git::Error {
            command: "zoom in".to_string(),
            message: "invalid path: cannot zoom into repository root".to_string(),
        });
    }

    for part in &parts {
        if *part == ".." {
            return Err(git::Error {
                command: "zoom in".to_string(),
                message: "invalid path: '..' not allowed".to_string(),
            });
        }
    }

    Ok(parts.join("/"))
}

/// Execute git zoom in.
pub fn zoom_in(path: Option<&str>, allow_empty: bool) -> git::Result<()> {
    // 1. Resolve path and find existing subtree history
    let (target_path, zoom_out_found) = match path {
        None => {
            // No path specified: must find it from a previous zoom-out
            let found = scan::scan_for_zoom_out(None)?;
            match found {
                None => {
                    return Err(git::Error {
                        command: "zoom in".to_string(),
                        message: "no path specified and no previous zoom-out found".to_string(),
                    });
                }
                Some(f) => (f.path.clone(), Some(f)),
            }
        }
        Some(p) => {
            // Path specified: normalize and check if we have previous zoom-out
            let normalized = normalize_path(p)?;
            let found = scan::scan_for_zoom_out(Some(&normalized))?;
            (normalized, found)
        }
    };

    // 2. Check path exists in HEAD (or create empty tree if --allow-empty)
    let head_commit = git::head()?;
    let subtree_hash = match git::tree_at_path(&head_commit, &target_path)? {
        Some(hash) => hash,
        None => {
            if !allow_empty {
                return Err(git::Error {
                    command: "zoom in".to_string(),
                    message: format!("path '{}' does not exist in HEAD", target_path),
                });
            }
            git::empty_tree()?
        }
    };

    // 3. Determine merge first parent
    let merge_first_parent = match zoom_out_found {
        None => {
            // Fresh subtree: create orphan seed commit
            let empty_tree = git::empty_tree()?;
            let seed_msg = "Initial commit".to_string();
            git::commit_tree(&empty_tree, &[], &seed_msg, COMMITTER_NAME, COMMITTER_EMAIL)?
        }
        Some(found) => {
            // Existing subtree: continue from where we last zoomed out
            // found.second_parent = last subtree commit (S6 in example)
            found.second_parent.ok_or_else(|| git::Error {
                command: "zoom in".to_string(),
                message: "zoom-out commit has no second parent".to_string(),
            })?
        }
    };

    // 4. Create merge commit
    let merge_msg = format!(
        "Merge from '{}'\n\ngit-zoom-in: {}",
        target_path, target_path
    );
    let merge_commit = git::commit_tree(
        &subtree_hash,
        &[&merge_first_parent, &head_commit],
        &merge_msg,
        COMMITTER_NAME,
        COMMITTER_EMAIL,
    )?;

    // 5. Update HEAD and reset
    git::update_ref_head(&merge_commit)?;
    git::reset_hard()?;

    eprintln!("Zoomed in to '{}' at {}", target_path, &merge_commit[..8]);
    Ok(())
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        std::{
            fs,
            process::Command,
        },
        tempfile::TempDir,
    };

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
    fn test_zoom_in_fresh() {
        let dir = setup_test_repo();

        // Create initial structure
        fs::create_dir_all(dir.path().join("src/lib")).unwrap();
        fs::write(dir.path().join("src/lib/foo.txt"), "hello").unwrap();
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

        // Zoom in to src/lib
        zoom_in(Some("src/lib"), false).unwrap();

        // Verify: working directory should only have foo.txt
        assert!(dir.path().join("foo.txt").exists());
        assert!(!dir.path().join("root.txt").exists());
        assert!(!dir.path().join("src").exists());

        // Verify: HEAD should have git-zoom-in trailer
        let body = git::commit_body("HEAD").unwrap();
        assert!(body.contains("git-zoom-in: src/lib"));

        // Verify: should have 2 parents (seed and original)
        let parents = git::parents("HEAD").unwrap();
        assert_eq!(parents.len(), 2);
    }

    #[test]
    fn test_zoom_in_path_not_exist() {
        let dir = setup_test_repo();

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

        // Zoom in to non-existent path should fail
        let result = zoom_in(Some("src/lib"), false);
        assert!(result.is_err());
    }

    #[test]
    fn test_zoom_in_allow_empty() {
        let dir = setup_test_repo();

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

        // Zoom in to non-existent path with --allow-empty should succeed
        zoom_in(Some("src/lib"), true).unwrap();

        // Working directory should be empty
        let entries: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .filter(|e| e.as_ref().map(|e| e.file_name() != ".git").unwrap_or(false))
            .collect();
        assert_eq!(entries.len(), 0);
    }

    #[test]
    fn test_zoom_in_trailing_slash() {
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

        // Trailing slash should be normalized
        zoom_in(Some("src/lib/"), false).unwrap();

        // Verify: trailer should have normalized path
        let body = git::commit_body("HEAD").unwrap();
        assert!(body.contains("git-zoom-in: src/lib"));
        assert!(!body.contains("git-zoom-in: src/lib/"));
    }

    #[test]
    fn test_zoom_in_invalid_paths() {
        let dir = setup_test_repo();

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

        // Empty path should fail
        assert!(zoom_in(Some(""), false).is_err());
        assert!(zoom_in(Some("."), false).is_err());
        assert!(zoom_in(Some("/"), false).is_err());
        assert!(zoom_in(Some("./"), false).is_err());
        assert!(zoom_in(Some("/./"), false).is_err());

        // Path with .. should fail
        assert!(zoom_in(Some("src/../lib"), false).is_err());
        assert!(zoom_in(Some(".."), false).is_err());
    }

    #[test]
    fn test_zoom_in_dot_normalization() {
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

        // Path with . components should be normalized
        zoom_in(Some("./src/./lib"), false).unwrap();

        // Verify: trailer should have normalized path (no dots)
        let body = git::commit_body("HEAD").unwrap();
        assert!(body.contains("git-zoom-in: src/lib"));
        assert!(!body.contains("./"));
    }
}
