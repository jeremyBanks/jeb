//! Test polyglot with multiple files that stay under the 42KB total limit
//!
//! Current limitation: ~42KB TOTAL content due to single IDAT deflate block

use indexmap::IndexMap;
use std::fs;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating polyglot PNG+ZIP file with multiple files (within limits)...\n");

    // Total must stay under ~42KB
    const MAX_TOTAL: usize = 40_000;

    let mut files = IndexMap::new();

    // Split content across multiple files
    let per_file = MAX_TOTAL / 5;

    // File 1: Text file
    let text_chunk = "The quick brown fox jumps over the lazy dog. ";
    let large_text: String = text_chunk.repeat(per_file / text_chunk.len());
    println!("  text.txt: {} bytes", large_text.len());
    files.insert(b"text.txt".to_vec(), large_text.into_bytes());

    // File 2: Binary sequential data
    let binary_data: Vec<u8> = (0..per_file).map(|i| (i % 256) as u8).collect();
    println!("  data.bin: {} bytes", binary_data.len());
    files.insert(b"data.bin".to_vec(), binary_data);

    // File 3: Pseudo-random data
    let mut rng_state: u64 = 12345;
    let random_data: Vec<u8> = (0..per_file)
        .map(|_| {
            rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
            ((rng_state >> 16) & 0xFF) as u8
        })
        .collect();
    println!("  rand.bin: {} bytes", random_data.len());
    files.insert(b"rand.bin".to_vec(), random_data);

    // File 4: JSON-like data
    let json_entry = r#"{"id":123,"name":"Test"},"#;
    let json_data = format!("[{}]", json_entry.repeat(per_file / json_entry.len()));
    println!("  data.json: {} bytes", json_data.len());
    files.insert(b"data.json".to_vec(), json_data.into_bytes());

    // File 5: Small file
    files.insert(b"small.txt".to_vec(), b"Hello, World!".to_vec());
    println!("  small.txt: 13 bytes");

    let total_content: usize = files.values().map(|v| v.len()).sum();
    println!("\nTotal uncompressed content: {} bytes ({:.1} KB)",
             total_content, total_content as f64 / 1024.0);

    if total_content > 42000 {
        println!("WARNING: Content exceeds ~42KB limit, extraction may fail!");
    }

    let files_converted: zipng::Files = files.into();
    let polyglot_data = zipng::zipng(&files_converted);

    println!("Polyglot size: {} bytes ({:.1} KB)",
             polyglot_data.len(), polyglot_data.len() as f64 / 1024.0);
    println!("Overhead: {:.1}%",
             (polyglot_data.len() as f64 / total_content as f64 - 1.0) * 100.0);

    // Save it
    let output_path = "target/polyglot_medium.png";
    fs::create_dir_all("target")?;
    fs::write(output_path, &polyglot_data)?;
    println!("\nSaved to {}", output_path);

    // Verify with system tools
    println!("\n--- Verification ---");
    match Command::new("file").arg(output_path).output() {
        Ok(output) => println!("{}", String::from_utf8_lossy(&output.stdout).trim()),
        Err(_) => println!("(file command not available)"),
    }

    // Test extraction
    let extract_dir = "target/polyglot_medium_extracted";
    let _ = fs::remove_dir_all(extract_dir);
    fs::create_dir_all(extract_dir)?;

    match Command::new("unzip")
        .args(["-o", "-q", output_path, "-d", extract_dir])
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                println!("\n✓ Extraction successful!");

                // Verify sizes
                let mut total_extracted = 0usize;
                for entry in fs::read_dir(extract_dir)? {
                    let entry = entry?;
                    let size = entry.metadata()?.len() as usize;
                    total_extracted += size;
                    println!("  {}: {} bytes",
                             entry.file_name().to_string_lossy(), size);
                }

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
