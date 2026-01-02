use {
    super::{
        git_ops::{
            create_revert_commit,
            find_git_root,
            get_changed_files,
            restore_file_from_commit,
        },
        patterns::NoeditMatcher,
    },
    crate::{
        HookInput,
        HookOutput,
        HookOutputDetails,
    },
    eyre::{
        Context,
        Result,
    },
    git2::{
        Oid,
        Repository,
    },
    std::path::PathBuf,
};

/// Handle PostToolUse hook: detect and auto-revert .noedit violations
pub fn handle(input: &HookInput) -> Result<Option<HookOutput>> {
    // 1. Get JEB_CLAUDE_INITIAL_COMMIT from environment
    let initial_commit_sha = match std::env::var("JEB_CLAUDE_INITIAL_COMMIT") {
        Ok(sha) => {
            eprintln!("PostToolUse: Found JEB_CLAUDE_INITIAL_COMMIT={}", sha);
            sha
        }
        Err(_) => {
            eprintln!("PostToolUse: JEB_CLAUDE_INITIAL_COMMIT not set, skipping validation");
            // Return warning to user instead of silent fail-open
            return Ok(Some(HookOutput {
                should_continue: None,
                stop_reason: None,
                suppress_output: None,
                system_message: Some(
                    "⚠️ .noedit protection not active: JEB_CLAUDE_INITIAL_COMMIT not \
                     set.\nProtection will activate on next session start."
                        .to_string(),
                ),
                permission_decision: None,
                hook_specific_output: None,
            }));
        }
    };

    // 2. Open git repository
    let repo_path = find_git_root(&input.cwd).context("Failed to find git repository")?;
    let repo = Repository::open(&repo_path).context("Failed to open repository")?;

    // 3. Get initial commit object
    let initial_oid = Oid::from_str(&initial_commit_sha).context("Invalid commit SHA")?;
    let initial_commit = repo
        .find_commit(initial_oid)
        .context("Failed to find initial commit")?;

    // 4. Load .noedit patterns from initial commit
    let matcher = NoeditMatcher::from_commit(&repo, &initial_commit)
        .context("Failed to load .noedit patterns from initial commit")?;

    // 5. Get all changed files since initial commit
    let changed_files =
        get_changed_files(&repo, &initial_commit).context("Failed to get changed files")?;

    // 6. Filter to only .noedit-protected files
    let violated_files: Vec<PathBuf> = changed_files
        .into_iter()
        .filter(|path| matcher.matches(path))
        .collect();

    // 7. If no violations, pass through
    if violated_files.is_empty() {
        return Ok(None);
    }

    eprintln!(
        "PostToolUse: Found {} .noedit violations: {:?}",
        violated_files.len(),
        violated_files
    );

    // 8. Revert ALL violated files
    for path in &violated_files {
        restore_file_from_commit(&repo, &initial_commit, path)
            .with_context(|| format!("Failed to restore file: {}", path.display()))?;
    }

    // 9. Create commit if there are staged changes
    let index = repo.index().context("Failed to get index")?;
    if !index.is_empty() {
        create_revert_commit(
            &repo,
            &violated_files,
            &input.session_id,
            &input.transcript_path,
        )
        .context("Failed to create revert commit")?;
        eprintln!(
            "PostToolUse: Created revert commit for {} files",
            violated_files.len()
        );
    } else {
        eprintln!(
            "PostToolUse: Restored {} files (no commit needed)",
            violated_files.len()
        );
    }

    // 10. Return system message to agent
    Ok(Some(HookOutput {
        should_continue: None,
        stop_reason: None,
        suppress_output: None,
        system_message: Some(format!(
            "⚠️ Reverted {} .noedit-protected file(s): {}\n\nThese files are read-only per \
             .noedit patterns.",
            violated_files.len(),
            violated_files
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )),
        permission_decision: None,
        hook_specific_output: Some(HookOutputDetails::PostToolUse {
            additional_context: format!(
                "Automatically reverted {} protected files",
                violated_files.len()
            ),
        }),
    }))
}
