use {
    super::{
        git_ops::{
            create_restoration_commit,
            find_git_root,
            find_session_boundary,
            get_changed_files,
            restore_file_from_commit,
        },
        patterns::NoeditMatcher,
    },
    crate::{
        HookInput,
        HookOutput,
    },
    eyre::{
        Context,
        Result,
    },
    git2::Repository,
    std::path::Path,
};

/// Handle Stop/SessionEnd hook: validate and restore any violated .noedit files
pub fn handle(input: &HookInput) -> Result<Option<HookOutput>> {
    eprintln!("=== .noedit validation hook CALLED ===");
    eprintln!("Session ID: {}", input.session_id);

    // Find git repository root from current working directory
    let repo_path = find_git_root(&input.cwd).context("Failed to find git repository")?;
    eprintln!("Found git repo at: {}", repo_path.display());
    let repo = Repository::open(&repo_path).context("Failed to open repository")?;

    // 1. Find session boundary by scanning git history
    let initial_commit =
        find_session_boundary(&repo).context("Failed to find session boundary")?;

    eprintln!(
        "Validation: Found session boundary at commit {} ({})",
        initial_commit.id(),
        initial_commit.summary().unwrap_or("<no summary>")
    );

    // 3. Read .noedit files from initial commit
    let matcher = NoeditMatcher::from_commit(&repo, &initial_commit)
        .context("Failed to load .noedit patterns from initial commit")?;

    // 4. Get changed files (index and working tree)
    let changed_files =
        get_changed_files(&repo, &initial_commit).context("Failed to get changed files")?;

    // 5. Find violated files (changed AND in .noedit)
    let cwd = Path::new(&input.cwd);
    let mut violated_files = Vec::new();
    for file in &changed_files {
        if matcher.matches_path(cwd, file) {
            violated_files.push(file.clone());
        }
    }

    if violated_files.is_empty() {
        eprintln!("No .noedit violations found");
        return Ok(None); // No violations, pass through
    }

    eprintln!(
        "Found {} .noedit violations, restoring...",
        violated_files.len()
    );

    // 6. Restore violated files to their state in initial_commit
    for file in &violated_files {
        restore_file_from_commit(&repo, &initial_commit, file)
            .with_context(|| format!("Failed to restore file: {}", file.display()))?;
    }

    // 7. Create commit documenting the restoration
    create_restoration_commit(&repo, &violated_files, &initial_commit.id().to_string())
        .context("Failed to create restoration commit")?;

    // 8. Return system message to inform user
    let file_list = violated_files
        .iter()
        .map(|f| f.display().to_string())
        .collect::<Vec<_>>()
        .join(", ");

    Ok(Some(HookOutput {
        should_continue: None,
        stop_reason: None,
        suppress_output: None,
        system_message: Some(format!(
            "Restored {} .noedit-protected file(s) that were modified: {}",
            violated_files.len(),
            file_list
        )),
        permission_decision: None,
        hook_specific_output: None,
    }))
}
