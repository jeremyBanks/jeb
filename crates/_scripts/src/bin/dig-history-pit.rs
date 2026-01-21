use anyhow::{Context, Result};
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn git(args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .output()
        .context("failed to run git")?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn find_deleted_files(pattern: &str) -> Result<Vec<String>> {
    let output = git(&[
        "log",
        "--all",
        "--diff-filter=D",
        "--name-only",
        "--format=",
        "--",
        pattern,
    ])?;

    let files: HashSet<String> = output
        .lines()
        .filter(|line| !line.is_empty())
        // Exclude files in history-pit/ (our output directory)
        .filter(|line| !line.starts_with("history-pit/"))
        .map(|s| s.to_string())
        .collect();

    let mut files: Vec<_> = files.into_iter().collect();
    files.sort();
    Ok(files)
}

struct FileInfo {
    path: String,
    created: String,
    deleted: String,
    delete_commit: String,
    /// The last commit where the file actually existed (for content recovery)
    last_good_commit: String,
}

fn get_file_info(path: &str) -> Result<FileInfo> {
    // Get earliest date (using -m for merge commits)
    let created = git(&["log", "--all", "-m", "--format=%cs", "--", path])?
        .lines()
        .last()
        .unwrap_or("")
        .to_string();

    // Get deletion date
    let deleted = git(&[
        "log",
        "--all",
        "-m",
        "--diff-filter=D",
        "--format=%cs",
        "--",
        path,
    ])?
    .lines()
    .next()
    .unwrap_or("")
    .to_string();

    // Get commit hash of deletion
    let delete_commit = git(&[
        "log",
        "--all",
        "-m",
        "--diff-filter=D",
        "--format=%H",
        "--",
        path,
    ])?
    .lines()
    .next()
    .unwrap_or("")
    .to_string();

    // Get the last commit where the file was a regular blob (not symlink/submodule/tree)
    // Mode 100644 or 100755 = regular file, 120000 = symlink, 160000 = submodule
    let commits: Vec<_> = git(&[
        "log",
        "--all",
        "-m",
        "--diff-filter=ACMR",
        "--format=%H",
        "--",
        path,
    ])?
    .lines()
    .map(|s| s.to_string())
    .collect();

    let mut last_good_commit = String::new();
    for commit in commits {
        let ls_tree = git(&["ls-tree", &commit, "--", path])?;
        if let Some(mode) = ls_tree.split_whitespace().next() {
            // Regular files have mode 100644 or 100755
            if mode.starts_with("100") {
                last_good_commit = commit;
                break;
            }
        }
    }

    Ok(FileInfo {
        path: path.to_string(),
        created,
        deleted,
        delete_commit,
        last_good_commit,
    })
}

fn abbreviate_date(created: &str, deleted: &str) -> String {
    // Dates are in YYYY-MM-DD format, convert to YYYYMMDD
    let created_compact = created.replace('-', "");
    let deleted_compact = deleted.replace('-', "");

    if created_compact == deleted_compact {
        // Same day - omit second date entirely
        return String::new();
    }

    let c_year = &created_compact[0..4];
    let c_month = &created_compact[4..6];
    let _c_day = &created_compact[6..8];

    let d_year = &deleted_compact[0..4];
    let d_month = &deleted_compact[4..6];
    let d_day = &deleted_compact[6..8];

    if c_year != d_year {
        // Year differs - show full date
        deleted_compact
    } else if c_month != d_month {
        // Month differs - show MMDD
        format!("{}{}", d_month, d_day)
    } else {
        // Only day differs - show DD
        d_day.to_string()
    }
}

fn build_filename(info: &FileInfo) -> String {
    let created = info.created.replace('-', "");
    let deleted_abbrev = abbreviate_date(&info.created, &info.deleted);
    let commit_short = &info.delete_commit[..6.min(info.delete_commit.len())];

    // Flatten path: replace / with -
    let flattened = info.path.replace('/', "-");

    if deleted_abbrev.is_empty() {
        format!("{}-{}-{}", created, commit_short, flattened)
    } else {
        format!("{}-{}-{}-{}", created, deleted_abbrev, commit_short, flattened)
    }
}

fn recover_content(commit: &str, path: &str) -> Result<String> {
    // Get content from the commit where file last existed (don't trim - preserve exact bytes)
    let spec = format!("{}:{}", commit, path);
    let output = Command::new("git")
        .args(["show", &spec])
        .output()
        .context("failed to run git show")?;

    if !output.status.success() {
        anyhow::bail!(
            "git show failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let content = String::from_utf8_lossy(&output.stdout).to_string();
    if content.is_empty() {
        anyhow::bail!("recovered content is empty");
    }

    Ok(content)
}

fn main() -> Result<()> {
    let pattern = env::args().nth(1).unwrap_or_else(|| "*.md".to_string());

    let output_dir = Path::new("history-pit");
    fs::create_dir_all(output_dir)?;

    let files = find_deleted_files(&pattern)?;
    println!("Found {} deleted files matching '{}'", files.len(), pattern);

    let mut recovered = 0;
    let mut failed = 0;

    for path in &files {
        let info = match get_file_info(path) {
            Ok(info) => info,
            Err(e) => {
                eprintln!("Failed to get info for {}: {}", path, e);
                failed += 1;
                continue;
            }
        };

        if info.created.is_empty()
            || info.deleted.is_empty()
            || info.delete_commit.is_empty()
            || info.last_good_commit.is_empty()
        {
            eprintln!("Missing metadata for {}", path);
            failed += 1;
            continue;
        }

        let content = match recover_content(&info.last_good_commit, path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to recover content for {}: {}", path, e);
                failed += 1;
                continue;
            }
        };

        let filename = build_filename(&info);
        let output_path = output_dir.join(&filename);

        fs::write(&output_path, &content)?;
        println!("Recovered: {}", filename);
        recovered += 1;
    }

    println!("\nDone: {} recovered, {} failed", recovered, failed);
    Ok(())
}
