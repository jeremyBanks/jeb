use git_snapshot::{
    CommitIdStyle,
    SerializationOptions,
    parse,
    serialize,
};

#[test]
fn test_truncated_hash_roundtrip() {
    // Read the actual generated fixture
    let yaml_with_truncated =
        std::fs::read_to_string("tests/fixtures/03-tree-references.hex.out.yaml")
            .expect("fixture should exist");

    eprintln!("=== Parsing YAML with truncated hash ===");
    let result = parse(&yaml_with_truncated);

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
            let output = serialize(&repo, CommitIdStyle::Hex, SerializationOptions::default());
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
