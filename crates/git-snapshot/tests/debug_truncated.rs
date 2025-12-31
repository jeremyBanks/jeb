use git_snapshot::{
    CommitIdStyle,
    parse,
    serialize,
};

#[test]
fn test_truncated_hash_roundtrip() {
    // This is the serialized output from fixture 03
    let yaml_with_truncated = r#"77a352:
  message: commit 1
  tree:
    README.md: '# Original README'
    src:
      helper.rs: pub fn help() {}
      lib.rs: pub fn original() {}
HEAD: refs/heads/main
b9d0b12d6c37ac7e529e6206bf28c0168e85de61:
  message: commit 2
  tree:
    GUIDE.md: '# Original README'
    src:
      helper.rs: pub fn help() { /* updated */ }
      new.rs: pub fn new_func() {}
refs:
  heads:
    main: b9d0b12d6c37ac7e529e6206bf28c0168e85de61
"#;

    eprintln!("=== Parsing YAML with truncated hash ===");
    let result = parse(yaml_with_truncated);

    match result {
        Ok(repo) => {
            eprintln!(
                "Parse successful! Repo has {} commits",
                repo.commits().count()
            );
            for commit in repo.commits() {
                eprintln!(
                    "  Commit: {} (parents: {})",
                    commit.id.to_hex(),
                    commit.parents.len()
                );
                for parent_id in &commit.parents {
                    eprintln!("    Parent: {}", parent_id.to_hex());
                }
            }
            eprintln!("HEAD: {:?}", repo.head());
            eprintln!("Refs: {} refs", repo.refs().count());

            eprintln!("\n=== Serializing back ===");
            let output = serialize(&repo, CommitIdStyle::Hex);
            eprintln!("{}", output);

            eprintln!("\n=== Second parse ===");
            let repo2 = parse(&output).expect("Second parse should work");
            eprintln!(
                "Second parse successful! Repo has {} commits",
                repo2.commits().count()
            );
        }
        Err(e) => {
            panic!("Parse failed: {}", e);
        }
    }
}
