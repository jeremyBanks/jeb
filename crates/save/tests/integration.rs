use {
    save::cli::Save,
    std::fs,
    std::sync::Mutex,
    once_cell::sync::Lazy,
    std::path::PathBuf,
};

static CWD_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

struct TestContext {
    original_cwd: PathBuf,
    _lock: std::sync::MutexGuard<'static, ()>,
}

impl TestContext {
    fn new(new_cwd: &std::path::Path) -> Self {
        let lock = CWD_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let original_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(new_cwd).unwrap();
        Self {
            original_cwd,
            _lock: lock,
        }
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        std::env::set_current_dir(&self.original_cwd).unwrap();
    }
}

#[test]
fn test_basic_save() {
    let yaml = r#"
HEAD: refs/heads/trunk
"#;
    let snapshot = git_snapshot::parse(yaml).unwrap();
    let temp_repo = snapshot.to_temporary_repository().unwrap();
    let repo_path = temp_repo.path().parent().unwrap();

    fs::write(repo_path.join("file1.txt"), "content1").unwrap();
    fs::write(repo_path.join("file2.txt"), "content2").unwrap();

    let _ctx = TestContext::new(repo_path);

    let args = Save::with(|_| {});
    args.save().expect("save failed");

    let roundtrip = temp_repo.to_snapshot().unwrap();
    assert_eq!(roundtrip.commits().count(), 1);
    let commit = roundtrip.head_commit().expect("No HEAD commit");
    assert_eq!(commit.tree.get("file1.txt"), Some("content1"));
    assert_eq!(commit.tree.get("file2.txt"), Some("content2"));
    assert!(commit.message.starts_with("r0"));
}

#[test]
fn test_staged_save() {
    let yaml = r#"
HEAD: refs/heads/trunk
"#;
    let snapshot = git_snapshot::parse(yaml).unwrap();
    let temp_repo = snapshot.to_temporary_repository().unwrap();
    let repo_path = temp_repo.path().parent().unwrap();

    fs::write(repo_path.join("staged.txt"), "staged content").unwrap();
    fs::write(repo_path.join("unstaged.txt"), "unstaged content").unwrap();

    {
        let mut index = temp_repo.index().unwrap();
        index.add_path(std::path::Path::new("staged.txt")).unwrap();
        index.write().unwrap();
    }

    let _ctx = TestContext::new(repo_path);

    let args = Save::with(|s| {
        s.staged = true;
    });
    args.save().expect("save --staged failed");

    let roundtrip = temp_repo.to_snapshot().unwrap();
    let commit = roundtrip.head_commit().expect("No HEAD commit");
    
    assert_eq!(commit.tree.get("staged.txt"), Some("staged content"));
    assert_eq!(commit.tree.get("unstaged.txt"), None);
}

#[test]
fn test_allow_empty() {
    let yaml = r#"
HEAD: refs/heads/trunk
"#;
    let snapshot = git_snapshot::parse(yaml).unwrap();
    let temp_repo = snapshot.to_temporary_repository().unwrap();
    let repo_path = temp_repo.path().parent().unwrap();

    let _ctx = TestContext::new(repo_path);

    // Without --allow-empty, it should NOT create a commit if there are no changes
    let args = Save::with(|_| {});
    args.save().expect("save failed");
    
    let roundtrip = temp_repo.to_snapshot().unwrap();
    assert_eq!(roundtrip.commits().count(), 0);

    // With --allow-empty, it SHOULD create a commit
    let args = Save::with(|s| {
        s.allow_empty = true;
    });
    args.save().expect("save --allow-empty failed");

    let roundtrip = temp_repo.to_snapshot().unwrap();
    assert_eq!(roundtrip.commits().count(), 1);
}

#[test]
fn test_custom_message() {
    let yaml = r#"
HEAD: refs/heads/trunk
"#;
    let snapshot = git_snapshot::parse(yaml).unwrap();
    let temp_repo = snapshot.to_temporary_repository().unwrap();
    let repo_path = temp_repo.path().parent().unwrap();

    fs::write(repo_path.join("file.txt"), "content").unwrap();

    let _ctx = TestContext::new(repo_path);

    let args = Save::with(|s| {
        s.message = Some("Custom commit message".to_string());
    });
    args.save().expect("save --message failed");

    let roundtrip = temp_repo.to_snapshot().unwrap();
    let commit = roundtrip.head_commit().expect("No HEAD commit");
    assert_eq!(commit.message, "Custom commit message");
}

