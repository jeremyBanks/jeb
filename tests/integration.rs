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
        let encoded_path = test_cases_dir.join(format!("{}.encoded", base_name));

        let input_bytes = fs::read(&input_path).expect("Failed to read input file");
        let encoded_str = fs::read_to_string(&encoded_path).expect("Failed to read encoded file");

        // Check if this is an error test case
        let input_str = String::from_utf8_lossy(&input_bytes);

        if input_str == "<error />" {
            // This is a decode error test: encoded should fail to decode
            let result = run_rust_decode(&encoded_str);
            assert!(result.is_err(), "Expected decode error for {}", base_name);
        } else {
            // Normal test: encode input should match encoded
            let actual_encoded = run_rust_encode(&input_bytes);
            assert_eq!(actual_encoded, encoded_str, "encode mismatch for {}", base_name);

            // And decode should roundtrip
            let decoded = run_rust_decode(&encoded_str).expect("Decode failed");
            assert_eq!(decoded, input_bytes, "decode mismatch for {}", base_name);
        }
    }
}
