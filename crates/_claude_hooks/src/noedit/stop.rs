use {
    super::{
        git_ops::{
            create_restoration_commit,
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
        bail,
    },
    git2::{
        Oid,
        Repository,
    },
    std::path::Path,
};

/// Handle Stop hook: validate and restore any violated .noedit files
pub fn handle(input: &HookInput) -> Result<Option<HookOutput>> {
    let repo = Repository::open(&input.cwd).context("Failed to open repository")?;

    // 1. Validate JEB_CLAUDE_INITIAL_COMMIT exists
    let initial_commit_sha = match std::env::var("JEB_CLAUDE_INITIAL_COMMIT") {
        Ok(sha) => sha,
        Err(_) => {
            eprintln!("JEB_CLAUDE_INITIAL_COMMIT not set, skipping .noedit validation");
            return Ok(None);
        }
    };

    // 2. Parse commit and verify it's an ancestor of HEAD
    let initial_oid = Oid::from_str(&initial_commit_sha)
        .context("Failed to parse JEB_CLAUDE_INITIAL_COMMIT as git OID")?;
    let initial_commit = repo
        .find_commit(initial_oid)
        .context("Failed to find initial commit")?;

    let head = repo.head().context("Failed to get HEAD")?;
    let head_commit = head
        .peel_to_commit()
        .context("Failed to peel HEAD to commit")?;

    // Check if initial_commit is ancestor of HEAD
    if !repo
        .graph_descendant_of(head_commit.id(), initial_oid)
        .context("Failed to check ancestry")?
    {
        bail!(
            "HEAD is not a descendant of JEB_CLAUDE_INITIAL_COMMIT ({}). History may have been \
             rewritten.",
            initial_commit_sha
        );
    }

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
    create_restoration_commit(&repo, &violated_files, &initial_commit_sha)
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
