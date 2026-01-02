use git_snapshot::{parse, Repository};

#[test]
fn test_fixture_hash_validation() {
    // This is the exact content from basic-zoom-in.in.normalized.yaml
    let yaml = r#"
HEAD: refs/heads/main
refs:
  heads:
    main: 03199c304255cb51507aef9ac1bb27bb858c67fd
03199c304255cb51507aef9ac1bb27bb858c67fd:
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

    eprintln!("Parsing fixture content...");
    let repo = parse(yaml).expect("Should parse");
    let commit = repo.commits().next().expect("Should have commit");

    eprintln!("Fixture key: 03199c304255cb51507aef9ac1bb27bb858c67fd");
    eprintln!("Our hash:    {}", commit.id.to_hex());
    eprintln!("Match: {}", commit.id.to_hex() == "03199c304255cb51507aef9ac1bb27bb858c67fd");

    // Write to git and see what hash we get
    eprintln!("\nWriting to git...");
    let temp_repo = repo.to_temporary_repository().expect("Should create temp repo");
    let read_back = Repository::from_git_dir(temp_repo.path()).expect("Should read from git");
    let git_commit = read_back.commits().next().expect("Should have commit");

    eprintln!("Git hash:    {}", git_commit.id.to_hex());
    eprintln!("Match with our hash: {}", git_commit.id == commit.id);
}
