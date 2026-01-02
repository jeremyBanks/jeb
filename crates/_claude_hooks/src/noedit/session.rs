use crate::HookInput;
use eyre::{Context, Result};
use git2::Repository;
use std::io::Write;

/// Handle SessionStart hook: initialize or preserve JEB_CLAUDE_INITIAL_COMMIT
pub fn handle(input: &HookInput) -> Result<()> {
    // Get CLAUDE_ENV_FILE path from environment
    let claude_env_file = match std::env::var("CLAUDE_ENV_FILE") {
        Ok(path) => path,
        Err(_) => {
            eprintln!("CLAUDE_ENV_FILE not set, skipping .noedit session initialization");
            return Ok(());
        }
    };

    // Check if JEB_CLAUDE_INITIAL_COMMIT already exists in env
    let initial_commit = if let Ok(existing) = std::env::var("JEB_CLAUDE_INITIAL_COMMIT") {
        // Preserve existing value
        eprintln!("Preserving existing JEB_CLAUDE_INITIAL_COMMIT: {}", existing);
        existing
    } else {
        // Get current HEAD
        let repo = Repository::open(&input.cwd).context("Failed to open repository")?;
        let head = repo.head().context("Failed to get HEAD")?;
        let commit = head.peel_to_commit().context("Failed to peel HEAD to commit")?;
        let commit_id = commit.id().to_string();
        eprintln!("Initializing JEB_CLAUDE_INITIAL_COMMIT: {}", commit_id);
        commit_id
    };

    // Write/append to CLAUDE_ENV_FILE
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(claude_env_file)
        .context("Failed to open CLAUDE_ENV_FILE")?;

    writeln!(file, "JEB_CLAUDE_INITIAL_COMMIT={}", initial_commit)
        .context("Failed to write to CLAUDE_ENV_FILE")?;

    Ok(())
}
