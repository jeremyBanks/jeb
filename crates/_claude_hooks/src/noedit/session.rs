use {
    super::git_ops::find_git_root,
    crate::HookInput,
    eyre::{
        Context,
        Result,
    },
    git2::Repository,
    std::io::Write,
};

/// Handle SessionStart hook: initialize or preserve JEB_CLAUDE_INITIAL_COMMIT
pub fn handle(input: &HookInput) -> Result<()> {
    // Get CLAUDE_ENV_FILE path from environment
    let claude_env_file = match std::env::var("CLAUDE_ENV_FILE") {
        Ok(path) => {
            eprintln!("Found CLAUDE_ENV_FILE={}", path);
            path
        }
        Err(_) => {
            eprintln!("⚠ CLAUDE_ENV_FILE not set, skipping .noedit session initialization");
            eprintln!("  This is normal if not running in a SessionStart hook");
            return Ok(());
        }
    };

    // Check if JEB_CLAUDE_INITIAL_COMMIT already exists in env
    let initial_commit = if let Ok(existing) = std::env::var("JEB_CLAUDE_INITIAL_COMMIT") {
        // Preserve existing value
        eprintln!(
            "Preserving existing JEB_CLAUDE_INITIAL_COMMIT: {}",
            existing
        );
        existing
    } else {
        // Find git repository root from current working directory
        let repo_path = find_git_root(&input.cwd).context("Failed to find git repository")?;
        let repo = Repository::open(&repo_path).context("Failed to open repository")?;
        let head = repo.head().context("Failed to get HEAD")?;
        let commit = head
            .peel_to_commit()
            .context("Failed to peel HEAD to commit")?;
        let commit_id = commit.id().to_string();
        eprintln!("Initializing JEB_CLAUDE_INITIAL_COMMIT: {}", commit_id);
        commit_id
    };

    // Write/append to CLAUDE_ENV_FILE
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(&claude_env_file)
        .context("Failed to open CLAUDE_ENV_FILE")?;

    writeln!(file, "export JEB_CLAUDE_INITIAL_COMMIT={}", initial_commit)
        .context("Failed to write to CLAUDE_ENV_FILE")?;

    eprintln!("✓ Wrote JEB_CLAUDE_INITIAL_COMMIT={} to {}", initial_commit, claude_env_file);

    Ok(())
}
