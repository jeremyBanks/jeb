use {
    eyre::{
        Context,
        ContextCompat,
        Result,
        bail,
    },
    git2::{
        Commit,
        DiffOptions,
        ObjectType,
        Repository,
        TreeWalkMode,
        TreeWalkResult,
    },
    std::{
        collections::HashMap,
        path::{
            Path,
            PathBuf,
        },
    },
};

/// Find git repository root by walking up from a starting path
pub fn find_git_root(start_path: &str) -> Result<PathBuf> {
    let mut current = PathBuf::from(start_path);

    loop {
        if current.join(".git").exists() {
            return Ok(current);
        }

        if !current.pop() {
            bail!("Not in a git repository (no .git directory found)");
        }
    }
}

/// Represents the content of a .noedit file found in a git tree
pub struct NoeditFile {
    pub path: PathBuf,
    pub content: String,
}

/// Read all .noedit files from a specific commit
pub fn read_noedit_files_from_commit(
    repo: &Repository,
    commit: &Commit,
) -> Result<Vec<NoeditFile>> {
    let tree = commit.tree().context("Failed to get tree from commit")?;
    let mut noedit_files = Vec::new();

    tree.walk(TreeWalkMode::PreOrder, |root, entry| {
        if entry.name() == Some(".noedit") && entry.kind() == Some(ObjectType::Blob) {
            let oid = entry.id();
            if let Ok(blob) = repo.find_blob(oid) {
                let content = blob.content();
                let content_str = String::from_utf8_lossy(content).to_string();

                let path = if root.is_empty() {
                    PathBuf::from(".noedit")
                } else {
                    PathBuf::from(root).join(".noedit")
                };

                noedit_files.push(NoeditFile {
                    path,
                    content: content_str,
                });
            }
        }
        TreeWalkResult::Ok
    })
    .context("Failed to walk tree")?;

    Ok(noedit_files)
}

/// Get list of files changed between a base commit and current state (index +
/// working tree)
pub fn get_changed_files(repo: &Repository, base_commit: &Commit) -> Result<Vec<PathBuf>> {
    let base_tree = base_commit.tree().context("Failed to get base tree")?;
    let mut changed = HashMap::new();

    // Get changes in index (staged)
    let mut index = repo.index().context("Failed to get index")?;
    let index_tree_oid = index.write_tree().context("Failed to write index tree")?;
    let index_tree = repo
        .find_tree(index_tree_oid)
        .context("Failed to find index tree")?;

    let diff = repo
        .diff_tree_to_tree(Some(&base_tree), Some(&index_tree), None)
        .context("Failed to diff base tree to index")?;

    diff.foreach(
        &mut |delta, _| {
            if let Some(path) = delta.new_file().path() {
                changed.insert(path.to_path_buf(), ());
            }
            true
        },
        None,
        None,
        None,
    )
    .context("Failed to iterate diff deltas")?;

    // Get changes in working tree (unstaged)
    let mut opts = DiffOptions::new();
    opts.include_untracked(true);

    let diff = repo
        .diff_tree_to_workdir_with_index(Some(&base_tree), Some(&mut opts))
        .context("Failed to diff base tree to workdir")?;

    diff.foreach(
        &mut |delta, _| {
            if let Some(path) = delta.new_file().path() {
                changed.insert(path.to_path_buf(), ());
            }
            true
        },
        None,
        None,
        None,
    )
    .context("Failed to iterate workdir diff deltas")?;

    Ok(changed.into_keys().collect())
}

/// Restore a file to its state in a specific commit, or delete it if it didn't
/// exist
pub fn restore_file_from_commit(repo: &Repository, commit: &Commit, path: &Path) -> Result<()> {
    let tree = commit.tree().context("Failed to get tree from commit")?;

    match tree.get_path(path) {
        Ok(entry) => {
            // File existed in base commit, restore it
            let oid = entry.id();
            let blob = repo.find_blob(oid).context("Failed to find blob")?;

            let workdir = repo
                .workdir()
                .context("Repository has no working directory")?;
            let full_path = workdir.join(path);

            if let Some(parent) = full_path.parent() {
                std::fs::create_dir_all(parent).context("Failed to create parent directories")?;
            }

            std::fs::write(&full_path, blob.content()).context("Failed to write file content")?;

            // Stage the restored file
            let mut index = repo.index().context("Failed to get index")?;
            index
                .add_path(path)
                .context("Failed to add path to index")?;
            index.write().context("Failed to write index")?;
        }
        Err(_) => {
            // File didn't exist in base commit, delete it
            delete_file(repo, path)?;
        }
    }

    Ok(())
}