#[test]
fn test_authorship_override() {
    let yaml = r#"
HEAD: refs/heads/trunk
"#;
    let snapshot = git_snapshot::parse(yaml).unwrap();
    let temp_repo = snapshot.to_temporary_repository().unwrap();
    let repo_path = temp_repo.path().parent().unwrap();

    fs::write(repo_path.join("file.txt"), "content").unwrap();

    let _ctx = TestContext::new(repo_path);

    let args = Save::with(|s| {
        s.author = Some("Custom Author <author@example.com>".to_string());
    });
    args.save().expect("save --author failed");

    let roundtrip = temp_repo.to_snapshot().unwrap();
    let commit = roundtrip.head_commit().expect("No HEAD commit");
    assert_eq!(commit.author.name, "Custom Author");
    assert_eq!(commit.author.email, "author@example.com");
}

#[test]
fn test_graph_stats_increment() {
    let yaml = r#"
HEAD: refs/heads/trunk
"#;
    let snapshot = git_snapshot::parse(yaml).unwrap();
    let temp_repo = snapshot.to_temporary_repository().unwrap();
    let repo_path = temp_repo.path().parent().unwrap();

    let _ctx = TestContext::new(repo_path);

    // First commit
    fs::write(repo_path.join("file1.txt"), "1").unwrap();
    Save::with(|s| { s.allow_empty = true; }).save().unwrap();
    
    // Second commit
    fs::write(repo_path.join("file2.txt"), "2").unwrap();
    Save::with(|s| { s.allow_empty = true; }).save().unwrap();

    let roundtrip = temp_repo.to_snapshot().unwrap();
    assert_eq!(roundtrip.commits().count(), 2);
    
    // Check messages
    let mut messages: Vec<String> = roundtrip.commits().map(|c| c.message.clone()).collect();
    messages.sort(); // r0 ..., r1 ...
    
    assert!(messages[0].starts_with("r0"));
    assert!(messages[1].starts_with("r1"));
}

#[test]
fn test_agent_committer() {
    let yaml = r#"
HEAD: refs/heads/trunk
"#;
    let snapshot = git_snapshot::parse(yaml).unwrap();
    let temp_repo = snapshot.to_temporary_repository().unwrap();
    let repo_path = temp_repo.path().parent().unwrap();

    fs::write(repo_path.join("file.txt"), "content").unwrap();

    let _ctx = TestContext::new(repo_path);

    // Save current CLAUDECODE state and unset it (CLAUDECODE takes precedence over GEMINI_CLI)
    let claudecode_was_set = std::env::var("CLAUDECODE").ok();
    unsafe { std::env::remove_var("CLAUDECODE"); }

    // Set GEMINI_CLI environment variable
    unsafe { std::env::set_var("GEMINI_CLI", "1"); }

    let args = Save::with(|_| {});
    args.save().expect("save failed with agent env");

    unsafe { std::env::remove_var("GEMINI_CLI"); }
    // Restore CLAUDECODE if it was set
    if let Some(val) = claudecode_was_set {
        unsafe { std::env::set_var("CLAUDECODE", val); }
    }

    let roundtrip = temp_repo.to_snapshot().unwrap();
    let commit = roundtrip.head_commit().expect("No HEAD commit");

    // Committer should be Gemini CLI
    assert_eq!(commit.committer.name, "⟡ Gemini CLI");
    assert_eq!(commit.committer.email, "noreply@google.com");
}

#[test]
fn test_tree_override() {
    let yaml = r#"
HEAD: refs/heads/trunk
refs:
  heads:
    trunk: 1
1:
  message: initial
  tree:
    old.txt: old content
"#;
    let snapshot = git_snapshot::parse(yaml).unwrap();
    let temp_repo = snapshot.to_temporary_repository().unwrap();
    let repo_path = temp_repo.path().parent().unwrap();

    // Create a new tree in the repo without using save
    let new_tree_oid = {
        let mut index = temp_repo.index().unwrap();
        index.clear().unwrap();
        fs::write(repo_path.join("new.txt"), "new content").unwrap();
        index.add_path(std::path::Path::new("new.txt")).unwrap();
        index.write_tree().unwrap()
    };

    let _ctx = TestContext::new(repo_path);

    let args = Save::with(|s| {
        s.tree = Some(new_tree_oid.to_string());
    });
    args.save().expect("save --tree failed");

    let roundtrip = temp_repo.to_snapshot().unwrap();
    let commit = roundtrip.head_commit().expect("No HEAD commit");
    
    assert_eq!(commit.tree.get("new.txt"), Some("new content"));
    assert_eq!(commit.tree.get("old.txt"), None);
}

