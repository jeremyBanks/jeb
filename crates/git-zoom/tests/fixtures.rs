//! Fixture-based testing infrastructure for git-zoom
//!
//! Pattern:
//! - `{test-name}.in.yaml` - hand-written initial state
//! - `{test-name}.in.normalized.yaml` - auto-generated normalized version
//! - `{test-name}.final.yaml` - expected final state after operations

use std::{env, fs, path::Path};

use git_snapshot::{parse, serialize, CommitIdStyle, SerializationOptions};

use crate::helpers::TestRepo;

/// Run a fixture-based test
///
/// This function:
/// 1. Loads `{name}.in.yaml` from the fixtures directory
/// 2. Normalizes it and saves as `{name}.in.normalized.yaml` (always overwrites)
/// 3. Creates a TestRepo with the parsed state
/// 4. Runs the user-provided operations
/// 5. On TestRepo drop, compares final state with `{name}.final.yaml`
///
/// If `JEB_UPDATE_FIXTURES=1` is set, mismatched `.final.yaml` files are updated without failing.
pub fn test_fixture<F>(name: &str, operations: F)
where
    F: FnOnce(&TestRepo) -> Result<(), String>,
{
    let fixtures_dir = Path::new("tests/fixtures");

    // Paths for this test's fixtures
    let in_path = fixtures_dir.join(format!("{}.in.yaml", name));
    let normalized_path = fixtures_dir.join(format!("{}.in.normalized.yaml", name));
    let final_path = fixtures_dir.join(format!("{}.final.yaml", name));

    eprintln!("Testing fixture: {}", name);

    // 1. Read input fixture
    let input_yaml = fs::read_to_string(&in_path)
        .unwrap_or_else(|e| panic!("Failed to read input fixture {:?}: {}", in_path, e));

    // 2. Parse input
    let repo_snapshot = parse(&input_yaml)
        .unwrap_or_else(|e| panic!("Failed to parse input fixture {:?}: {}", in_path, e));

    // 3. Normalize and save (always overwrite, never fail)
    let normalized_yaml = serialize(
        &repo_snapshot,
        CommitIdStyle::Hex,
        SerializationOptions::default(),
    );
    fs::write(&normalized_path, &normalized_yaml)
        .unwrap_or_else(|e| panic!("Failed to write normalized fixture {:?}: {}", normalized_path, e));

    // 4. Create TestRepo with expectation tracking
    let mut repo = TestRepo::from_snapshot(repo_snapshot);
    repo.set_expected_fixture(final_path);

    // 5. Run user operations
    if let Err(e) = operations(&repo) {
        panic!("Test operations failed: {}", e);
    }

    // 6. TestRepo::drop will handle final comparison
}

/// Compare actual output with expected fixture, handling JEB_UPDATE_FIXTURES
pub fn compare_or_update_fixture(expected_path: &Path, actual_yaml: &str) {
    let update_mode = env::var("JEB_UPDATE_FIXTURES").is_ok();

    let expected_exists = expected_path.exists();

    if expected_exists {
        let expected_yaml = fs::read_to_string(expected_path)
            .unwrap_or_else(|e| panic!("Failed to read expected fixture {:?}: {}", expected_path, e));

        if actual_yaml != expected_yaml {
            if update_mode {
                eprintln!("UPDATE MODE: Overwriting {}", expected_path.display());
                fs::write(expected_path, actual_yaml)
                    .unwrap_or_else(|e| panic!("Failed to update fixture {:?}: {}", expected_path, e));
            } else {
                eprintln!("\n❌ Fixture mismatch: {}", expected_path.display());
                eprintln!("\nTo update fixtures, run:");
                eprintln!("  JEB_UPDATE_FIXTURES=1 cargo test");
                eprintln!("\n=== EXPECTED ===");
                eprintln!("{}", expected_yaml);
                eprintln!("\n=== ACTUAL ===");
                eprintln!("{}", actual_yaml);
                panic!("Fixture mismatch for {:?}. Run with JEB_UPDATE_FIXTURES=1 to update.", expected_path);
            }
        } else {
            eprintln!("✓ Fixture matches: {}", expected_path.display());
        }
    } else {
        // Missing fixture - create it
        eprintln!("Creating missing fixture: {}", expected_path.display());
        fs::write(expected_path, actual_yaml)
            .unwrap_or_else(|e| panic!("Failed to create fixture {:?}: {}", expected_path, e));
    }
}
