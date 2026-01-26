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
fn test_merge_flow_timeless() {
...
#[test]
fn test_add_remove_parent() {
    let yaml = r#"
HEAD: refs/heads/main
refs:
  heads:
    main: 2
    other: 3
1:
  message: one
  tree: {a: "1"}
2:
  parents: [1]
  message: two
  tree: {a: "2"}
3:
  message: three
  tree: {b: "3"}
"#;
    let snapshot = git_snapshot::parse(yaml).unwrap();
    let temp_repo = snapshot.to_temporary_repository().unwrap();
    let repo_path = temp_repo.path().parent().unwrap();

    fs::write(repo_path.join("a"), "4").unwrap();

    let _ctx = TestContext::new(repo_path);
    
    // HEAD is 2. Parents: [1].
    // Add 3, Remove 1. Resulting parents should be [3].
    Save::with(|s| { 
        s.added_parent_ref = vec![":/three".to_string()];
        s.removed_parent_ref = vec![":/one".to_string()];
        s.timeless = true;
        s.message = Some("new parents".to_string());
    }).save().expect("save --add-parent --remove-parent failed");

    let result = temp_repo.to_snapshot().unwrap();
    let commit = result.head_commit().expect("No HEAD");
    
    // Verify parents
    // Since we don't know the exact OIDs, we check that it has 1 parent and it's 3.
    assert_eq!(commit.parents.len(), 1);
    
    let output = serialize(&result, CommitIdStyle::Integer, SerializationOptions::default());
    assert!(snap!(r#"HEAD: refs/heads/main
refs:
  heads:
    main: 4
1:
  author: Author <author@example.com>
  author-date: 1970-01-01T00:00:00Z
  commit-date: 1970-01-01T00:00:00Z
  message: one
  tree:
    a: "1"
2:
  parents: [1]
  author: Author <author@example.com>
  author-date: 1970-01-01T00:00:00Z
  commit-date: 1970-01-01T00:00:00Z
  message: two
  tree:
    a: "2"
3:
  author: Author <author@example.com>
  author-date: 1970-01-01T00:00:00Z
  commit-date: 1970-01-01T00:00:00Z
  message: three
  tree:
    b: "3"
4:
  parents: [3]
  author: dev <dev@localhost>
  author-date: 1970-06-26T17:31:44Z
  commit-date: 1970-06-26T17:31:44Z
  message: new parents
  tree:
    a: "4"
    b: "3"
"#) == output);
}
