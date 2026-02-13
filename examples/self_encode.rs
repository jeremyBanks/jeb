//! Encodes the release binary itself as extended Z85 and saves to target/
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
    let binary_path = "target/release/cleanroom";
    println!("Reading binary from {}...", binary_path);
    let binary_data = fs::read(binary_path).expect("Failed to read release binary");
    println!("Binary size: {} bytes", binary_data.len());

    // Encode as extended Z85
    println!("Encoding as extended Z85...");
    let encoded = cleanroom::encode(&binary_data);
    println!("Encoded size: {} characters", encoded.len());

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
    let output_path = "target/cleanroom-release.z85.txt";
    println!("Writing to {}...", output_path);
    let mut file = fs::File::create(output_path).expect("Failed to create output file");
    file.write_all(with_newlines.as_bytes()).expect("Failed to write output");

    println!("Done!");
    println!("Lines: {}", with_newlines.lines().count());
}
