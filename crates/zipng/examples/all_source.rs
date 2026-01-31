//! Generate a sample containing ALL zipng source code.
//!
//! Large files (>60KB) are split into chunks to fit the IDAT boundary limit.

use indexmap::IndexMap;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;
use zipng::{panic, Files};

const MAX_CHUNK_SIZE: usize = 60_000; // 60000 byte limit (not 60KB)

fn main() -> Result<(), panic> {
    let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");

    let mut files: IndexMap<Vec<u8>, Vec<u8>> = IndexMap::new();
    let mut total_size = 0usize;
    let mut chunked_count = 0usize;

    // Walk all .rs files in src/
    for entry in WalkDir::new(&src_dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }

        let rel_path = path.strip_prefix(&src_dir).unwrap();
        let content = fs::read(path)?;
        total_size += content.len();

        if content.len() <= MAX_CHUNK_SIZE {
            // Fits as-is
            files.insert(
                format!("src/{}", rel_path.display()).into_bytes(),
                content,
            );
        } else {
            // Split into chunks
            chunked_count += 1;
            let chunk_count = (content.len() + MAX_CHUNK_SIZE - 1) / MAX_CHUNK_SIZE;
            println!(
                "Splitting {} ({} bytes) into {} chunks",
                rel_path.display(),
                content.len(),
                chunk_count
            );

            for (i, chunk) in content.chunks(MAX_CHUNK_SIZE).enumerate() {
                files.insert(
                    format!("src/{}.part{:02}", rel_path.display(), i + 1).into_bytes(),
                    chunk.to_vec(),
                );
            }
        }
    }

    let files_struct: Files = files.into();
    let polyglot = zipng::zipng(&files_struct);

    let output_path = "target/all_source.png";
    fs::write(output_path, &polyglot)?;

    println!("\nGenerated {}", output_path);
    println!("  {} source files", files_struct.files.len());
    println!("  {} files required chunking", chunked_count);
    println!("  {:.1} KiB total source", total_size as f64 / 1024.0);
    println!("  {:.1} KiB polyglot output", polyglot.len() as f64 / 1024.0);

    Ok(())
}
