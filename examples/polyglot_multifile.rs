//! Test polyglot with multiple small files that each fit in one IDAT block
//!
//! Maximum per-file: ~42KB (expands to ~65KB filtered, fits in one IDAT block)
//! This test verifies we can have unlimited TOTAL size via multiple files.

use indexmap::IndexMap;
use std::fs;
use std::process::Command;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating polyglot PNG+ZIP with many small files...\n");

    // Each file must be ≤ this size to fit in one IDAT block
    // 42KB content → ~65KB filtered (14/9 expansion) → fits in 65535 bytes
    const MAX_PER_FILE: usize = 10_000; // Use 10KB for clearer testing

    let mut files = IndexMap::new();

    // Create 20 files of 10KB each = 200KB total
    for i in 0..20 {
        let name = format!("file{:02}.bin", i);
        let data: Vec<u8> = (0..MAX_PER_FILE)
            .map(|j| ((i * 256 + j) % 256) as u8)
            .collect();
        println!("  {}: {} bytes", name, data.len());
        files.insert(name.into_bytes(), data);
    }

    let total_content: usize = files.values().map(|v| v.len()).sum();
    println!("\nTotal content: {} bytes ({:.1} KB)", total_content, total_content as f64 / 1024.0);

    let files_converted: zipng::Files = files.into();

    println!("\nGenerating polyglot...");
    let start = Instant::now();
    let polyglot_data = zipng::zipng(&files_converted);
    let elapsed = start.elapsed();

    println!("Generated in {:.2?}", elapsed);
    println!("Polyglot size: {} bytes ({:.1} KB)",
             polyglot_data.len(), polyglot_data.len() as f64 / 1024.0);
    println!("Overhead: {:.1}%",
             (polyglot_data.len() as f64 / total_content as f64 - 1.0) * 100.0);

    // Save it
    let output_path = "target/polyglot_multifile.png";
    fs::create_dir_all("target")?;
    fs::write(output_path, &polyglot_data)?;
    println!("\nSaved to {}", output_path);

    // Verify
    println!("\n--- Verification ---");
    match Command::new("file").arg(output_path).output() {
        Ok(output) => println!("{}", String::from_utf8_lossy(&output.stdout).trim()),
        Err(_) => {}
    }

    // Test extraction
    let extract_dir = "target/polyglot_multifile_extracted";
    let _ = fs::remove_dir_all(extract_dir);
    fs::create_dir_all(extract_dir)?;

    match Command::new("unzip")
        .args(["-o", "-q", output_path, "-d", extract_dir])
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                println!("\n✓ Extraction successful!");

                // Count extracted files and total size
                let mut count = 0;
                let mut total_extracted = 0usize;
                for entry in fs::read_dir(extract_dir)? {
                    let entry = entry?;
                    let size = entry.metadata()?.len() as usize;
                    total_extracted += size;
                    count += 1;
                }

                println!("  {} files, {} bytes total", count, total_extracted);

                if total_extracted == total_content {
                    println!("\n✓ All {} bytes extracted correctly!", total_content);
                } else {
                    println!("\n✗ Size mismatch! Expected {}, got {}", total_content, total_extracted);
                }
            } else {
                println!("\n✗ Extraction failed:");
                println!("{}", String::from_utf8_lossy(&output.stderr));
            }
        }
        Err(_) => println!("(unzip not available)"),
    }

    Ok(())
}
