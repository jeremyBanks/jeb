//! Test the polyglot PNG+ZIP functionality.
//!
//! This example creates a polyglot file and verifies it works as both formats.

use indexmap::IndexMap;
use std::fs;
use std::process::Command;
use zipng::Files;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating polyglot PNG+ZIP file...\n");

    // Create some test files
    let mut files = IndexMap::new();
    files.insert(
        b"hello.txt".to_vec(),
        b"Hello, World! This is a test file inside a polyglot.".to_vec(),
    );
    files.insert(
        b"readme.md".to_vec(),
        b"# Polyglot Test\n\nThis ZIP is also a valid PNG image!".to_vec(),
    );
    files.insert(
        b"data/numbers.txt".to_vec(),
        b"1\n2\n3\n4\n5\n6\n7\n8\n9\n10".to_vec(),
    );

    let files: Files = files.into();

    // Generate the polyglot
    let polyglot_data = zipng::zipng(&files);

    // Save it
    let output_path = "target/polyglot_test.png";
    fs::create_dir_all("target")?;
    fs::write(output_path, &polyglot_data)?;
    println!("✓ Created {} ({} bytes)", output_path, polyglot_data.len());

    // Verify PNG structure
    println!("\n--- PNG Verification ---");
    if &polyglot_data[0..8] == b"\x89PNG\r\n\x1A\n" {
        println!("✓ Valid PNG signature");
    } else {
        println!("✗ Invalid PNG signature");
    }

    // Find PNG chunks
    let mut pos = 8;
    while pos < polyglot_data.len() {
        if pos + 8 > polyglot_data.len() {
            break;
        }
        let length = u32::from_be_bytes([
            polyglot_data[pos],
            polyglot_data[pos + 1],
            polyglot_data[pos + 2],
            polyglot_data[pos + 3],
        ]) as usize;
        let chunk_type = String::from_utf8_lossy(&polyglot_data[pos + 4..pos + 8]);
        println!("  PNG chunk: {} ({} bytes)", chunk_type, length);

        if chunk_type == "IEND" {
            println!("  (PNG data ends at byte {})", pos + 12);
            break;
        }

        pos += 12 + length; // 4 (length) + 4 (type) + data + 4 (crc)
    }

    // Verify ZIP structure
    println!("\n--- ZIP Verification ---");

    // Look for ZIP end of central directory
    let eocd_sig = b"PK\x05\x06";
    if let Some(eocd_pos) = polyglot_data
        .windows(4)
        .rposition(|w| w == eocd_sig)
    {
        println!("✓ Found ZIP EOCD at offset {}", eocd_pos);

        // Parse EOCD
        let num_entries = u16::from_le_bytes([
            polyglot_data[eocd_pos + 8],
            polyglot_data[eocd_pos + 9],
        ]);
        let cd_size = u32::from_le_bytes([
            polyglot_data[eocd_pos + 12],
            polyglot_data[eocd_pos + 13],
            polyglot_data[eocd_pos + 14],
            polyglot_data[eocd_pos + 15],
        ]);
        let cd_offset = u32::from_le_bytes([
            polyglot_data[eocd_pos + 16],
            polyglot_data[eocd_pos + 17],
            polyglot_data[eocd_pos + 18],
            polyglot_data[eocd_pos + 19],
        ]);
        println!("  {} entries, central directory at offset {}, size {}", num_entries, cd_offset, cd_size);

        // Parse central directory entries
        let mut cd_pos = cd_offset as usize;
        for i in 0..num_entries {
            if cd_pos + 46 > polyglot_data.len() {
                println!("  Warning: Central directory truncated");
                break;
            }
            if &polyglot_data[cd_pos..cd_pos + 4] != b"PK\x01\x02" {
                println!("  Warning: Invalid central directory signature at {}", cd_pos);
                break;
            }
            let name_len = u16::from_le_bytes([
                polyglot_data[cd_pos + 28],
                polyglot_data[cd_pos + 29],
            ]) as usize;
            let local_offset = u32::from_le_bytes([
                polyglot_data[cd_pos + 42],
                polyglot_data[cd_pos + 43],
                polyglot_data[cd_pos + 44],
                polyglot_data[cd_pos + 45],
            ]);
            let name = String::from_utf8_lossy(&polyglot_data[cd_pos + 46..cd_pos + 46 + name_len]);
            println!("  Entry {}: \"{}\" at local offset {}", i, name, local_offset);

            // Verify local header exists
            if local_offset as usize + 4 <= polyglot_data.len() {
                let local_sig = &polyglot_data[local_offset as usize..local_offset as usize + 4];
                if local_sig == b"PK\x03\x04" {
                    println!("    ✓ Local header signature valid");
                } else {
                    println!("    ✗ Local header signature invalid: {:?}", local_sig);
                }
            }

            cd_pos += 46 + name_len;
        }
    } else {
        println!("✗ ZIP EOCD not found");
    }

    // Try using system tools to verify
    println!("\n--- System Tool Verification ---");

    // Test with `file` command
    match Command::new("file").arg(output_path).output() {
        Ok(output) => {
            let result = String::from_utf8_lossy(&output.stdout);
            println!("file: {}", result.trim());
        }
        Err(_) => println!("(file command not available)"),
    }

    // Test with `unzip -l`
    match Command::new("unzip").args(["-l", output_path]).output() {
        Ok(output) => {
            if output.status.success() {
                println!("\nunzip -l output:");
                println!("{}", String::from_utf8_lossy(&output.stdout));
            } else {
                println!("unzip failed: {}", String::from_utf8_lossy(&output.stderr));
            }
        }
        Err(_) => println!("(unzip command not available)"),
    }

    // Test extraction
    let extract_dir = "target/polyglot_extracted";
    let _ = fs::remove_dir_all(extract_dir);
    fs::create_dir_all(extract_dir)?;

    match Command::new("unzip")
        .args(["-o", output_path, "-d", extract_dir])
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                println!("✓ Successfully extracted files to {}/", extract_dir);

                // List extracted files
                for entry in fs::read_dir(extract_dir)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_file() {
                        let content = fs::read_to_string(&path).unwrap_or_else(|_| "(binary)".to_string());
                        println!("  {:?}: {} bytes", path.file_name().unwrap(), content.len());
                    }
                }
            } else {
                println!("✗ Extraction failed: {}", String::from_utf8_lossy(&output.stderr));
            }
        }
        Err(_) => println!("(unzip command not available for extraction test)"),
    }

    println!("\n--- Done ---");
    println!("The polyglot file is at: {}", output_path);
    println!("View as PNG: open {} (or use any image viewer)", output_path);
    println!("Extract as ZIP: unzip {}", output_path);

    Ok(())
}
