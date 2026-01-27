//! ZipNG example - embeds project files into a polyglot PNG+ZIP.

use indexmap::IndexMap;
use std::fs;
use zipng::{panic, Files};

fn main() -> Result<(), panic> {
    fs::create_dir_all("target")?;

    // Embed some project files
    let files: [(&[u8], &[u8]); 2] = [
        (
            b"assets/Cargo.toml".as_ref(),
            include_bytes!("../Cargo.toml"),
        ),
        (
            b"assets/README.md".as_ref(),
            include_bytes!("../README.md"),
        ),
    ];
    let files = IndexMap::from_iter(files.iter().map(|(k, v)| (k.to_vec(), v.to_vec())));
    let files: Files = files.into();

    // Create polyglot
    let polyglot = zipng::zipng(&files);

    fs::write("target/zipng_example.png", &polyglot)?;
    println!("Created target/zipng_example.png ({} bytes)", polyglot.len());
    println!("Contains:");
    println!("  - assets/Cargo.toml");
    println!("  - assets/README.md");
    println!("\nVerify with: unzip -l target/zipng_example.png");

    Ok(())
}
