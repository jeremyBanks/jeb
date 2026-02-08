//! Test polyglot with content near the maximum limit
//!
//! The polyglot approach has a ~42KB total content limit due to IDAT
//! deflate block boundaries. This test verifies behavior near that limit.

use indexmap::IndexMap;
use std::fs;
use std::process::Command;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating polyglot PNG+ZIP near the 42KB limit...\n");

    // Total must stay under ~42KB
    const TOTAL_TARGET: usize = 38_000; // Leave room for headers

    let mut files = IndexMap::new();

    // Split across multiple files
    let per_file = TOTAL_TARGET / 4;

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

    // File 4: JSON-like structured text
    let json_entry = r#"{"id":123,"name":"Test"},"#;
    let json_data = format!("[{}]", json_entry.repeat(per_file / json_entry.len()));
    println!("  data.json: {} bytes", json_data.len());
    files.insert(b"data.json".to_vec(), json_data.into_bytes());

    // File 5: Small file
    files.insert(b"small.txt".to_vec(), b"Hello!".to_vec());
    println!("  small.txt: 6 bytes");

    let total_content: usize = files.values().map(|v| v.len()).sum();
    println!("\nTotal uncompressed content: {} bytes ({:.2} MB)",
             total_content, total_content as f64 / 1_000_000.0);

    let files_converted: zipng::Files = files.into();

    // Generate the polyglot
    println!("\nGenerating polyglot...");
    let start = Instant::now();
    let polyglot_data = zipng::zipng(&files_converted);
    let elapsed = start.elapsed();

    println!("Generated in {:.2?}", elapsed);
    println!("Polyglot size: {} bytes ({:.2} MB)",
             polyglot_data.len(), polyglot_data.len() as f64 / 1_000_000.0);
    println!("Overhead: {:.1}%",
             (polyglot_data.len() as f64 / total_content as f64 - 1.0) * 100.0);

    // Save it
    let output_path = "target/polyglot_large.png";
    fs::create_dir_all("target")?;
    fs::write(output_path, &polyglot_data)?;
    println!("\n✓ Saved to {}", output_path);

    // Verify PNG
    println!("\n--- PNG Verification ---");
    match Command::new("file").arg(output_path).output() {
        Ok(output) => println!("{}", String::from_utf8_lossy(&output.stdout).trim()),
        Err(_) => println!("(file command not available)"),
    }

    // Verify ZIP listing
    println!("\n--- ZIP Verification ---");
    match Command::new("unzip").args(["-l", output_path]).output() {
        Ok(output) => {
            if output.status.success() {
                // Just show summary
                let out = String::from_utf8_lossy(&output.stdout);
                for line in out.lines().take(3) {
                    println!("{}", line);
                }
                println!("...");
                for line in out.lines().rev().take(3).collect::<Vec<_>>().into_iter().rev() {
                    println!("{}", line);
                }
            } else {
                println!("unzip -l failed: {}", String::from_utf8_lossy(&output.stderr));
            }
        }
        Err(_) => println!("(unzip not available)"),
    }

    // Test extraction
    println!("\n--- Extraction Test ---");
    let extract_dir = "target/polyglot_large_extracted";
    let _ = fs::remove_dir_all(extract_dir);
    fs::create_dir_all(extract_dir)?;

    let start = Instant::now();
    match Command::new("unzip")
        .args(["-o", "-q", output_path, "-d", extract_dir])
        .output()
    {
        Ok(output) => {
            let elapsed = start.elapsed();
            if output.status.success() {
                println!("✓ Extraction successful in {:.2?}", elapsed);

                // Verify file sizes
                println!("\nExtracted files:");
                let mut total_extracted = 0usize;
                for entry in fs::read_dir(extract_dir)? {
                    let entry = entry?;
                    let meta = entry.metadata()?;
                    let size = meta.len() as usize;
                    total_extracted += size;
                    println!("  {:12} {:>10} bytes",
                             entry.file_name().to_string_lossy(), size);
                }
                println!("  {:12} {:>10} bytes", "TOTAL", total_extracted);

                if total_extracted == total_content {
                    println!("\n✓ All content extracted correctly!");
                } else {
                    println!("\n✗ Size mismatch! Expected {} bytes", total_content);
                }

                // Verify content of small file
                let small_content = fs::read_to_string(format!("{}/small.txt", extract_dir))?;
                if small_content == "Hello!" {
                    println!("✓ small.txt content verified");
                } else {
                    println!("✗ small.txt content mismatch: {:?}", small_content);
                }

            } else {
                println!("✗ Extraction failed:");
                println!("{}", String::from_utf8_lossy(&output.stderr));
            }
        }
        Err(_) => println!("(unzip not available for extraction)"),
    }

    println!("\n--- Done ---");
    println!("View as PNG: open {}", output_path);
    println!("Extract: unzip {}", output_path);

    Ok(())
}