#[test]
fn test_ai_agent_email_skipped_for_author() {
    // Start with a repo that has a commit with a known author
    let yaml = r#"
HEAD: refs/heads/trunk
refs:
  heads:
    trunk: 1
1:
  message: initial
  author: Human User <human@example.com>
  tree:
    file.txt: initial
"#;
    let snapshot = git_snapshot::parse(yaml).unwrap();
    let temp_repo = snapshot.to_temporary_repository().unwrap();
    let repo_path = temp_repo.path().parent().unwrap();

    // Configure git with an AI agent email
    let repo = git2::Repository::open(repo_path).unwrap();
    repo.config().unwrap().set_str("user.name", "Claude Code").unwrap();
    repo.config().unwrap().set_str("user.email", "noreply@anthropic.com").unwrap();

    // Make a change
    fs::write(repo_path.join("file.txt"), "changed").unwrap();

    let _ctx = TestContext::new(repo_path);

    let args = Save::with(|_| {});
    args.save().expect("save failed");

    let roundtrip = temp_repo.to_snapshot().unwrap();
    let commit = roundtrip.head_commit().expect("No HEAD commit");

    // Author should be from previous commit (Human User), NOT from git config (Claude Code)
    assert_eq!(commit.author.name, "Human User");
    assert_eq!(commit.author.email, "human@example.com");
}

#[test]
fn test_explicit_author_overrides_ai_agent_detection() {
    // Start with a repo that has a commit with a known author
    let yaml = r#"
HEAD: refs/heads/trunk
refs:
  heads:
    trunk: 1
1:
  message: initial
  author: Human User <human@example.com>
  tree:
    file.txt: initial
"#;
    let snapshot = git_snapshot::parse(yaml).unwrap();
    let temp_repo = snapshot.to_temporary_repository().unwrap();
    let repo_path = temp_repo.path().parent().unwrap();

    // Configure git with an AI agent email
    let repo = git2::Repository::open(repo_path).unwrap();
    repo.config().unwrap().set_str("user.name", "Claude Code").unwrap();
    repo.config().unwrap().set_str("user.email", "noreply@anthropic.com").unwrap();

    // Make a change
    fs::write(repo_path.join("file.txt"), "changed").unwrap();

    let _ctx = TestContext::new(repo_path);

    // Use explicit --author flag - this should override the AI agent detection
    let args = Save::with(|s| {
        s.author = Some("Explicit Author <explicit@example.com>".to_string());
    });
    args.save().expect("save failed");

    let roundtrip = temp_repo.to_snapshot().unwrap();
    let commit = roundtrip.head_commit().expect("No HEAD commit");

    // Author should be the explicit one, not from previous commit
    assert_eq!(commit.author.name, "Explicit Author");
    assert_eq!(commit.author.email, "explicit@example.com");
}

#[test]
fn test_non_ai_agent_email_used_normally() {
    // Start with a repo that has a commit with a known author
    let yaml = r#"
HEAD: refs/heads/trunk
refs:
  heads:
    trunk: 1
1:
  message: initial
  author: Previous User <previous@example.com>
  tree:
    file.txt: initial
"#;
    let snapshot = git_snapshot::parse(yaml).unwrap();
    let temp_repo = snapshot.to_temporary_repository().unwrap();
    let repo_path = temp_repo.path().parent().unwrap();

    // Configure git with a normal (non-AI agent) email
    let repo = git2::Repository::open(repo_path).unwrap();
    repo.config().unwrap().set_str("user.name", "Regular Dev").unwrap();
    repo.config().unwrap().set_str("user.email", "dev@company.com").unwrap();

    // Make a change
    fs::write(repo_path.join("file.txt"), "changed").unwrap();

    let _ctx = TestContext::new(repo_path);

    let args = Save::with(|_| {});
    args.save().expect("save failed");

    let roundtrip = temp_repo.to_snapshot().unwrap();
    let commit = roundtrip.head_commit().expect("No HEAD commit");

    // Author should be from git config, not previous commit
    assert_eq!(commit.author.name, "Regular Dev");
    assert_eq!(commit.author.email, "dev@company.com");
}