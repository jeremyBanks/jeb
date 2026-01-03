//! Debug test to understand git-snapshot behavior

mod common;

use {
    common::helpers::*,
    std::{
        env,
        process::Command,
    },
};

#[test]
fn test_debug_git_status() {
    let yaml = r#"
HEAD: refs/heads/main
refs:
  heads:
    main: 1
1:
  message: "Initial commit"
  tree:
    README.md: "readme"
    file.txt: "content"
"#;

    let repo = TestRepo::from_yaml(yaml);

    // Check git status
    let original_dir = env::current_dir().unwrap();
    env::set_current_dir(repo.workdir()).unwrap();

    let status = Command::new("git")
        .args(&["status", "--porcelain"])
        .output()
        .unwrap();

    println!("Git status output:");
    println!("{}", String::from_utf8_lossy(&status.stdout));

    let log = Command::new("git")
        .args(&["log", "--oneline", "-n", "5"])
        .output()
        .unwrap();

    println!("\nGit log:");
    println!("{}", String::from_utf8_lossy(&log.stdout));

    let ls = Command::new("ls").args(&["-la"]).output().unwrap();

    println!("\nDirectory contents:");
    println!("{}", String::from_utf8_lossy(&ls.stdout));

    env::set_current_dir(original_dir).unwrap();
}
