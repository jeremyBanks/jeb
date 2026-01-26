use {
    git_snapshot::{
        CommitIdStyle,
        SerializationOptions,
        serialize,
    },
    inline::snap,
    once_cell::sync::Lazy,
    save::cli::Save,
    std::{
        fs,
        path::PathBuf,
        sync::Mutex,
    },
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
    })
    .save()
    .expect("save failed");

    let result = temp_repo.to_snapshot().unwrap();
    let output = serialize(
        &result,
        CommitIdStyle::Integer,
        SerializationOptions::default(),
    );

    // We expect the new commit 5 to be added
    assert!(
        snap!(
            r#"HEAD: refs/heads/main
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
"#
        ) == output
    );
}

#[test]
fn test_squash_to() {
    let yaml = r#"
HEAD: refs/heads/main
refs:
  heads:
    main: 3
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
"#;
    let snapshot = git_snapshot::parse(yaml).unwrap();
    let temp_repo = snapshot.to_temporary_repository().unwrap();
    let repo_path = temp_repo.path().parent().unwrap();

    fs::write(repo_path.join("a"), "4").unwrap();

    let _ctx = TestContext::new(repo_path);

    // Squash to 1 (this means new commit's parent is 1)
    Save::with(|s| {
        s.squash_to_ref = vec!["HEAD~2".to_string()];
        s.timeless = true;
        s.message = Some("squashed to one".to_string());
    })
    .save()
    .expect("save --squash-to failed");

    let result = temp_repo.to_snapshot().unwrap();
    let output = serialize(
        &result,
        CommitIdStyle::Integer,
        SerializationOptions::default(),
    );

    assert!(
        snap!(
            r#"HEAD: refs/heads/main
refs:
  heads:
    main: 2
1:
  author: Author <author@example.com>
  author-date: 1970-01-01T00:00:00Z
  commit-date: 1970-01-01T00:00:00Z
  message: one
  tree:
    a: "1"
2:
  parents: [1]
  author: dev <dev@localhost>
  author-date: 1970-06-26T17:31:44Z
  commit-date: 1970-06-26T17:31:44Z
  message: squashed to one
  tree:
    a: "4"
"#
        ) == output
    );
}

#[test]
fn test_trust_messages() {
    let yaml = r#"
HEAD: refs/heads/main
refs:
  heads:
    main: 2
1:
  message: r10
  tree: {file1: "1"}
2:
  parents: [1]
  message: r11
  tree: {file1: "1", file2: "2"}
"#;
    let snapshot = git_snapshot::parse(yaml).unwrap();
    let temp_repo = snapshot.to_temporary_repository().unwrap();
    let repo_path = temp_repo.path().parent().unwrap();

    // Ensure all files exist in workdir so save doesn't delete them
    fs::write(repo_path.join("file1"), "1").unwrap();
    fs::write(repo_path.join("file2"), "2").unwrap();
    fs::write(repo_path.join("file3"), "3").unwrap();

    let _ctx = TestContext::new(repo_path);

    // Should trust r11 and create r12
    Save::with(|s| {
        s.timeless = true;
    })
    .save()
    .expect("save failed");

    let result = temp_repo.to_snapshot().unwrap();
    let commit = result.head_commit().expect("No HEAD");
    assert!(
        commit.message.starts_with("r12"),
        "Message '{}' should start with 'r12'",
        commit.message
    );
}

#[test]
fn test_rebuild() {
    let yaml = r#"
HEAD: refs/heads/main
refs:
  heads:
    main: 2
1:
  message: r10
  tree: {file1: "1"}
2:
  parents: [1]
  message: r11
  tree: {file1: "1", file2: "2"}
"#;
    let snapshot = git_snapshot::parse(yaml).unwrap();
    let temp_repo = snapshot.to_temporary_repository().unwrap();
    let repo_path = temp_repo.path().parent().unwrap();

    fs::write(repo_path.join("file1"), "1").unwrap();
    fs::write(repo_path.join("file2"), "2").unwrap();
    fs::write(repo_path.join("file3"), "3").unwrap();

    let _ctx = TestContext::new(repo_path);

    // With --rebuild, it should ignore r11 and calculate based on graph (which is
    // root -> 1 -> 2)
    Save::with(|s| {
        s.rebuild = true;
        s.timeless = true;
    })
    .save()
    .expect("save --rebuild failed");

    let result = temp_repo.to_snapshot().unwrap();
    let commit = result.head_commit().expect("No HEAD");
    // Graph is 1(r0) -> 2(r1) -> new(r2)
    assert!(
        commit.message.starts_with("r2"),
        "Message '{}' should start with 'r2'",
        commit.message
    );
}

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

    // HEAD is 2.
    // We want new commit to have parent 3 (other).
    // Default parents for new commit is [HEAD] (which is 2).
    // So we add 3 (other) and remove HEAD (2).
    Save::with(|s| {
        s.added_parent_ref = vec!["other".to_string()];
        s.removed_parent_ref = vec!["HEAD".to_string()];
        s.timeless = true;
        s.message = Some("new parents".to_string());
    })
    .save()
    .expect("save --add-parent --remove-parent failed");

    let result = temp_repo.to_snapshot().unwrap();
    let commit = result.head_commit().expect("No HEAD");

    // Verify parents
    assert_eq!(commit.parents.len(), 1);

    let output = serialize(
        &result,
        CommitIdStyle::Integer,
        SerializationOptions::default(),
    );
    assert!(
        snap!(
            r#"HEAD: refs/heads/main
refs:
  heads:
    main: 4
    other: 3
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
"#
        ) == output
    );
}
