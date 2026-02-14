//! Encodes the release binary itself as extended Z855 and saves to target/
//!
//! Run with: cargo run --example self_encode

use std::fs;
use std::io::Write;
use std::process::Command;

fn main() {
    // Rebuild the CLI in release mode
    println!("Building release binary...");
    let status = Command::new("cargo")
        .args(["build", "--release"])
        .status()
        .expect("Failed to run cargo build");

    if !status.success() {
        eprintln!("error: cargo build failed");
        std::process::exit(1);
    }

    // Read the release binary
    let binary_path = "target/release/z855";
    println!("Reading binary from {}...", binary_path);
    let binary_data = fs::read(binary_path).expect("Failed to read release binary");
    println!("Binary size: {} bytes", binary_data.len());

    // Encode as extended Z855
    println!("Encoding as extended Z855...");
    let encoded = z855::encode(&binary_data);
    println!("Extended Z855 size: {} characters", encoded.len());

    // Also encode as standard Z85 for comparison
    println!("Encoding as standard Z85...");
    let standard = z855::z855::encode_standard(&binary_data);
    println!("Standard Z85 size: {} characters", standard.len());

    let expected_len = (binary_data.len() * 5 + 3) / 4;
    println!("Expected length: {} characters", expected_len);
    println!("Difference (extended - standard): {} characters",
             encoded.len() as i64 - standard.len() as i64);

    // Try decoding to verify correctness
    match z855::decode(&encoded) {
        Ok(decoded) => {
            if decoded == binary_data {
                println!("Decoding verification: OK");
            } else {
                println!("Decoding verification: FAILED (decoded {} bytes vs original {} bytes)",
                         decoded.len(), binary_data.len());
            }
        }
        Err(e) => {
            println!("Decoding verification: ERROR {:?}", e);
        }
    }

    // Insert newlines every 80 characters
    let mut with_newlines = String::new();
    for (i, ch) in encoded.chars().enumerate() {
        if i > 0 && i % 80 == 0 {
            with_newlines.push('\n');
        }
        with_newlines.push(ch);
    }
    with_newlines.push('\n'); // trailing newline

    // Save to target/
    let output_path = "target/z855-release.z855.txt";
    println!("Writing to {}...", output_path);
    let mut file = fs::File::create(output_path).expect("Failed to create output file");
    file.write_all(with_newlines.as_bytes()).expect("Failed to write output");

    println!("Done!");
    println!("Lines: {}", with_newlines.lines().count());
}
