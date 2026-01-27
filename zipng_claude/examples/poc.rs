//! Proof-of-concept demo - creates a polyglot PNG+ZIP with palette visualization.

use std::fs;
use indexmap::IndexMap;
use zipng::{panic, Files, palettes::oceanic::TOPO};

fn main() -> Result<(), panic> {
    fs::create_dir_all("target")?;

    // Create some test files
    let mut files = IndexMap::new();
    files.insert(
        b"palette.txt".to_vec(),
        format!("This polyglot uses the TOPO palette ({} bytes)", TOPO.len()).into_bytes(),
    );
    files.insert(
        b"readme.txt".to_vec(),
        b"This is a proof-of-concept polyglot PNG+ZIP file.".to_vec(),
    );

    let files: Files = files.into();
    let polyglot = zipng::zipng(&files);

    fs::write("target/poc.png", &polyglot)?;
    println!("Created target/poc.png ({} bytes)", polyglot.len());
    println!("  - View as PNG: open target/poc.png");
    println!("  - Extract as ZIP: unzip target/poc.png");

    Ok(())
}

#[test]
fn test() {
    main().unwrap()
}
