use {
    once_cell::sync::Lazy,
    save::cli::Save,
    std::{fs, path::PathBuf, sync::Mutex},
};

static CWD_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

struct TestContext {
    original_cwd: PathBuf,
    _lock: std::sync::MutexGuard<'static, ()>,
}

impl TestContext {
    fn new(new_cwd: &std::path::Path) -> Self {
        let lock = CWD_MUTEX.lock().unwrap();
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
    Save::with(|s| {
        s.allow_empty = true;
    })
    .save()
    .unwrap();

    // Second commit
    fs::write(repo_path.join("file2.txt"), "2").unwrap();
    Save::with(|s| {
        s.allow_empty = true;
    })
    .save()
    .unwrap();

    let roundtrip = temp_repo.to_snapshot().unwrap();
    assert_eq!(roundtrip.commits().count(), 2);

    // Check messages
    let mut messages: Vec<String> = roundtrip.commits().map(|c| c.message.clone()).collect();
    messages.sort(); // r0 ..., r1 ...

    assert!(messages[0].starts_with("r0"));
    assert!(messages[1].starts_with("r1"));
}

/// All environment variables that affect committer detection, used to isolate
/// tests from the host environment.
const COMMITTER_ENV_VARS: &[&str] = &[
    "CLAUDECODE",
    "CLAUDE_CODE_REMOTE",
    "GEMINI_CLI",
    "CURSOR_AGENT",
    "GIT_COMMITTER_NAME",
    "GIT_COMMITTER_EMAIL",
];

/// Clear all committer-related env vars, returning their previous values for
/// restoration.
fn clear_committer_env() -> Vec<(&'static str, Option<String>)> {
    COMMITTER_ENV_VARS
        .iter()
        .map(|&var| {
            let prev = std::env::var(var).ok();
            unsafe { std::env::remove_var(var) };
            (var, prev)
        })
        .collect()
}

/// Restore previously saved env vars.
fn restore_committer_env(saved: Vec<(&'static str, Option<String>)>) {
    for (var, val) in saved {
        unsafe {
            match val {
                Some(v) => std::env::set_var(var, v),
                None => std::env::remove_var(var),
            }
        }
    }
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

    // Clear all committer env vars and set only the one we're testing
    let saved = clear_committer_env();
    unsafe {
        std::env::set_var("GEMINI_CLI", "1");
    }

    let args = Save::with(|_| {});
    let result = args.save();

    // Restore env before asserting so cleanup happens even on failure
    unsafe {
        std::env::remove_var("GEMINI_CLI");
    }
    restore_committer_env(saved);

    result.expect("save failed with agent env");

    let roundtrip = temp_repo.to_snapshot().unwrap();
    let commit = roundtrip.head_commit().expect("No HEAD commit");

    // Committer should be Gemini CLI
    assert_eq!(commit.committer.name, "⟡ Gemini CLI");
    assert_eq!(commit.committer.email, "gemini-cli@google.com");
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
fn test_phonetic_encoding() {
    let yaml = r#"
HEAD: refs/heads/trunk
"#;
    let snapshot = git_snapshot::parse(yaml).unwrap();
    let temp_repo = snapshot.to_temporary_repository().unwrap();
    let repo_path = temp_repo.path().parent().unwrap();

    fs::write(repo_path.join("file.txt"), "content").unwrap();

    let _ctx = TestContext::new(repo_path);

    let args = Save::with(|_| {});
    args.save().expect("save failed");

    let roundtrip = temp_repo.to_snapshot().unwrap();
    let commit = roundtrip.head_commit().expect("No HEAD commit");

    // Message should contain phonetic encoding
    let message = &commit.message;

    // Should have format: r0 / xHHHH phonetic-words
    assert!(message.starts_with("r0 / x"));

    // Extract tree hash from message (format: r0 / xHHHH word word word word)
    let parts: Vec<&str> = message.split_whitespace().collect();
    assert!(parts.len() >= 7); // r0 / x1234 word word word word

    // Find the xHHHH part
    let tree_hex_pos = parts
        .iter()
        .position(|p| p.starts_with('x'))
        .expect("Should have tree hash in message");

    let tree_hex = parts[tree_hex_pos].trim_start_matches('x');

    assert_eq!(tree_hex.len(), 4, "Tree hash should be 4 hex chars");

    // Phonetic words should be immediately after the tree hash
    // Format: r0 / xHHHH word word word word [/ oHHHH]
    // Extract 4 words after xHHHH
    let phonetic_words = &parts[tree_hex_pos + 1..tree_hex_pos + 5];
    assert_eq!(
        phonetic_words.len(),
        4,
        "Should have 4 phonetic words for 4 hex chars"
    );

    // Verify each phonetic word is valid
    let valid_phonetics = [
        "zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "alfa",
        "bravo", "charlie", "delta", "echo", "foxtrot",
    ];

    for word in phonetic_words {
        assert!(
            valid_phonetics.contains(word),
            "Invalid phonetic word: {}",
            word
        );
    }
}

#[test]
fn test_phonetic_omitted_for_empty_tree() {
    let yaml = r#"
HEAD: refs/heads/trunk
refs:
  heads:
    trunk: 1
1:
  message: initial
  tree:
    file.txt: content
"#;
    let snapshot = git_snapshot::parse(yaml).unwrap();
    let temp_repo = snapshot.to_temporary_repository().unwrap();
    let repo_path = temp_repo.path().parent().unwrap();

    // Don't add any files - tree will be empty
    let _ctx = TestContext::new(repo_path);

    let args = Save::with(|_| {});
    args.save().expect("save failed");

    let roundtrip = temp_repo.to_snapshot().unwrap();
    let commit = roundtrip.head_commit().expect("No HEAD commit");

    // Message should NOT contain tree hash or phonetic for empty tree
    let message = &commit.message;

    // Should start with r1 and not have xHHHH
    assert!(message.starts_with("r1"));
    assert!(
        !message.contains(" / x"),
        "Empty tree should not have tree hash component"
    );

    // Should not contain phonetic words
    let valid_phonetics = [
        "zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "alfa",
        "bravo", "charlie", "delta", "echo", "foxtrot",
    ];

    let message_words: Vec<&str> = message.split_whitespace().collect();
    for word in message_words {
        assert!(
            !valid_phonetics.contains(&word),
            "Empty tree message should not contain phonetic words, found: {}",
            word
        );
    }
}
