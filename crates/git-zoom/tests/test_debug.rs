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

#[test]
fn test_debug_nested_structure() {
    // Matches the complete-zoom-cycle fixture structure
    let yaml = r#"
HEAD: refs/heads/main
refs:
  heads:
    main: 1
1:
  message: "Initial commit"
  tree:
    README.md: "root readme"
    other.txt: "other file"
    src:
      lib:
        foo.txt: "original"
        bar.txt: "bar"
"#;

    let repo = TestRepo::from_yaml(yaml);

    let original_dir = env::current_dir().unwrap();
    env::set_current_dir(repo.workdir()).unwrap();

    println!("\n=== Before zoom ===");
    let status = Command::new("git")
        .args(&["status", "--porcelain"])
        .output()
        .unwrap();
    println!("Git status: {:?}", String::from_utf8_lossy(&status.stdout));

    let tree = Command::new("find")
        .args(&[".", "-type", "f", "-not", "-path", "./.git/*"])
        .output()
        .unwrap();
    println!("Files: {:?}", String::from_utf8_lossy(&tree.stdout));

    // Try running git-zoom
    println!("\n=== Running git-zoom in ===");
    let zoom = Command::new(env!("CARGO_BIN_EXE_git-zoom"))
        .args(&["in", "src/lib"])
        .output();

    match zoom {
        Ok(output) => {
            println!("Exit code: {:?}", output.status);
            println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
            println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }

    println!("\n=== After zoom attempt ===");
    let status = Command::new("git")
        .args(&["status", "--porcelain"])
        .output()
        .unwrap();
    println!("Git status: {:?}", String::from_utf8_lossy(&status.stdout));

    env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_debug_fixture_simulation() {
    // Simulate exactly what the fixture test does
    use std::fs;

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let fixtures_dir = std::path::Path::new(&manifest_dir).join("tests/fixtures");

    // 1. Read input fixture
    let in_path = fixtures_dir.join("complete-zoom-cycle.in.yaml");
    let input_yaml = fs::read_to_string(&in_path)
        .unwrap_or_else(|e| panic!("Failed to read input fixture {:?}: {}", in_path, e));

    // 2. Parse input
    let repo_snapshot = git_snapshot::parse(&input_yaml)
        .unwrap_or_else(|e| panic!("Failed to parse input fixture {:?}: {}", in_path, e));

    // 3. Normalize and save (this writes to the main project!)
    let normalized_path = fixtures_dir.join("complete-zoom-cycle.in.normalized.yaml");
    let normalized_yaml = git_snapshot::serialize(
        &repo_snapshot,
        git_snapshot::CommitIdStyle::Hex,
        git_snapshot::SerializationOptions::default(),
    );
    fs::write(&normalized_path, &normalized_yaml).unwrap();
    println!("Wrote normalized fixture to: {:?}", normalized_path);

    // 4. Create TestRepo
    let repo = TestRepo::from_snapshot(repo_snapshot);
    println!("Created temp repo at: {:?}", repo.workdir());

    // 5. Check git status in MAIN project directory
    println!("\n=== Main project git status ===");
    let main_status = Command::new("git")
        .args(&["status", "--porcelain"])
        .current_dir(&manifest_dir)
        .output()
        .unwrap();
    println!(
        "Main project status: {:?}",
        String::from_utf8_lossy(&main_status.stdout)
    );

    // 6. Run zoom in temp directory
    let original_dir = env::current_dir().unwrap();
    env::set_current_dir(repo.workdir()).unwrap();

    println!("\n=== Current directory for zoom ===");
    println!("CWD: {:?}", env::current_dir().unwrap());
    println!("Workdir: {:?}", repo.workdir());

    println!("\n=== Git toplevel ===");
    let toplevel = Command::new("git")
        .args(&["rev-parse", "--show-toplevel"])
        .output()
        .unwrap();
    println!("Toplevel: {:?}", String::from_utf8_lossy(&toplevel.stdout));

    println!("\n=== Git status in temp ===");
    let temp_status = Command::new("git")
        .args(&["status", "--porcelain"])
        .output()
        .unwrap();
    println!(
        "Temp status: {:?}",
        String::from_utf8_lossy(&temp_status.stdout)
    );

    // Use an inline snapshot like the integration tests do
    inline::cell("test".to_string()).value = "test".to_string();

    println!("\n=== Running git-zoom via run_zoom ===");
    // Use the same method as the actual fixture tests
    match repo.run_zoom(&["in", "src/lib"]) {
        Ok(()) => println!("run_zoom succeeded"),
        Err(e) => println!("run_zoom failed: {}", e),
    }

    env::set_current_dir(original_dir).unwrap();
}
