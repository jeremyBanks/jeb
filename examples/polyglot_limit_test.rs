//! Test to find the exact size limit for polyglot files
//!
//! The IDAT chunk uses deflate stored blocks with max 65535 bytes each.
//! Files that span these boundaries get corrupted.

use indexmap::IndexMap;
use std::fs;
use std::process::Command;

fn test_size(total_bytes: usize) -> bool {
    let mut files = IndexMap::new();

    // Create a single file of the target size
    let data: Vec<u8> = (0..total_bytes).map(|i| (i % 256) as u8).collect();
    files.insert(b"test.bin".to_vec(), data);

    let files_converted: zipng::Files = files.into();
    let polyglot_data = zipng::zipng(&files_converted);

    // Try to extract
    let output_path = format!("target/limit_test_{}.png", total_bytes);
    fs::write(&output_path, &polyglot_data).unwrap();

    let result = Command::new("unzip")
        .args(["-o", "-q", &output_path, "-d", "target/limit_test_extract"])
        .output();

    let _ = fs::remove_dir_all("target/limit_test_extract");

    match result {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

fn main() {
    println!("Testing polyglot size limits...\n");

    fs::create_dir_all("target").unwrap();
    let _ = fs::remove_dir_all("target/limit_test_extract");

    // Binary search to find the limit
    let mut low = 1000;
    let mut high = 50000;

    // First verify low works and high fails
    println!("Checking bounds...");
    if !test_size(low) {
        println!("ERROR: Even {} bytes fails!", low);
        return;
    }
    println!("  {} bytes: OK", low);

    if test_size(high) {
        println!("  {} bytes: OK (no limit found in range)", high);
        return;
    }
    println!("  {} bytes: FAIL", high);

    // Binary search
    println!("\nBinary searching for limit...");
    while high - low > 100 {
        let mid = (low + high) / 2;
        if test_size(mid) {
            println!("  {} bytes: OK", mid);
            low = mid;
        } else {
            println!("  {} bytes: FAIL", mid);
            high = mid;
        }
    }

    // Fine-tune
    println!("\nFine-tuning...");
    let mut limit = low;
    for size in (low..=high).step_by(10) {
        if test_size(size) {
            limit = size;
        } else {
            break;
        }
    }

    println!("\n=== RESULT ===");
    println!("Maximum working file size: ~{} bytes ({:.1} KB)", limit, limit as f64 / 1024.0);

    // Calculate theoretical limit
    // Filtered data = (header_rows + content_rows) * 14 bytes
    // header_rows ≈ 3-4 for a short filename
    // content_rows = ceil(content_bytes / 9)
    // Must fit in 65535 bytes of IDAT deflate data (minus zlib overhead)
    let max_filtered = 65535 - 7; // 2 zlib header + 5 deflate header
    let max_rows = max_filtered / 14;
    let header_rows = 4; // approximate
    let content_rows = max_rows - header_rows;
    let theoretical_max = content_rows * 9;

    println!("Theoretical max (single IDAT block): ~{} bytes ({:.1} KB)",
             theoretical_max, theoretical_max as f64 / 1024.0);

    // Clean up test files
    for entry in fs::read_dir("target").unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with("limit_test_") {
            let _ = fs::remove_file(entry.path());
        }
    }
}
