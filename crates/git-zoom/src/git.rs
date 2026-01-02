//! Wrapper functions for git commands.
use std::{
    io::Write,
    process::{
        Command,
        Output,
        Stdio,
    },
};
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
    let output = Command::new("git").args(args).output().map_err(|e| Error {
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
    let cwd = std::env::current_dir().map_err(|e| Error {
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
        Ok(Some(
            String::from_utf8_lossy(&output.stdout).trim().to_string(),
        ))
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
/// Timestamps are deterministically derived from parent commits.
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

    // Determine timestamp deterministically from parents
    let timestamp = if parents.is_empty() {
        // Root commit - use a fixed default
        "2024-12-06T06:12:24-06:24".to_string()
    } else {
        // Use latest parent timestamp
        latest_parent_timestamp(parents)?
    };

    let output = Command::new("git")
        .args(&args)
        .env("GIT_COMMITTER_NAME", committer_name)
        .env("GIT_COMMITTER_EMAIL", committer_email)
        .env("GIT_AUTHOR_DATE", &timestamp)
        .env("GIT_COMMITTER_DATE", &timestamp)
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

/// Get author and committer timestamps for a commit in ISO8601 format.
/// Returns (author_date, committer_date).
pub fn get_commit_timestamps(commit: &str) -> Result<(String, String)> {
    let output = git_stdout(&["show", "-s", "--format=%aI%n%cI", commit])?;
    let mut lines = output.lines();
    let author_date = lines
        .next()
        .ok_or_else(|| Error {
            command: "show".to_string(),
            message: "missing author date in output".to_string(),
        })?
        .to_string();
    let committer_date = lines
        .next()
        .ok_or_else(|| Error {
            command: "show".to_string(),
            message: "missing committer date in output".to_string(),
        })?
        .to_string();
    Ok((author_date, committer_date))
}

/// Parse ISO8601 timestamp to comparable form (unix seconds).
/// Handles formats like "2026-01-02T21:10:36Z" or "2026-01-02T21:10:36+05:00".
/// Returns unix timestamp in seconds (UTC).
fn parse_timestamp(iso8601: &str) -> Result<i64> {
    // Parse the date-time part
    let (datetime_part, tz_part) = if iso8601.ends_with('Z') {
        (&iso8601[..iso8601.len() - 1], "+00:00")
    } else if let Some(pos) = iso8601.rfind(|c| c == '+' || c == '-') {
        if pos > 10 {
            // Make sure it's a timezone offset, not part of the date
            (&iso8601[..pos], &iso8601[pos..])
        } else {
            return Err(Error {
                command: "parse_timestamp".to_string(),
                message: format!("invalid timestamp format: {}", iso8601),
            });
        }
    } else {
        return Err(Error {
            command: "parse_timestamp".to_string(),
            message: format!("missing timezone in timestamp: {}", iso8601),
        });
    };

    // Parse datetime: "2026-01-02T21:10:36"
    let parts: Vec<&str> = datetime_part.split('T').collect();
    if parts.len() != 2 {
        return Err(Error {
            command: "parse_timestamp".to_string(),
            message: format!("invalid datetime format: {}", datetime_part),
        });
    }

    let date_parts: Vec<&str> = parts[0].split('-').collect();
    let time_parts: Vec<&str> = parts[1].split(':').collect();

    if date_parts.len() != 3 || time_parts.len() != 3 {
        return Err(Error {
            command: "parse_timestamp".to_string(),
            message: format!("invalid date/time components: {}", datetime_part),
        });
    }

    let year: i32 = date_parts[0].parse().map_err(|_| Error {
        command: "parse_timestamp".to_string(),
        message: format!("invalid year: {}", date_parts[0]),
    })?;
    let month: i32 = date_parts[1].parse().map_err(|_| Error {
        command: "parse_timestamp".to_string(),
        message: format!("invalid month: {}", date_parts[1]),
    })?;
    let day: i32 = date_parts[2].parse().map_err(|_| Error {
        command: "parse_timestamp".to_string(),
        message: format!("invalid day: {}", date_parts[2]),
    })?;
    let hour: i32 = time_parts[0].parse().map_err(|_| Error {
        command: "parse_timestamp".to_string(),
        message: format!("invalid hour: {}", time_parts[0]),
    })?;
    let minute: i32 = time_parts[1].parse().map_err(|_| Error {
        command: "parse_timestamp".to_string(),
        message: format!("invalid minute: {}", time_parts[1]),
    })?;
    let second: i32 = time_parts[2].parse().map_err(|_| Error {
        command: "parse_timestamp".to_string(),
        message: format!("invalid second: {}", time_parts[2]),
    })?;

    // Parse timezone offset: "+05:00" or "-06:24"
    let tz_sign = if tz_part.starts_with('+') { 1 } else { -1 };
    let tz_nums: Vec<&str> = tz_part[1..].split(':').collect();
    if tz_nums.len() != 2 {
        return Err(Error {
            command: "parse_timestamp".to_string(),
            message: format!("invalid timezone format: {}", tz_part),
        });
    }
    let tz_hours: i32 = tz_nums[0].parse().map_err(|_| Error {
        command: "parse_timestamp".to_string(),
        message: format!("invalid timezone hours: {}", tz_nums[0]),
    })?;
    let tz_minutes: i32 = tz_nums[1].parse().map_err(|_| Error {
        command: "parse_timestamp".to_string(),
        message: format!("invalid timezone minutes: {}", tz_nums[1]),
    })?;
    let tz_offset_seconds = tz_sign * (tz_hours * 3600 + tz_minutes * 60);

    // Simplified unix timestamp calculation (good enough for comparison)
    // Days since epoch (1970-01-01)
    let mut days = 0i64;

    // Add days for complete years
    for y in 1970..year {
        days += if is_leap_year(y) { 366 } else { 365 };
    }

    // Add days for complete months
    let days_in_month = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    for m in 1..month {
        days += days_in_month[(m - 1) as usize] as i64;
        if m == 2 && is_leap_year(year) {
            days += 1;
        }
    }

    // Add remaining days
    days += (day - 1) as i64;

    // Convert to seconds and add time
    let mut seconds = days * 86400 + hour as i64 * 3600 + minute as i64 * 60 + second as i64;

    // Adjust for timezone (subtract offset to get UTC)
    seconds -= tz_offset_seconds as i64;

    Ok(seconds)
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Find the latest timestamp among parent commits.
/// Checks author-date first, then commit-date for each parent in order.
/// Returns the timestamp string with its original timezone.
pub fn latest_parent_timestamp(parent_commits: &[&str]) -> Result<String> {
    let mut latest_seconds = 0i64;
    let mut latest_timestamp = String::new();

    for parent in parent_commits {
        let (author_date, commit_date) = get_commit_timestamps(parent)?;

        // Check author-date first
        let author_secs = parse_timestamp(&author_date)?;
        if author_secs > latest_seconds {
            latest_seconds = author_secs;
            latest_timestamp = author_date;
        }

        // Then check commit-date
        let commit_secs = parse_timestamp(&commit_date)?;
        if commit_secs > latest_seconds {
            latest_seconds = commit_secs;
            latest_timestamp = commit_date;
        }
    }

    Ok(latest_timestamp)
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
        let parent_list: Vec<String> = parts[1].split_whitespace().map(|s| s.to_string()).collect();
        let body = parts[2].to_string();
        results.push((hash, parent_list, body));
    }
    Ok(results)
}
#[cfg(test)]
mod tests {
    use {
        super::*,
        std::fs,
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
    fn test_empty_tree() {
        let _dir = setup_test_repo();
        let tree = empty_tree().unwrap();
        assert_eq!(tree, "4b825dc642cb6eb9a060e54bf8d69288fbee4904");
    }
    #[test]
    fn test_mktree_and_ls_tree() {
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
        let tree = rev_parse("HEAD^{tree}").unwrap();
        let entries = ls_tree(&tree).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].3, "test.txt");
        assert_eq!(entries[0].1, "blob");
    }
}
