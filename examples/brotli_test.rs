//! Test brotli compression feature (placeholder).
//!
//! Note: Brotli-compressed polyglot (zipngbr) is not yet implemented in the new API.
//! This example demonstrates the regular polyglot creation instead.

use indexmap::IndexMap;
use std::fs;
use zipng::Files;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all("target")?;

    let mut files = IndexMap::new();
    files.insert(b"test.txt".to_vec(), b"Hello from polyglot!".to_vec());
    let files: Files = files.into();

    // Create regular polyglot
    let polyglot = zipng::zipng(&files);
    println!("Regular polyglot: {} bytes", polyglot.len());

    fs::write("target/brotli_test.png", &polyglot)?;
    println!("Saved to target/brotli_test.png");

    // Note: Brotli compression (zipngbr) would go here when implemented
    println!("\nNote: Brotli-compressed polyglot (zipngbr) is not yet implemented.");

    Ok(())
}
