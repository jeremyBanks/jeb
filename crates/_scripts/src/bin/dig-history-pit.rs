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

/// Convert a glob pattern to a safe directory name
fn glob_to_dirname(pattern: &str) -> String {
    pattern
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Find all blob deletions using git log --raw
/// This handles merge commits correctly by using -m flag
fn find_all_deletions(pattern: &str) -> Result<Vec<BlobDeletion>> {
    // Use git log with --raw to get both commit info and diff info in one pass
    // -m shows merge commits as separate diffs against each parent
    // --diff-filter=D shows only deleted files
    // --diff-filter=T shows type changes (file -> symlink/submodule)
    let output = git(&[
        "log",
        "--all",
        "-m",
        "--raw",
        "--abbrev=40", // Full blob hashes for comparison with HEAD
        "--diff-filter=DT",
        "--format=COMMIT %H %cs",
        "--",
        pattern,
    ])?;

    let mut deletions = Vec::new();
    let mut current_commit = String::new();
    let mut current_date = String::new();

    for line in output.lines() {
        if line.starts_with("COMMIT ") {
            // Parse: COMMIT <hash> <date>
            let parts: Vec<&str> = line[7..].splitn(2, ' ').collect();
            if parts.len() == 2 {
                current_commit = parts[0].to_string();
                current_date = parts[1].to_string();
            }
            continue;
        }

        if line.starts_with(':') && !current_commit.is_empty() {
            // Raw diff line: :old_mode new_mode old_hash new_hash status\tpath
            // Example: :100644 000000 abc123... 0000000... D\tpath/to/file.md
            // Example: :100644 120000 abc123... def456... T\tpath/to/file.md (type change)

            let parts: Vec<&str> = line[1..].split_whitespace().collect();
            if parts.len() < 5 {
                continue;
            }

            let old_mode = parts[0];
            let new_mode = parts[1];
            let old_hash = parts[2];
            let status = parts[4];

            // Only consider regular files (mode 100644 or 100755)
            if !old_mode.starts_with("100") {
                continue;
            }

            // For type changes (T), only count if it became non-regular file
            if status.starts_with('T') && new_mode.starts_with("100") {
                continue;
            }

            // Find the path (after the tab in status field)
            let status_and_path = parts[4..].join(" ");
            let path = if let Some(tab_pos) = status_and_path.find('\t') {
                &status_and_path[tab_pos + 1..]
            } else if status_and_path.len() > 1 {
                // No tab, path might be after status letter
                status_and_path[1..].trim()
            } else {
                continue;
            };

            // Skip history-pit output directory
            if path.starts_with("history-pit/") || path.starts_with("history-pit\\") {
                continue;
            }

            deletions.push(BlobDeletion {
                blob_hash: old_hash.to_string(),
                path: path.to_string(),
                created: String::new(),
                deleted: current_date.clone(),
                delete_commit: current_commit.clone(),
            });
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
    let blob_short = &deletion.blob_hash[..8.min(deletion.blob_hash.len())];

    // Flatten path: replace / with -
    let flattened = deletion.path.replace('/', "-");

    // Concatenate created+abbrev directly (no dash between them)
    let date_part = format!("{}{}", created, deleted_abbrev);
    format!("{}-{}-{}-{}", date_part, commit_short, blob_short, flattened)
}

fn recover_blob_content(blob_hash: &str) -> Result<Vec<u8>> {
    git_raw(&["cat-file", "blob", blob_hash])
}

fn is_whitespace_only(content: &[u8]) -> bool {
    content.iter().all(|&b| b.is_ascii_whitespace())
}

fn main() -> Result<()> {
    let pattern = env::args().nth(1).unwrap_or_else(|| "*.md".to_string());

    let subdir_name = glob_to_dirname(&pattern);
    let output_dir = Path::new("history-pit").join(&subdir_name);
    fs::create_dir_all(&output_dir)?;

    // Step 1: Get all blobs in HEAD (these are not lost)
    println!("Getting blobs in HEAD...");
    let head_blobs = get_head_blobs()?;
    println!("  {} blobs currently in HEAD", head_blobs.len());

    // Step 2: Find all blob deletions (handles merge commits with -m)
    println!("Finding blob deletions...");
    let all_deletions = find_all_deletions(&pattern)?;
    println!("  {} total blob deletions found", all_deletions.len());

    // Step 3: Filter out blobs that still exist in HEAD
    println!("Filtering out blobs still in HEAD...");
    let lost_deletions: Vec<_> = all_deletions
        .into_iter()
        .filter(|d| !head_blobs.contains(&d.blob_hash))
        .collect();
    println!("  {} truly lost blobs", lost_deletions.len());

    // Step 4: Deduplicate by (blob_hash, path) - keep oldest AND newest deletion
    println!("Deduplicating...");
    let mut by_blob_path: HashMap<(String, String), Vec<BlobDeletion>> = HashMap::new();
    for deletion in lost_deletions {
        let key = (deletion.blob_hash.clone(), deletion.path.clone());
        by_blob_path.entry(key).or_default().push(deletion);
    }

    let mut unique_deletions = Vec::new();
    for (_key, mut deletions) in by_blob_path {
        deletions.sort_by(|a, b| a.deleted.cmp(&b.deleted)); // Sort by date
        if deletions.len() == 1 {
            unique_deletions.push(deletions.remove(0));
        } else {
            // Keep oldest and newest
            let oldest = deletions.remove(0);
            let newest = deletions.pop().unwrap();
            unique_deletions.push(oldest.clone());
            if oldest.delete_commit != newest.delete_commit {
                unique_deletions.push(newest);
            }
        }
    }
    unique_deletions.sort_by(|a, b| (&a.path, &a.deleted).cmp(&(&b.path, &b.deleted)));
    println!("  {} unique (blob, path) pairs", unique_deletions.len());

    // Step 4.5: Filter out whitespace-only blobs unless they're the only version for that path
    println!("Filtering whitespace-only content...");
    let mut deletions_by_path: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, d) in unique_deletions.iter().enumerate() {
        deletions_by_path.entry(d.path.clone()).or_default().push(i);
    }

    let mut skip_indices: HashSet<usize> = HashSet::new();
    for (_path, indices) in &deletions_by_path {
        // Find which indices have non-whitespace content
        let non_whitespace: Vec<usize> = indices
            .iter()
            .copied()
            .filter(|&i| {
                let content = recover_blob_content(&unique_deletions[i].blob_hash).unwrap_or_default();
                !is_whitespace_only(&content)
            })
            .collect();

        if !non_whitespace.is_empty() {
            // Skip whitespace-only versions since non-empty ones exist
            for &i in indices {
                let content = recover_blob_content(&unique_deletions[i].blob_hash).unwrap_or_default();
                if is_whitespace_only(&content) {
                    skip_indices.insert(i);
                }
            }
        }
    }

    let mut unique_deletions: Vec<_> = unique_deletions
        .into_iter()
        .enumerate()
        .filter(|(i, _)| !skip_indices.contains(i))
        .map(|(_, d)| d)
        .collect();
    println!("  {} after whitespace filtering", unique_deletions.len());

    // Step 5: Fill in creation dates and recover content
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