/// Delete a file from both working directory and index
pub fn delete_file(repo: &Repository, path: &Path) -> Result<()> {
    let workdir = repo
        .workdir()
        .context("Repository has no working directory")?;
    let full_path = workdir.join(path);

    if full_path.exists() {
        std::fs::remove_file(&full_path).context("Failed to remove file from working directory")?;
    }

    // Remove from index
    let mut index = repo.index().context("Failed to get index")?;
    index
        .remove_path(path)
        .context("Failed to remove path from index")?;
    index.write().context("Failed to write index")?;

    Ok(())
}

/// Create a commit documenting file restorations
pub fn create_restoration_commit(
    repo: &Repository,
    files: &[PathBuf],
    initial_commit_sha: &str,
) -> Result<()> {
    let signature = repo.signature().context("Failed to get signature")?;

    let message = format!(
        "Restore .noedit-protected files\n\nThe following files were modified but are protected \
         by .noedit patterns\nfrom commit {}:\n\n{}\n\nThese files have been restored to their \
         original state.",
        initial_commit_sha,
        files
            .iter()
            .map(|f| format!("  - {}", f.display()))
            .collect::<Vec<_>>()
            .join("\n")
    );

    let mut index = repo.index().context("Failed to get index")?;
    let tree_oid = index.write_tree().context("Failed to write tree")?;
    let tree = repo.find_tree(tree_oid).context("Failed to find tree")?;

    let head = repo.head().context("Failed to get HEAD")?;
    let parent_commit = head
        .peel_to_commit()
        .context("Failed to peel HEAD to commit")?;

    repo.commit(Some("HEAD"), &signature, &signature, &message, &tree, &[
        &parent_commit,
    ])
    .context("Failed to create commit")?;

    Ok(())
}

/// Create a commit reverting .noedit violations
pub fn create_revert_commit(
    repo: &Repository,
    _files: &[PathBuf],
    session_id: &str,
    transcript_path: &str,
) -> Result<()> {
    let signature = repo.signature().context("Failed to get signature")?;

    // Message format per user requirements
    let message = format!(
        "revert changes disallowed by .noedit\n\nSession-Id: {}\nTranscript: {}",
        session_id, transcript_path
    );

    let mut index = repo.index().context("Failed to get index")?;
    let tree_oid = index.write_tree().context("Failed to write tree")?;
    let tree = repo.find_tree(tree_oid).context("Failed to find tree")?;

    let head = repo.head().context("Failed to get HEAD")?;
    let parent_commit = head
        .peel_to_commit()
        .context("Failed to peel HEAD to commit")?;

    repo.commit(Some("HEAD"), &signature, &signature, &message, &tree, &[
        &parent_commit,
    ])
    .context("Failed to create commit")?;

    Ok(())
}

/// Find the commit where the current Claude session started by walking
/// first-parent history backward until we find a commit that's NOT from Claude.
pub fn find_session_boundary(repo: &Repository) -> Result<Commit<'_>> {
    let mut commit = repo
        .head()
        .context("Failed to get HEAD")?
        .peel_to_commit()
        .context("Failed to peel HEAD to commit")?;

    loop {
        // Check if this commit is from Claude
        if !is_claude_commit(&commit)? {
            // Found the boundary - this is the last non-Claude commit
            return Ok(commit);
        }

        // Walk to first parent
        let parents: Vec<_> = commit.parents().collect();
        if parents.is_empty() {
            // Reached initial commit, it's the boundary
            return Ok(commit);
        }

        commit = parents[0].clone();
    }
}

/// Check if a commit was made by Claude
fn is_claude_commit(commit: &Commit) -> Result<bool> {
    let message = commit.message().unwrap_or("");

    // Check for Session-Id trailer
    if message.contains("Session-Id:") {
        return Ok(true);
    }

    // Check for Co-Authored-By trailer with noreply@anthropic.com
    // Check both capitalizations: Co-Authored-By and Co-authored-by
    for line in message.lines() {
        let line_lower = line.to_lowercase();
        if line_lower.starts_with("co-authored-by:") && line.contains("noreply@anthropic.com") {
            return Ok(true);
        }
    }

    // Check author email
    if let Some(email) = commit.author().email() {
        if email.contains("noreply@anthropic.com") {
            return Ok(true);
        }
    }

    // Check committer email
    if let Some(email) = commit.committer().email() {
        if email.contains("noreply@anthropic.com") {
            return Ok(true);
        }
    }

    Ok(false)
}
