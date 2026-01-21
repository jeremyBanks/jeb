use anyhow::{Context, Result};
use std::collections::HashSet;
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

fn find_deleted_md_files() -> Result<Vec<String>> {
    let output = git(&[
        "log",
        "--all",
        "--diff-filter=D",
        "--name-only",
        "--format=",
        "--",
        "*.md",
    ])?;

    let files: HashSet<String> = output
        .lines()
        .filter(|line| line.ends_with(".md"))
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
    last_commit: String,
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
    let last_commit = git(&[
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

    Ok(FileInfo {
        path: path.to_string(),
        created,
        deleted,
        last_commit,
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
    let commit_short = &info.last_commit[..6.min(info.last_commit.len())];

    // Flatten path: replace / with -
    let flattened = info.path.replace('/', "-");

    if deleted_abbrev.is_empty() {
        format!("{}-{}-{}", created, commit_short, flattened)
    } else {
        format!("{}-{}-{}-{}", created, deleted_abbrev, commit_short, flattened)
    }
}

fn recover_content(commit: &str, path: &str) -> Result<String> {
    // Get content from parent of deletion commit (don't trim - preserve exact bytes)
    let spec = format!("{}^:{}", commit, path);
    let output = Command::new("git")
        .args(["show", &spec])
        .output()
        .context("failed to run git show")?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn main() -> Result<()> {
    let output_dir = Path::new("history-pit");
    fs::create_dir_all(output_dir)?;

    let files = find_deleted_md_files()?;
    println!("Found {} deleted markdown files", files.len());

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

        if info.created.is_empty() || info.deleted.is_empty() || info.last_commit.is_empty() {
            eprintln!("Missing metadata for {}", path);
            failed += 1;
            continue;
        }

        let content = match recover_content(&info.last_commit, path) {
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
