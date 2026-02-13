// Integration tests for Z85 implementation
// Includes cross-testing with TypeScript/Deno CLI and shared test case verification

use std::process::{Command, Stdio};
use std::io::Write;
use std::fs;
use std::path::Path;

// Re-import the z85 module functions
// Note: For integration tests, we test via the CLI interface

/// Run the TypeScript/Deno CLI for encoding
fn run_deno_encode(input: &[u8]) -> String {
    let mut child = Command::new("deno")
        .args(["run", "/Users/jeb/cleanroom/main.ts", "encode"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn deno");

    {
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        stdin.write_all(input).expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to wait on child");
    String::from_utf8(output.stdout).expect("Invalid UTF-8 in output")
}

/// Run the TypeScript/Deno CLI for decoding
fn run_deno_decode(input: &str) -> Result<Vec<u8>, String> {
    let mut child = Command::new("deno")
        .args(["run", "/Users/jeb/cleanroom/main.ts", "decode"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn deno");

    {
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        stdin.write_all(input.as_bytes()).expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to wait on child");

    if output.status.success() {
        Ok(output.stdout)
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

/// Run the Rust CLI for encoding (self-test helper)
fn run_rust_encode(input: &[u8]) -> String {
    let mut child = Command::new("cargo")
        .args(["run", "--quiet", "--release", "--manifest-path", "/Users/jeb/cleanroom/Cargo.toml", "--", "encode"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn cargo");

    {
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        stdin.write_all(input).expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to wait on child");
    String::from_utf8(output.stdout).expect("Invalid UTF-8 in output")
}

/// Run the Rust CLI for decoding (self-test helper)
fn run_rust_decode(input: &str) -> Result<Vec<u8>, String> {
    let mut child = Command::new("cargo")
        .args(["run", "--quiet", "--release", "--manifest-path", "/Users/jeb/cleanroom/Cargo.toml", "--", "decode"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn cargo");

    {
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        stdin.write_all(input.as_bytes()).expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to wait on child");

    if output.status.success() {
        Ok(output.stdout)
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[test]
fn test_cross_encode_with_deno() {
    let test_cases: &[&[u8]] = &[
        &[],
        &[0],
        &[0, 0, 0, 0],
        &[0xff, 0xff, 0xff, 0xff],
        &[1, 2, 3, 4, 5, 6, 7, 8],
    ];

    for input in test_cases {
        let rust_result = run_rust_encode(input);
        let deno_result = run_deno_encode(input);
        assert_eq!(rust_result, deno_result, "encode mismatch for {:?}", input);
    }
}

#[test]
fn test_cross_decode_with_deno() {
    let test_cases = ["", "00", "00000", "%nSc0", "0000000000"];

    for input in test_cases {
        let rust_result = run_rust_decode(input).expect("Rust decode failed");
        let deno_result = run_deno_decode(input).expect("Deno decode failed");
        assert_eq!(rust_result, deno_result, "decode mismatch for {:?}", input);
    }
}

/// Holds all encoded file variants for a test case.
///
/// Test case structure:
/// - X.input: raw bytes to encode/decode
/// - X.encoded: standard Z85 encoding (MUST always be present)
/// - X.encoded-Y: alternative valid encodings (optional, any number)
/// - X.encoded-expected: if present, encoder MUST produce exactly this
struct EncodedFiles {
    standard: String,
    alternatives: Vec<String>,
    expected: Option<String>,
}

/// Find all encoded files for a given test case base name.
fn find_encoded_files(test_cases_dir: &Path, base_name: &str) -> EncodedFiles {
    let mut result = EncodedFiles {
        standard: String::new(),
        alternatives: Vec::new(),
        expected: None,
    };

    // Read the standard .encoded file
    let encoded_path = test_cases_dir.join(format!("{}.encoded", base_name));
    result.standard = fs::read_to_string(&encoded_path).expect("Failed to read encoded file");

    // Scan for alternative encoded files
    for entry in fs::read_dir(test_cases_dir).expect("Failed to read test-cases directory") {
        let entry = entry.expect("Failed to read directory entry");
        let file_name = entry.file_name().to_string_lossy().to_string();

        // Check for .encoded-expected
        let expected_name = format!("{}.encoded-expected", base_name);
        if file_name == expected_name {
            let path = test_cases_dir.join(&file_name);
            result.expected = Some(fs::read_to_string(&path).expect("Failed to read expected file"));
        }
        // Check for .encoded-Y pattern (but not .encoded-expected)
        else if file_name.starts_with(&format!("{}.encoded-", base_name)) && file_name != expected_name {
            let path = test_cases_dir.join(&file_name);
            let alt_encoded = fs::read_to_string(&path).expect("Failed to read alternative file");
            result.alternatives.push(alt_encoded);
        }
    }

    result
}

#[test]
fn test_shared_test_cases() {
    let test_cases_dir = Path::new("/Users/jeb/cleanroom/test-cases");

    for entry in fs::read_dir(test_cases_dir).expect("Failed to read test-cases directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();

        if !path.extension().map_or(false, |ext| ext == "input") {
            continue;
        }

        let base_name = path.file_stem().unwrap().to_str().unwrap();
        let input_path = test_cases_dir.join(format!("{}.input", base_name));

        let input_bytes = fs::read(&input_path).expect("Failed to read input file");

        // Check if this is an error test case
        let input_str = String::from_utf8_lossy(&input_bytes);

        if input_str == "<error />" {
            // This is a decode error test: all encoded files should fail to decode
            let encoded_files = find_encoded_files(test_cases_dir, base_name);

            // Test standard encoding fails
            let result = run_rust_decode(&encoded_files.standard);
            assert!(result.is_err(), "Expected decode error for {} (standard)", base_name);

            // Test alternatives also fail
            for (i, alt) in encoded_files.alternatives.iter().enumerate() {
                let result = run_rust_decode(alt);
                assert!(result.is_err(), "Expected decode error for {} (alternative {})", base_name, i);
            }

            // Test expected also fails if present
            if let Some(expected) = &encoded_files.expected {
                let result = run_rust_decode(expected);
                assert!(result.is_err(), "Expected decode error for {} (expected)", base_name);
            }
        } else {
            // Normal test case
            let encoded_files = find_encoded_files(test_cases_dir, base_name);

            // DECODE TESTS: All encoded files must decode to same input
            let decoded_standard = run_rust_decode(&encoded_files.standard).expect("Decode failed (standard)");
            assert_eq!(decoded_standard, input_bytes, "decode mismatch for {} (standard)", base_name);

            for (i, alt) in encoded_files.alternatives.iter().enumerate() {
                let decoded_alt = run_rust_decode(alt).expect(&format!("Decode failed (alternative {})", i));
                assert_eq!(decoded_alt, input_bytes, "decode mismatch for {} (alternative {})", base_name, i);
            }

            if let Some(expected) = &encoded_files.expected {
                let decoded_expected = run_rust_decode(expected).expect("Decode failed (expected)");
                assert_eq!(decoded_expected, input_bytes, "decode mismatch for {} (expected)", base_name);
            }

            // ENCODE TEST: result must match .encoded-expected if present, otherwise any .encoded* file
            let actual_encoded = run_rust_encode(&input_bytes);

            if let Some(expected) = &encoded_files.expected {
                // Must match expected exactly
                assert_eq!(actual_encoded, *expected, "encode must match expected for {}", base_name);
            } else {
                // Must match one of: standard, or any alternative
                let mut valid_encodings = vec![encoded_files.standard.clone()];
                valid_encodings.extend(encoded_files.alternatives.clone());
                let matches = valid_encodings.iter().any(|valid| actual_encoded == *valid);
                assert!(
                    matches,
                    "encode mismatch for {}: got \"{}\", expected one of: {:?}",
                    base_name, actual_encoded, valid_encodings
                );
            }
        }
    }
}
