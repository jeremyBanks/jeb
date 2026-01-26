//! Generate sample polyglot files

use indexmap::IndexMap;
use std::fs;
use zipng::Files;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all("target/samples")?;

    // Sample 1: Simple greeting
    {
        let mut files = IndexMap::new();
        files.insert(b"greeting.txt".to_vec(), b"Hello! This file is both a PNG and a ZIP.".to_vec());
        let polyglot = zipng::zipng(&files.into());
        fs::write("target/samples/greeting.png", &polyglot)?;
        println!("Created greeting.png ({} bytes)", polyglot.len());
    }

    // Sample 2: A tiny website
    {
        let mut files = IndexMap::new();
        files.insert(b"index.html".to_vec(), br#"<!DOCTYPE html>
<html>
<head><title>Polyglot Page</title></head>
<body>
<h1>This HTML came from a PNG!</h1>
<p>The image you're looking at IS the zip containing this file.</p>
</body>
</html>"#.to_vec());
        files.insert(b"style.css".to_vec(), b"body { font-family: sans-serif; background: #f0f0f0; }".to_vec());
        let polyglot = zipng::zipng(&files.into());
        fs::write("target/samples/website.png", &polyglot)?;
        println!("Created website.png ({} bytes)", polyglot.len());
    }

    // Sample 3: JSON data
    {
        let mut files = IndexMap::new();
        files.insert(b"data.json".to_vec(), br#"{
  "name": "polyglot",
  "type": "PNG+ZIP",
  "magic": "This JSON lives inside an image!"
}"#.to_vec());
        let polyglot = zipng::zipng(&files.into());
        fs::write("target/samples/data.png", &polyglot)?;
        println!("Created data.png ({} bytes)", polyglot.len());
    }

    // Sample 4: Source code (meta!)
    {
        let mut files = IndexMap::new();
        files.insert(b"polyglot.rs".to_vec(), br#"// This Rust code is stored inside a PNG image!
fn main() {
    println!("I am a polyglot file.");
    println!("View me as PNG: I'm an image.");
    println!("Unzip me: I contain this source code!");
}
"#.to_vec());
        let polyglot = zipng::zipng(&files.into());
        fs::write("target/samples/code.png", &polyglot)?;
        println!("Created code.png ({} bytes)", polyglot.len());
    }

    // Sample 5: Larger file to see more visual pattern
    {
        let mut files = IndexMap::new();
        let lorem = b"Lorem ipsum dolor sit amet, consectetur adipiscing elit. \
Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. \
Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris. ";
        let mut content = Vec::new();
        for _ in 0..5 {
            content.extend_from_slice(lorem);
        }
        files.insert(b"lorem.txt".to_vec(), content);
        let polyglot = zipng::zipng(&files.into());
        fs::write("target/samples/lorem.png", &polyglot)?;
        println!("Created lorem.png ({} bytes)", polyglot.len());
    }

    // Sample 6: Large multi-file archive (tests IDAT boundary handling)
    // Total content > 65KB to verify boundary padding works
    {
        let mut files = IndexMap::new();
        // Create 30KB files with varying patterns
        let data1: Vec<u8> = (0..30_000).map(|i| (i % 256) as u8).collect();
        let data2: Vec<u8> = (0..30_000).map(|i| ((i * 7) % 256) as u8).collect();
        let data3: Vec<u8> = (0..30_000).map(|i| ((i * 13) % 256) as u8).collect();

        files.insert(b"data1.bin".to_vec(), data1);
        files.insert(b"data2.bin".to_vec(), data2);
        files.insert(b"data3.bin".to_vec(), data3);

        let polyglot = zipng::zipng(&files.into());
        fs::write("target/samples/large.png", &polyglot)?;
        println!("Created large.png ({} bytes) - tests boundary handling", polyglot.len());
    }

    // Sample 7: Very large archive (200KB+, multiple IDAT boundaries)
    {
        let mut files = IndexMap::new();
        for i in 0..5 {
            let data: Vec<u8> = (0..40_000).map(|j| ((i * 17 + j * 7) % 256) as u8).collect();
            files.insert(format!("chunk_{}.bin", i).into_bytes(), data);
        }

        let polyglot = zipng::zipng(&files.into());
        fs::write("target/samples/verylarge.png", &polyglot)?;
        println!("Created verylarge.png ({} bytes) - 200KB+ content", polyglot.len());
    }

    println!("\nAll samples created in target/samples/");
    Ok(())
}
