//! Test zipngbr (brotli-compressed polyglot).

use indexmap::IndexMap;
use std::fs;
use zipng::Files;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut files = IndexMap::new();
    files.insert(b"test.txt".to_vec(), b"Hello from brotli-compressed polyglot!".to_vec());
    let files: Files = files.into();

    // Create regular polyglot
    let polyglot = zipng::zipng(&files);
    println!("Regular polyglot: {} bytes", polyglot.len());

    // Create brotli-compressed polyglot
    let compressed = zipng::zipngbr(&files);
    println!("Brotli compressed: {} bytes", compressed.len());
    println!("Compression ratio: {:.1}%", (compressed.len() as f64 / polyglot.len() as f64) * 100.0);

    fs::write("target/polyglot.png.br", &compressed)?;
    println!("Saved to target/polyglot.png.br");

    Ok(())
}
