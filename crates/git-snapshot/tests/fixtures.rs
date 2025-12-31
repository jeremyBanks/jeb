use std::{
    fs,
    path::{
        Path,
        PathBuf,
    },
};

/// Fixture-based tests for git-snapshot library
///
/// Tests follow the pattern described in IDEA-1.1.md:
/// 1. Read an input `.yaml` file
/// 2. Deserialize to in-memory representation
/// 3. Serialize back to YAML
/// 4. Compare against expected output file
/// 5. Verify round-trip stability (deserialize output, serialize again)
///
/// If expected output doesn't exist, it's auto-generated and the test fails.
use git_snapshot::{
    CommitIdStyle,
    parse,
    serialize,
};

/// Test a single fixture file
///
/// - `input_path`: Path to the input .yaml file (e.g.,
///   "tests/fixtures/basic.in.yaml")
/// - `expected_path`: Path to expected output .yaml file (e.g.,
///   "tests/fixtures/basic.out.yaml")
/// - `id_style`: Whether to use hex or integer commit IDs in output
fn test_fixture(input_path: &Path, expected_path: &Path, id_style: CommitIdStyle) {
    eprintln!("Testing fixture: {}", input_path.display());

    // Read input
    let input_yaml = fs::read_to_string(input_path)
        .unwrap_or_else(|e| panic!("Failed to read input file {:?}: {}", input_path, e));

    // Parse input
    let repo = parse(&input_yaml)
        .unwrap_or_else(|e| panic!("Failed to parse input {:?}: {}", input_path, e));

    // Serialize back
    let output_yaml = serialize(&repo, id_style);

    // Check if expected output exists
    let expected_exists = expected_path.exists();
    let mut test_failed = false;

    if expected_exists {
        // Compare with expected
        let expected_yaml = fs::read_to_string(expected_path)
            .unwrap_or_else(|e| panic!("Failed to read expected file {:?}: {}", expected_path, e));

        if output_yaml != expected_yaml {
            eprintln!("MISMATCH in fixture: {}", input_path.display());
            eprintln!("Expected output differs from actual output");
            eprintln!("\n=== EXPECTED ===\n{}", expected_yaml);
            eprintln!("\n=== ACTUAL ===\n{}", output_yaml);
            test_failed = true;
        }
    } else {
        // Generate expected output
        eprintln!(
            "Generating missing expected output: {}",
            expected_path.display()
        );
        fs::write(expected_path, &output_yaml).unwrap_or_else(|e| {
            panic!("Failed to write expected output {:?}: {}", expected_path, e)
        });
        test_failed = true; // Fail the test so user knows to review generated file
    }

    // Round-trip stability check
    let repo2 = parse(&output_yaml)
        .unwrap_or_else(|e| panic!("Failed to parse serialized output {:?}: {}", input_path, e));

    let output_yaml2 = serialize(&repo2, id_style);

    if output_yaml != output_yaml2 {
        eprintln!(
            "ROUND-TRIP INSTABILITY in fixture: {}",
            input_path.display()
        );
        eprintln!("Second serialization differs from first");
        eprintln!("\n=== FIRST ===\n{}", output_yaml);
        eprintln!("\n=== SECOND ===\n{}", output_yaml2);
        panic!("Round-trip stability check failed for {:?}", input_path);
    }

    if test_failed {
        if !expected_exists {
            panic!(
                "Generated missing expected output file: {:?}. Please review and re-run tests.",
                expected_path
            );
        } else {
            panic!("Output mismatch for fixture: {:?}", input_path);
        }
    }
}

/// Find all fixture input files in the fixtures directory
fn find_fixtures() -> Vec<(PathBuf, PathBuf, CommitIdStyle)> {
    let fixtures_dir = Path::new("tests/fixtures");
    let mut fixtures = Vec::new();

    if !fixtures_dir.exists() {
        return fixtures;
    }

    for entry in fs::read_dir(fixtures_dir).expect("Failed to read fixtures directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();

        // Look for files ending with .in.yaml
        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            if file_name.ends_with(".in.yaml") {
                // Determine the ID style based on filename
                let id_style = if file_name.contains(".hex.") {
                    CommitIdStyle::Hex
                } else if file_name.contains(".int.") {
                    CommitIdStyle::Integer
                } else {
                    // Default to hex
                    CommitIdStyle::Hex
                };

                // Construct expected output path
                let out_file_name = file_name.replace(".in.yaml", ".out.yaml");
                let expected_path = path.with_file_name(out_file_name);

                fixtures.push((path, expected_path, id_style));
            }
        }
    }

    fixtures.sort_by(|a, b| a.0.cmp(&b.0));
    fixtures
}

#[test]
fn test_all_fixtures() {
    let fixtures = find_fixtures();

    if fixtures.is_empty() {
        eprintln!("WARNING: No fixture files found in tests/fixtures/");
        eprintln!("Fixture files should be named *.in.yaml with optional .hex. or .int. markers");
        return;
    }

    let mut failed = Vec::new();

    for (input_path, expected_path, id_style) in fixtures {
        let result = std::panic::catch_unwind(|| {
            test_fixture(&input_path, &expected_path, id_style);
        });

        if result.is_err() {
            failed.push(input_path.clone());
        }
    }

    if !failed.is_empty() {
        panic!(
            "Fixture tests failed for {} file(s): {:?}",
            failed.len(),
            failed
        );
    }
}
