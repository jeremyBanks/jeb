//! Wrapper functions for git commands.

use std::io::Write;
use std::process::{Command, Output, Stdio};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    pub command: String,
    pub message: String,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "git {}: {}", self.command, self.message)
    }
}

impl std::error::Error for Error {}

/// Run a git command and return the output.
fn git(args: &[&str]) -> Result<Output> {
    let output = Command::new("git")
        .args(args)
        .output()
        .map_err(|e| Error {
            command: args.join(" "),
            message: format!("failed to execute: {}", e),
        })?;
    Ok(output)
}

/// Run a git command and return stdout as string, or error if non-zero exit.
fn git_stdout(args: &[&str]) -> Result<String> {
    let output = git(args)?;
    if !output.status.success() {
        return Err(Error {
            command: args.join(" "),
            message: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        });
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Check if we're at the root of a git repository.
pub fn check_repo_root() -> Result<()> {
    let toplevel = git_stdout(&["rev-parse", "--show-toplevel"])?;
    let cwd = std::env::current_dir()
        .map_err(|e| Error {
            command: "cwd".to_string(),
            message: e.to_string(),
        })?;

    let cwd_str = cwd.to_string_lossy();
    if cwd_str != toplevel {
        return Err(Error {
            command: "check_repo_root".to_string(),
            message: format!(
                "must be run from repository root (current: {}, root: {})",
                cwd_str, toplevel
            ),
        });
    }
    Ok(())
}

/// Check if working tree is clean.
pub fn check_clean() -> Result<()> {
    let status = git_stdout(&["status", "--porcelain"])?;
    if !status.is_empty() {
        return Err(Error {
            command: "status".to_string(),
            message: "working tree has uncommitted changes".to_string(),
        });
    }
    Ok(())
}

/// Get the current HEAD commit hash.
pub fn head() -> Result<String> {
    git_stdout(&["rev-parse", "HEAD"])
}

/// Resolve a revision to a commit hash.
pub fn rev_parse(rev: &str) -> Result<String> {
    git_stdout(&["rev-parse", rev])
}

/// Try to resolve a revision, returning None if it doesn't exist.
pub fn try_rev_parse(rev: &str) -> Result<Option<String>> {
    let output = git(&["rev-parse", "--verify", "--quiet", rev])?;
    if output.status.success() {
        Ok(Some(String::from_utf8_lossy(&output.stdout).trim().to_string()))
    } else {
        Ok(None)
    }
}

/// Get tree hash at a path within a commit.
pub fn tree_at_path(commit: &str, path: &str) -> Result<Option<String>> {
    let rev = format!("{}:{}", commit, path);
    try_rev_parse(&rev)
}

/// List entries in a tree.
/// Returns Vec of (mode, type, hash, name).
pub fn ls_tree(tree: &str) -> Result<Vec<(String, String, String, String)>> {
    let output = git_stdout(&["ls-tree", tree])?;
    let mut entries = Vec::new();
    for line in output.lines() {
        if line.is_empty() {
            continue;
        }
        // Format: <mode> <type> <hash>\t<name>
        let (meta, name) = line.split_once('\t').ok_or_else(|| Error {
            command: "ls-tree".to_string(),
            message: format!("malformed line: {}", line),
        })?;
        let parts: Vec<&str> = meta.split_whitespace().collect();
        if parts.len() != 3 {
            return Err(Error {
                command: "ls-tree".to_string(),
                message: format!("malformed line: {}", line),
            });
        }
        entries.push((
            parts[0].to_string(),
            parts[1].to_string(),
            parts[2].to_string(),
            name.to_string(),
        ));
    }
    Ok(entries)
}

/// Create a tree from entries (using mktree).
/// Entries are (mode, type, hash, name).
pub fn mktree(entries: &[(String, String, String, String)]) -> Result<String> {
    let mut child = Command::new("git")
        .args(["mktree"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| Error {
            command: "mktree".to_string(),
            message: format!("failed to spawn: {}", e),
        })?;

    {
        let stdin = child.stdin.as_mut().unwrap();
        for (mode, obj_type, hash, name) in entries {
            writeln!(stdin, "{} {} {}\t{}", mode, obj_type, hash, name).map_err(|e| Error {
                command: "mktree".to_string(),
                message: format!("failed to write: {}", e),
            })?;
        }
    }

    let output = child.wait_with_output().map_err(|e| Error {
        command: "mktree".to_string(),
        message: format!("failed to wait: {}", e),
    })?;

    if !output.status.success() {
        return Err(Error {
            command: "mktree".to_string(),
            message: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        });
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Create an empty tree.
pub fn empty_tree() -> Result<String> {
    mktree(&[])
}

/// Create a commit with the given tree, parents, and message.
/// Uses custom committer identity.
pub fn commit_tree(
    tree: &str,
    parents: &[&str],
    message: &str,
    committer_name: &str,
    committer_email: &str,
) -> Result<String> {
    let mut args = vec!["commit-tree", tree];
    for parent in parents {
        args.push("-p");
        args.push(parent);
    }
    args.push("-m");
    args.push(message);

    let output = Command::new("git")
        .args(&args)
        .env("GIT_COMMITTER_NAME", committer_name)
        .env("GIT_COMMITTER_EMAIL", committer_email)
        .output()
        .map_err(|e| Error {
            command: args.join(" "),
            message: format!("failed to execute: {}", e),
        })?;

    if !output.status.success() {
        return Err(Error {
            command: args.join(" "),
            message: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        });
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Update HEAD to point to a commit.
pub fn update_ref_head(commit: &str) -> Result<()> {
    git_stdout(&["update-ref", "HEAD", commit])?;
    Ok(())
}

/// Reset working tree to match HEAD.
pub fn reset_hard() -> Result<()> {
    git_stdout(&["reset", "--hard", "HEAD"])?;
    Ok(())
}

/// Get the commit message body for a commit.
pub fn commit_body(commit: &str) -> Result<String> {
    git_stdout(&["log", "-1", "--format=%B", commit])
}

/// Get parents of a commit.
pub fn parents(commit: &str) -> Result<Vec<String>> {
    let output = git_stdout(&["rev-parse", &format!("{}^@", commit)])?;
    Ok(output.lines().map(|s| s.to_string()).collect())
}

/// Walk first-parent history, yielding (commit_hash, parents, body) for each.
pub fn walk_first_parent(start: &str) -> Result<Vec<(String, Vec<String>, String)>> {
    // Use a format with a unique record separator (ASCII RS = 0x1e)
    // Format: hash\x00parents\x00body\x1e
    let output = git_stdout(&[
        "log",
        "--first-parent",
        "--format=%H%x00%P%x00%B%x1e",
        start,
    ])?;

    let mut results = Vec::new();
    for record in output.split('\x1e') {
        let record = record.trim();
        if record.is_empty() {
            continue;
        }
        let parts: Vec<&str> = record.splitn(3, '\x00').collect();
        if parts.len() < 3 {
            continue;
        }
        let hash = parts[0].to_string();
        let parent_list: Vec<String> = parts[1]
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
        let body = parts[2].to_string();
        results.push((hash, parent_list, body));
    }
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
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
    fn test_empty_tree() {
        let _dir = setup_test_repo();
        let tree = empty_tree().unwrap();
        // Git's empty tree hash is well-known
        assert_eq!(tree, "4b825dc642cb6eb9a060e54bf8d69288fbee4904");
    }

    #[test]
    fn test_mktree_and_ls_tree() {
        let dir = setup_test_repo();

        // Create a file and add it
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

        let tree = rev_parse("HEAD^{tree}").unwrap();
        let entries = ls_tree(&tree).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].3, "test.txt");
        assert_eq!(entries[0].1, "blob");
    }
}
