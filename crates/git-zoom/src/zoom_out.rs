//! git zoom out implementation.
use crate::{
    git,
    scan,
    tree,
};
const COMMITTER_NAME: &str = "🔍";
const COMMITTER_EMAIL: &str = "git-zoom-out@localhost";
/// Normalize a path: strip slashes, remove `.` components.
fn normalize_path(path: &str) -> String {
    path.split('/')
        .filter(|s| !s.is_empty() && *s != ".")
        .collect::<Vec<_>>()
        .join("/")
}
/// Parse target[:path] argument.
fn parse_target_path(arg: Option<&str>) -> (Option<String>, Option<String>) {
    match arg {
        None => (None, None),
        Some(s) => {
            if let Some((target, path)) = s.split_once(':') {
                let target_opt = if target.is_empty() {
                    None
                } else {
                    Some(target.to_string())
                };
                let path_normalized = normalize_path(path);
                let path_opt = if path_normalized.is_empty() {
                    None
                } else {
                    Some(path_normalized)
                };
                (target_opt, path_opt)
            } else {
                (Some(s.to_string()), None)
            }
        }
    }
}
/// Execute git zoom out.
pub fn zoom_out(target_and_path: Option<&str>, deny_empty: bool) -> git::Result<()> {
    let (explicit_target, explicit_path) = parse_target_path(target_and_path);
    let found = scan::scan_for_zoom_in(explicit_path.as_deref())?;
    let found = found.ok_or_else(|| git::Error {
        command: "zoom out".to_string(),
        message: if let Some(p) = &explicit_path {
            format!("no zoom-in commit found in history for path '{}'", p)
        } else {
            "no zoom-in commit found in history".to_string()
        },
    })?;
    let path = explicit_path.unwrap_or(found.path);
    let base_commit = found.second_parent.ok_or_else(|| git::Error {
        command: "zoom out".to_string(),
        message: "zoom-in commit has no second parent".to_string(),
    })?;
    let target_commit = match explicit_target {
        Some(t) => git::rev_parse(&t)?,
        None => base_commit.clone(),
    };
    let head_commit = git::head()?;
    let head_tree = git::rev_parse("HEAD^{tree}")?;
    if deny_empty {
        let entries = git::ls_tree(&head_tree)?;
        if entries.is_empty() {
            return Err(git::Error {
                command: "zoom out".to_string(),
                message: "subtree is empty (use without --deny-empty to allow)".to_string(),
            });
        }
    }
    let target_tree = git::rev_parse(&format!("{}^{{tree}}", target_commit))?;
    let new_full_tree = tree::replace_subtree(&target_tree, &path, &head_tree)?;
    let merge_msg = format!("Merge to '{}'\n\ngit-zoom-out: {}", path, path);
    let merge_commit = git::commit_tree(
        &new_full_tree,
        &[&target_commit, &head_commit],
        &merge_msg,
        COMMITTER_NAME,
        COMMITTER_EMAIL,
    )?;
    git::update_ref_head(&merge_commit)?;
    git::reset_hard()?;
    eprintln!("Zoomed out from '{}' at {}", path, &merge_commit[..8]);
    Ok(())
}
#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::zoom_in,
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
    fn test_zoom_out_basic() {
        let dir = setup_test_repo();
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
        zoom_in::zoom_in(Some("src/lib"), false).unwrap();
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
        zoom_out(None, false).unwrap();
        assert!(dir.path().join("src/lib/foo.txt").exists());
        assert!(dir.path().join("root.txt").exists());
        let content = fs::read_to_string(dir.path().join("src/lib/foo.txt")).unwrap();
        assert_eq!(content, "modified");
        let body = git::commit_body("HEAD").unwrap();
        assert!(body.contains("git-zoom-out: src/lib"));
        let parents = git::parents("HEAD").unwrap();
        assert_eq!(parents.len(), 2);
        assert_eq!(parents[0], original_head);
    }
    #[test]
    fn test_zoom_cycle() {
        let dir = setup_test_repo();
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
        zoom_in::zoom_in(None, false).unwrap();
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
        assert_eq!(
            parse_target_path(Some(":src/lib/")),
            (None, Some("src/lib".to_string()))
        );
        assert_eq!(
            parse_target_path(Some("abc123:/src/lib/")),
            (Some("abc123".to_string()), Some("src/lib".to_string()))
        );
        assert_eq!(parse_target_path(Some(":")), (None, None));
        assert_eq!(parse_target_path(Some(":/")), (None, None));
    }
    #[test]
    fn test_multiple_paths() {
        let dir = setup_test_repo();
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
        let lib_content = fs::read_to_string(dir.path().join("src/lib/lib.txt")).unwrap();
        let bin_content = fs::read_to_string(dir.path().join("src/bin/main.txt")).unwrap();
        assert_eq!(lib_content, "modified library");
        assert_eq!(bin_content, "modified binary");
    }
    #[test]
    fn test_nested_zoom() {
        let dir = setup_test_repo();
        fs::create_dir_all(dir.path().join("src/lib/core")).unwrap();
        fs::write(dir.path().join("src/lib/core/mod.rs"), "v1").unwrap();
        fs::write(dir.path().join("src/lib/lib.rs"), "lib").unwrap();
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
        zoom_in::zoom_in(Some("src/lib"), false).unwrap();
        assert!(dir.path().join("core/mod.rs").exists());
        assert!(dir.path().join("lib.rs").exists());
        assert!(!dir.path().join("root.txt").exists());
        zoom_in::zoom_in(Some("core"), false).unwrap();
        assert!(dir.path().join("mod.rs").exists());
        assert!(!dir.path().join("lib.rs").exists());
        fs::write(dir.path().join("mod.rs"), "v2").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "modify in nested"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        zoom_out(None, false).unwrap();
        assert!(dir.path().join("core/mod.rs").exists());
        assert!(dir.path().join("lib.rs").exists());
        let content = fs::read_to_string(dir.path().join("core/mod.rs")).unwrap();
        assert_eq!(content, "v2");
        zoom_out(None, false).unwrap();
        assert!(dir.path().join("src/lib/core/mod.rs").exists());
        assert!(dir.path().join("root.txt").exists());
        let content = fs::read_to_string(dir.path().join("src/lib/core/mod.rs")).unwrap();
        assert_eq!(content, "v2");
    }
    #[test]
    fn test_explicit_target() {
        let dir = setup_test_repo();
        fs::create_dir_all(dir.path().join("src/lib")).unwrap();
        fs::write(dir.path().join("src/lib/foo.txt"), "v1").unwrap();
        fs::write(dir.path().join("root.txt"), "root v1").unwrap();
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
        let initial_commit = git::head().unwrap();
        fs::write(dir.path().join("root.txt"), "root v2").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "update root"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        let updated_commit = git::head().unwrap();
        zoom_in::zoom_in(Some("src/lib"), false).unwrap();
        fs::write(dir.path().join("foo.txt"), "v2").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "update foo"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        zoom_out(Some(&format!("{}:src/lib", updated_commit)), false).unwrap();
        let root_content = fs::read_to_string(dir.path().join("root.txt")).unwrap();
        let foo_content = fs::read_to_string(dir.path().join("src/lib/foo.txt")).unwrap();
        assert_eq!(root_content, "root v2");
        assert_eq!(foo_content, "v2");
        let parents = git::parents("HEAD").unwrap();
        assert_eq!(parents[0], updated_commit);
    }
    #[test]
    fn test_explicit_path_filter() {
        let dir = setup_test_repo();
        fs::create_dir_all(dir.path().join("src/lib")).unwrap();
        fs::create_dir_all(dir.path().join("src/bin")).unwrap();
        fs::write(dir.path().join("src/lib/lib.txt"), "lib").unwrap();
        fs::write(dir.path().join("src/bin/main.txt"), "bin").unwrap();
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
        zoom_in::zoom_in(Some("src/lib"), false).unwrap();
        fs::write(dir.path().join("lib.txt"), "lib v2").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "update lib"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        zoom_out(None, false).unwrap();
        zoom_in::zoom_in(Some("src/bin"), false).unwrap();
        fs::write(dir.path().join("main.txt"), "bin v2").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "update bin"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        zoom_out(Some(":src/bin"), false).unwrap();
        let lib_content = fs::read_to_string(dir.path().join("src/lib/lib.txt")).unwrap();
        let bin_content = fs::read_to_string(dir.path().join("src/bin/main.txt")).unwrap();
        assert_eq!(lib_content, "lib v2");
        assert_eq!(bin_content, "bin v2");
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
        zoom_in::zoom_in(Some("empty/path"), true).unwrap();
        let result = zoom_out(None, true);
        assert!(result.is_err());
        let result = zoom_out(None, false);
        assert!(result.is_ok());
    }
}
