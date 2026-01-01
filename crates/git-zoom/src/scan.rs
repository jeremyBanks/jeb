//! First-parent history scanning for trailers.
use crate::git;
/// Result of scanning for a trailer.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Found {
    /// The commit hash where the trailer was found.
    pub commit: String,
    /// The path from the trailer value.
    pub path: String,
    /// The second parent of the commit (if any).
    pub second_parent: Option<String>,
}
/// Scan first-parent history for a trailer.
/// If filter_path is Some, only match trailers with that path value.
pub fn scan_for_trailer(
    trailer_name: &str,
    filter_path: Option<&str>,
) -> git::Result<Option<Found>> {
    let head = git::head()?;
    let history = git::walk_first_parent(&head)?;
    for (commit_hash, parents, body) in history {
        let trailer_prefix = format!("{}: ", trailer_name);
        for line in body.lines() {
            if let Some(path) = line.strip_prefix(&trailer_prefix) {
                let path = path.trim();
                if filter_path.is_some_and(|filter| path != filter) {
                    continue;
                }
                return Ok(
                    Some(Found {
                        commit: commit_hash,
                        path: path.to_string(),
                        second_parent: parents.get(1).cloned(),
                    }),
                );
            }
        }
    }
    Ok(None)
}
/// Scan for git-zoom-in trailer.
pub fn scan_for_zoom_in(filter_path: Option<&str>) -> git::Result<Option<Found>> {
    scan_for_trailer("git-zoom-in", filter_path)
}
/// Scan for git-zoom-out trailer.
pub fn scan_for_zoom_out(filter_path: Option<&str>) -> git::Result<Option<Found>> {
    scan_for_trailer("git-zoom-out", filter_path)
}
#[cfg(test)]
mod tests {
    use {
        super::*, std::{fs, process::Command},
        tempfile::TempDir,
    };
    fn setup_test_repo() -> TempDir {
        let dir = TempDir::new().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();
        Command::new("git").args(["init"]).current_dir(dir.path()).output().unwrap();
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
    fn test_scan_for_trailer_not_found() {
        let dir = setup_test_repo();
        fs::write(dir.path().join("test.txt"), "hello").unwrap();
        Command::new("git")
            .args(["add", "test.txt"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "initial"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        let result = scan_for_zoom_in(None).unwrap();
        assert!(result.is_none());
    }
    #[test]
    fn test_scan_for_trailer_found() {
        let dir = setup_test_repo();
        fs::write(dir.path().join("test.txt"), "hello").unwrap();
        Command::new("git")
            .args(["add", "test.txt"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "initial"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        fs::write(dir.path().join("test.txt"), "hello2").unwrap();
        Command::new("git")
            .args(["add", "test.txt"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "zoom\n\ngit-zoom-in: src/lib"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        let result = scan_for_zoom_in(None).unwrap();
        assert!(result.is_some());
        let found = result.unwrap();
        assert_eq!(found.path, "src/lib");
    }
    #[test]
    fn test_scan_with_filter() {
        let dir = setup_test_repo();
        fs::write(dir.path().join("test.txt"), "hello").unwrap();
        Command::new("git")
            .args(["add", "test.txt"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "initial"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        fs::write(dir.path().join("test.txt"), "hello2").unwrap();
        Command::new("git")
            .args(["add", "test.txt"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "zoom\n\ngit-zoom-in: src/lib"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        let result = scan_for_zoom_in(Some("src/other")).unwrap();
        assert!(result.is_none());
        let result = scan_for_zoom_in(Some("src/lib")).unwrap();
        assert!(result.is_some());
    }
}
