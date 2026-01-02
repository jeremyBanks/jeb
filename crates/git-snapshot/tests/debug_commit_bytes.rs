use git_snapshot::{parse, Repository};
use sha1_checked::Digest;

#[test]
fn test_debug_commit_bytes() {
    // Parse using integer key so we can see the calculated hash
    let yaml = r#"
HEAD: refs/heads/main
refs:
  heads:
    main: 1
1:
  author: Jeremy Banks <_@jeremy.ca>
  author-date: 2026-01-02T21:10:36Z
  commit-date: 2026-01-02T21:10:36Z
  message: Initial commit
  tree:
    README.md: root readme
    src:
      lib:
        bar.txt: more code
        foo.txt: library code
"#;

    eprintln!("Parsing with integer key...");
    let repo = parse(yaml).expect("Should parse");
    let commit = repo.commits().next().expect("Should have commit");

    eprintln!("Commit ID (our calculation): {}", commit.id.to_hex());
    eprintln!("Author: {:?}", commit.author);
    eprintln!("Author-date: {:?}", commit.author_date);
    eprintln!("Committer: {:?}", commit.committer);
    eprintln!("Committer-date: {:?}", commit.committer_date);
    eprintln!("Message: {:?}", commit.message.as_bytes());
    eprintln!("Parents: {}", commit.parents.len());

    // Write to git
    eprintln!("\nWriting to git...");
    let temp_repo = repo.to_temporary_repository().expect("Should create temp repo");
    let read_back = Repository::from_git_dir(temp_repo.path()).expect("Should read from git");
    let git_commit = read_back.commits().next().expect("Should have commit");

    eprintln!("Commit ID (git calculation): {}", git_commit.id.to_hex());
    eprintln!("Author: {:?}", git_commit.author);
    eprintln!("Author-date: {:?}", git_commit.author_date);
    eprintln!("Committer: {:?}", git_commit.committer);
    eprintln!("Committer-date: {:?}", git_commit.committer_date);
    eprintln!("Message: {:?}", git_commit.message.as_bytes());
    eprintln!("Parents: {}", git_commit.parents.len());

    // Try to replicate the git commit object format
    eprintln!("\n--- Checking commit object format ---");
    eprintln!("Our commit hash: {}", commit.id.to_hex());
    eprintln!("Git commit hash: {}", git_commit.id.to_hex());

    // Try to manually compute what the git hash should be
    // by getting the git object directly
    eprintln!("\nTree hashes:");
    eprintln!("  Our tree: (calculated from files)");
    eprintln!("  Git tree: {}", git_commit.tree.paths().next());
}
