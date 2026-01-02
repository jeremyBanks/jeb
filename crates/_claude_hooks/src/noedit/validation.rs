use {
    super::{
        git_ops::{
            create_restoration_commit,
            find_git_root,
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
/// Handle Stop/SessionEnd hook: validate and restore any violated .noedit files
pub fn handle(input: &HookInput) -> Result<Option<HookOutput>> {
    eprintln!("=== .noedit validation hook CALLED ===");
    eprintln!("Session ID: {}", input.session_id);
    let repo_path = find_git_root(&input.cwd).context("Failed to find git repository")?;
    eprintln!("Found git repo at: {}", repo_path.display());
    let repo = Repository::open(&repo_path).context("Failed to open repository")?;
    let initial_commit_sha = match std::env::var("JEB_CLAUDE_INITIAL_COMMIT") {
        Ok(sha) => {
            eprintln!("Found JEB_CLAUDE_INITIAL_COMMIT: {}", sha);
            sha
        }
        Err(_) => {
            eprintln!("⚠ JEB_CLAUDE_INITIAL_COMMIT not set, skipping .noedit validation",);
            eprintln!("  Validation hook will not check for violations");
            return Ok(None);
        }
    };
    let initial_oid = Oid::from_str(&initial_commit_sha)
        .context("Failed to parse JEB_CLAUDE_INITIAL_COMMIT as git OID")?;
    let initial_commit = repo
        .find_commit(initial_oid)
        .context("Failed to find initial commit")?;
    let head = repo.head().context("Failed to get HEAD")?;
    let head_commit = head
        .peel_to_commit()
        .context("Failed to peel HEAD to commit")?;
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
    let matcher = NoeditMatcher::from_commit(&repo, &initial_commit)
        .context("Failed to load .noedit patterns from initial commit")?;
    let changed_files =
        get_changed_files(&repo, &initial_commit).context("Failed to get changed files")?;
    let cwd = Path::new(&input.cwd);
    let mut violated_files = Vec::new();
    for file in &changed_files {
        if matcher.matches_path(cwd, file) {
            violated_files.push(file.clone());
        }
    }
    if violated_files.is_empty() {
        eprintln!("No .noedit violations found");
        return Ok(None);
    }
    eprintln!(
        "Found {} .noedit violations, restoring...",
        violated_files.len()
    );
    for file in &violated_files {
        restore_file_from_commit(&repo, &initial_commit, file)
            .with_context(|| format!("Failed to restore file: {}", file.display()))?;
    }
    create_restoration_commit(&repo, &violated_files, &initial_commit_sha)
        .context("Failed to create restoration commit")?;
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
            file_list,
        )),
        permission_decision: None,
        hook_specific_output: None,
    }))
}
