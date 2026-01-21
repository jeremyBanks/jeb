use anyhow::{Context, Result};
use std::collections::{HashMap, HashSet};
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

fn git_raw(args: &[&str]) -> Result<Vec<u8>> {
    let output = Command::new("git")
        .args(args)
        .output()
        .context("failed to run git")?;
    if !output.status.success() {
        anyhow::bail!(
            "git failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(output.stdout)
}

#[derive(Debug, Clone)]
struct BlobDeletion {
    blob_hash: String,
    path: String,
    created: String,      // YYYY-MM-DD
    deleted: String,      // YYYY-MM-DD
    delete_commit: String,
}

/// Get all blob hashes currently in HEAD
fn get_head_blobs() -> Result<HashSet<String>> {
    let output = git(&["ls-tree", "-r", "HEAD"])?;
    let mut blobs = HashSet::new();
    for line in output.lines() {
        // Format: mode type hash\tpath
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 && parts[1] == "blob" {
            blobs.insert(parts[2].to_string());
        }
    }
    Ok(blobs)
}

/// Find all commits that deleted files matching the pattern
fn find_deletion_commits(pattern: &str) -> Result<Vec<(String, String)>> {
    // Returns (commit_hash, date) pairs
    let output = git(&[
        "log",
        "--all",
        "--diff-filter=D",
        "--format=%H %cs",
        "--",
        pattern,
    ])?;

    let mut commits = Vec::new();
    for line in output.lines() {
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.splitn(2, ' ').collect();
        if parts.len() == 2 {
            commits.push((parts[0].to_string(), parts[1].to_string()));
        }
    }
    Ok(commits)
}

/// Get the blobs deleted in a specific commit compared to its first parent
fn get_deleted_blobs(commit: &str, pattern: &str) -> Result<Vec<(String, String)>> {
    // Returns (blob_hash, path) pairs for deleted regular files

    // Get parent commit
    let parent = git(&["rev-parse", &format!("{}^", commit)]);
    let parent = match parent {
        Ok(p) if !p.is_empty() => p,
        _ => return Ok(Vec::new()), // Root commit, no deletions possible
    };

    // Use diff-tree to find deleted files
    // Format: :old_mode new_mode old_hash new_hash status\tpath
    let output = git(&[
        "diff-tree",
        "-r",
        "--diff-filter=D",
        &parent,
        commit,
        "--",
        pattern,
    ])?;

    let mut deletions = Vec::new();
    for line in output.lines() {
        if line.is_empty() || !line.starts_with(':') {
            continue;
        }

        // Parse the diff-tree output
        // Example: :100644 000000 abc123... 0000000... D\tpath/to/file.md
        let parts: Vec<&str> = line[1..].splitn(5, |c| c == ' ' || c == '\t').collect();
        if parts.len() < 5 {
            continue;
        }

        let old_mode = parts[0];
        let old_hash = parts[2];

        // Only consider regular files (mode 100644 or 100755)
        if !old_mode.starts_with("100") {
            continue;
        }

        // Find the path (after the tab)
        if let Some(tab_pos) = line.find('\t') {
            let path = &line[tab_pos + 1..];
            // Skip history-pit output directory
            if path.starts_with("history-pit/") {
                continue;
            }
            deletions.push((old_hash.to_string(), path.to_string()));
        }
    }

    Ok(deletions)
}

/// Also detect when a regular file becomes a symlink or submodule (content is "deleted")
fn get_mode_changes(commit: &str, pattern: &str) -> Result<Vec<(String, String)>> {
    // Returns (blob_hash, path) pairs where file went from regular to symlink/submodule

    let parent = git(&["rev-parse", &format!("{}^", commit)]);
    let parent = match parent {
        Ok(p) if !p.is_empty() => p,
        _ => return Ok(Vec::new()),
    };

    // Use diff-tree with --diff-filter=T for type changes
    let output = git(&[
        "diff-tree",
        "-r",
        "--diff-filter=T",
        &parent,
        commit,
        "--",
        pattern,
    ])?;

    let mut deletions = Vec::new();
    for line in output.lines() {
        if line.is_empty() || !line.starts_with(':') {
            continue;
        }

        let parts: Vec<&str> = line[1..].splitn(5, |c| c == ' ' || c == '\t').collect();
        if parts.len() < 5 {
            continue;
        }

        let old_mode = parts[0];
        let new_mode = parts[1];
        let old_hash = parts[2];

        // Only if it was a regular file and became something else
        if old_mode.starts_with("100") && !new_mode.starts_with("100") {
            if let Some(tab_pos) = line.find('\t') {
                let path = &line[tab_pos + 1..];
                if !path.starts_with("history-pit/") {
                    deletions.push((old_hash.to_string(), path.to_string()));
                }
            }
        }
    }

    Ok(deletions)
}

/// Find the creation date of a blob at a specific path
fn find_creation_date(blob_hash: &str, path: &str, before_commit: &str) -> Result<String> {
    // Walk back through history to find the earliest commit where this blob existed at this path
    let output = git(&[
        "log",
        "--format=%H %cs",
        "--follow",
        before_commit,
        "--",
        path,
    ])?;

    let mut creation_date = String::new();

    for line in output.lines() {
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.splitn(2, ' ').collect();
        if parts.len() != 2 {
            continue;
        }
        let commit = parts[0];
        let date = parts[1];

        // Check if this commit has the blob at the path
        let ls_tree = git(&["ls-tree", commit, "--", path])?;
        if ls_tree.is_empty() {
            continue;
        }

        // Check the hash matches
        let tree_parts: Vec<&str> = ls_tree.split_whitespace().collect();
        if tree_parts.len() >= 3 && tree_parts[2] == blob_hash {
            creation_date = date.to_string();
        }
    }

    // If we couldn't trace back, use the deletion date as creation date
    if creation_date.is_empty() {
        // Try a simpler approach: just get the oldest commit touching this file
        let oldest = git(&[
            "log",
            "--format=%cs",
            "--follow",
            before_commit,
            "--",
            path,
        ])?;
        creation_date = oldest.lines().last().unwrap_or("").to_string();
    }

    Ok(creation_date)
}

fn abbreviate_date(created: &str, deleted: &str) -> String {
    // Dates are in YYYY-MM-DD format, convert to YYYYMMDD
    let created_compact = created.replace('-', "");
    let deleted_compact = deleted.replace('-', "");

    if created_compact.len() < 8 || deleted_compact.len() < 8 {
        return deleted_compact;
    }

    if created_compact == deleted_compact {
        // Same day - omit second date entirely
        return String::new();
    }

    let c_year = &created_compact[0..4];
    let c_month = &created_compact[4..6];

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

fn build_filename(deletion: &BlobDeletion) -> String {
    let created = deletion.created.replace('-', "");
    let deleted_abbrev = abbreviate_date(&deletion.created, &deletion.deleted);
    let commit_short = &deletion.delete_commit[..6.min(deletion.delete_commit.len())];

    // Flatten path: replace / with -
    let flattened = deletion.path.replace('/', "-");

    if deleted_abbrev.is_empty() {
        format!("{}-{}-{}", created, commit_short, flattened)
    } else {
        format!("{}-{}-{}-{}", created, deleted_abbrev, commit_short, flattened)
    }
}

fn recover_blob_content(blob_hash: &str) -> Result<Vec<u8>> {
    git_raw(&["cat-file", "blob", blob_hash])
}

fn main() -> Result<()> {
    let pattern = env::args().nth(1).unwrap_or_else(|| "*.md".to_string());

    let output_dir = Path::new("history-pit");
    fs::create_dir_all(output_dir)?;

    // Step 1: Get all blobs in HEAD (these are not lost)
    println!("Getting blobs in HEAD...");
    let head_blobs = get_head_blobs()?;
    println!("  {} blobs currently in HEAD", head_blobs.len());

    // Step 2: Find all deletion commits
    println!("Finding deletion commits...");
    let deletion_commits = find_deletion_commits(&pattern)?;
    println!("  {} commits with deletions", deletion_commits.len());

    // Step 3: Collect all blob deletions
    println!("Collecting blob deletions...");
    let mut all_deletions: Vec<BlobDeletion> = Vec::new();

    for (commit, date) in &deletion_commits {
        // Get actual file deletions
        let deleted = get_deleted_blobs(commit, &pattern)?;
        for (blob_hash, path) in deleted {
            all_deletions.push(BlobDeletion {
                blob_hash,
                path,
                created: String::new(), // Will fill in later
                deleted: date.clone(),
                delete_commit: commit.clone(),
            });
        }

        // Get mode changes (regular file -> symlink/submodule)
        let changed = get_mode_changes(commit, &pattern)?;
        for (blob_hash, path) in changed {
            all_deletions.push(BlobDeletion {
                blob_hash,
                path,
                created: String::new(),
                deleted: date.clone(),
                delete_commit: commit.clone(),
            });
        }
    }
    println!("  {} total blob deletions found", all_deletions.len());

    // Step 4: Filter out blobs that still exist in HEAD
    println!("Filtering out blobs still in HEAD...");
    let lost_deletions: Vec<_> = all_deletions
        .into_iter()
        .filter(|d| !head_blobs.contains(&d.blob_hash))
        .collect();
    println!("  {} truly lost blobs", lost_deletions.len());

    // Step 5: Deduplicate by (blob_hash, path) - keep most recent deletion
    println!("Deduplicating...");
    let mut dedup_map: HashMap<(String, String), BlobDeletion> = HashMap::new();
    for deletion in lost_deletions {
        let key = (deletion.blob_hash.clone(), deletion.path.clone());
        dedup_map
            .entry(key)
            .and_modify(|existing| {
                // Keep the most recent deletion date
                if deletion.deleted > existing.deleted {
                    *existing = deletion.clone();
                }
            })
            .or_insert(deletion);
    }
    let mut unique_deletions: Vec<_> = dedup_map.into_values().collect();
    unique_deletions.sort_by(|a, b| (&a.path, &a.deleted).cmp(&(&b.path, &b.deleted)));
    println!("  {} unique (blob, path) pairs", unique_deletions.len());

    // Step 6: Fill in creation dates and recover content
    println!("Recovering files...");
    let mut recovered = 0;
    let mut failed = 0;

    for deletion in &mut unique_deletions {
        // Find creation date
        let created = find_creation_date(
            &deletion.blob_hash,
            &deletion.path,
            &format!("{}^", deletion.delete_commit),
        )?;
        deletion.created = if created.is_empty() {
            deletion.deleted.clone()
        } else {
            created
        };

        // Recover content
        let content = match recover_blob_content(&deletion.blob_hash) {
            Ok(c) => c,
            Err(e) => {
                eprintln!(
                    "Failed to recover blob {} for {}: {}",
                    &deletion.blob_hash[..7.min(deletion.blob_hash.len())],
                    deletion.path,
                    e
                );
                failed += 1;
                continue;
            }
        };

        if content.is_empty() {
            eprintln!("Empty content for {} (blob {})", deletion.path, &deletion.blob_hash[..7.min(deletion.blob_hash.len())]);
            failed += 1;
            continue;
        }

        let filename = build_filename(deletion);
        let output_path = output_dir.join(&filename);

        fs::write(&output_path, &content)?;
        println!("  {} -> {}", deletion.path, filename);
        recovered += 1;
    }

    println!("\nDone: {} recovered, {} failed", recovered, failed);
    Ok(())
}
