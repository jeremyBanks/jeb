use {
    git_snapshot::{serialize, CommitIdStyle, SerializationOptions},
    save::cli::Save,
    std::fs,
    std::sync::Mutex,
    once_cell::sync::Lazy,
    std::path::PathBuf,
    inline::snap,
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
fn test_merge_flow_timeless() {
    let yaml = r#"
HEAD: refs/heads/main
refs:
  heads:
    main: 4
1:
  message: root
  tree:
    file1: root
2:
  parents: [1]
  message: left
  tree:
    file1: root
    file2: left
3:
  parents: [1]
  message: right
  tree:
    file1: root
    file3: right
4:
  parents: [2, 3]
  message: merge
  tree:
    file1: root
    file2: left
    file3: right
"#;
    let snapshot = git_snapshot::parse(yaml).unwrap();
    let temp_repo = snapshot.to_temporary_repository().unwrap();
    let repo_path = temp_repo.path().parent().unwrap();

    // Create a new file
    fs::write(repo_path.join("file4"), "new").unwrap();

    let _ctx = TestContext::new(repo_path);
    
    // Use timeless for deterministic output
    Save::with(|s| { 
        s.message = Some("post-merge".to_string()); 
        s.timeless = true;
    }).save().expect("save failed");

    let result = temp_repo.to_snapshot().unwrap();
    let output = serialize(&result, CommitIdStyle::Integer, SerializationOptions::default());

    // We expect the new commit 5 to be added
    assert!(snap!(r#"HEAD: refs/heads/main
refs:
  heads:
    main: 5
1:
  author: Author <author@example.com>
  author-date: 1970-01-01T00:00:00Z
  commit-date: 1970-01-01T00:00:00Z
  message: root
  tree:
    file1: root
2:
  parents: [1]
  author: Author <author@example.com>
  author-date: 1970-01-01T00:00:00Z
  commit-date: 1970-01-01T00:00:00Z
  message: left
  tree:
    file1: root
    file2: left
3:
  parents: [1]
  author: Author <author@example.com>
  author-date: 1970-01-01T00:00:00Z
  commit-date: 1970-01-01T00:00:00Z
  message: right
  tree:
    file1: root
    file3: right
4:
  parents: [2, 3]
  author: Author <author@example.com>
  author-date: 1970-01-01T00:00:00Z
  commit-date: 1970-01-01T00:00:00Z
  message: merge
  tree:
    file1: root
    file2: left
    file3: right
5:
  parents: [4]
  author: dev <dev@localhost>
  author-date: 1970-06-26T17:31:44Z
  commit-date: 1970-06-26T17:31:44Z
  message: post-merge
  tree:
    file1: root
    file2: left
    file3: right
    file4: new
"#) == output);
}

#[test]
fn test_z_mode_depth_limit() {
    // Chain of 5 commits: 1 -> 2 -> 3 -> 4 -> 5
    let yaml = r#"
HEAD: refs/heads/main
refs:
  heads:
    main: 5
1:
  message: one
  tree: {a: "1"}
2:
  parents: [1]
  message: two
  tree: {a: "2"}
3:
  parents: [2]
  message: three
  tree: {a: "3"}
4:
  parents: [3]
  message: four
  tree: {a: "4"}
5:
  parents: [4]
  message: five
  tree: {a: "5"}
"#;
    let snapshot = git_snapshot::parse(yaml).unwrap();
    let temp_repo = snapshot.to_temporary_repository().unwrap();
    let repo_path = temp_repo.path().parent().unwrap();

    fs::write(repo_path.join("a"), "6").unwrap();

    let _ctx = TestContext::new(repo_path);
    
    // Set max_depth to 2. Chain is 5 deep. Should trigger z-mode.
    Save::with(|s| { 
        s.max_depth = 2;
        s.timeless = true; 
    }).save().expect("save failed");

    let result = temp_repo.to_snapshot().unwrap();
    let commit = result.head_commit().expect("No HEAD");
    
    assert!(commit.message.starts_with('z'), "Message '{}' should start with 'z'", commit.message);
    
    let output = serialize(&result, CommitIdStyle::Integer, SerializationOptions::default());
    assert!(snap!("") == output);
}

#[test]
fn test_squash() {
    let yaml = r#"
HEAD: refs/heads/main
refs:
  heads:
    main: 2
1:
  message: one
  tree: {a: "1"}
2:
  parents: [1]
  message: two
  tree: {a: "2"}
"#;
    let snapshot = git_snapshot::parse(yaml).unwrap();
    let temp_repo = snapshot.to_temporary_repository().unwrap();
    let repo_path = temp_repo.path().parent().unwrap();

    fs::write(repo_path.join("a"), "3").unwrap();

    let _ctx = TestContext::new(repo_path);
    
    // Squash 1 (amend HEAD)
    Save::with(|s| { 
        s.squash = 1;
        s.timeless = true;
        s.message = Some("amended".to_string());
    }).save().expect("save --squash failed");

    let result = temp_repo.to_snapshot().unwrap();
    let output = serialize(&result, CommitIdStyle::Integer, SerializationOptions::default());
    
    // Result should have:
    // - Commit 3 (new HEAD) with parent 1 (because 2 was squashed)
    // - Wait, squash=1 means amend the last commit?
    // "Squashes these changes into the first parent."
    // If squash=1, we squash current changes into HEAD.
    // So new commit replaces HEAD (2). Parent of new commit should be parent of HEAD (1).
    // So parents: [1].
    
    assert!(snap!("") == output);
}
