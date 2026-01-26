//! Generate diverse demo polyglot files showcasing various content types

use indexmap::IndexMap;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all("target/demos")?;

    // Demo 1: Mini web app
    {
        let mut files = IndexMap::new();
        files.insert(b"index.html".to_vec(), br#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Polyglot Web App</title>
    <link rel="stylesheet" href="style.css">
</head>
<body>
    <div class="container">
        <h1>* This page came from a PNG! *</h1>
        <p>The image you downloaded contains this entire website.</p>
        <pre id="output"></pre>
    </div>
    <script src="app.js"></script>
</body>
</html>"#.to_vec());

        files.insert(b"style.css".to_vec(), br#"* { box-sizing: border-box; margin: 0; padding: 0; }
body { 
    font-family: system-ui, -apple-system, sans-serif;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    min-height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
}
.container {
    background: white;
    padding: 2rem 3rem;
    border-radius: 12px;
    box-shadow: 0 20px 60px rgba(0,0,0,0.3);
    max-width: 600px;
}
h1 { color: #333; margin-bottom: 1rem; }
p { color: #666; margin-bottom: 1rem; }
pre { background: #f5f5f5; padding: 1rem; border-radius: 6px; overflow-x: auto; }"#.to_vec());

        files.insert(b"app.js".to_vec(), br#"// JavaScript from inside a PNG!
document.getElementById('output').textContent = 
    `Loaded at: ${new Date().toISOString()}\n` +
    `User Agent: ${navigator.userAgent.slice(0, 50)}...`;
console.log('This JS was extracted from a PNG file!');"#.to_vec());

        files.insert(b"data.json".to_vec(), br#"{
    "name": "polyglot-webapp",
    "version": "1.0.0",
    "description": "A web app living inside a PNG image",
    "files": ["index.html", "style.css", "app.js"]
}"#.to_vec());

        let polyglot = zipng::zipng(&files.into());
        fs::write("target/demos/webapp.png", &polyglot)?;
        println!("Created webapp.png ({} bytes) - complete web application", polyglot.len());
    }

    // Demo 2: Binary formats (simulated)
    {
        let mut files = IndexMap::new();
        
        // Fake PDF header + content
        let mut pdf = b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n".to_vec();
        pdf.extend(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");
        pdf.extend(b"(This is a simulated PDF structure)\n");
        files.insert(b"document.pdf".to_vec(), pdf);
        
        // SQLite database header + fake tables
        let mut db = b"SQLite format 3\x00".to_vec();
        db.resize(100, 0);
        db.extend(b"CREATE TABLE users (id INTEGER, name TEXT);");
        db.extend(b"INSERT INTO users VALUES (1, 'Alice'), (2, 'Bob');");
        files.insert(b"data.sqlite".to_vec(), db);
        
        // Tar-like structure
        let mut tar = Vec::new();
        tar.extend(b"file1.txt");
        tar.resize(100, 0);
        tar.extend(b"0000644\x000000000\x000000000\x00");
        tar.extend(b"Contents of file1");
        files.insert(b"archive.tar".to_vec(), tar);

        // Raw binary data with patterns
        let binary: Vec<u8> = (0..1000)
            .map(|i| {
                let x = (i as f64 * 0.1).sin();
                ((x * 127.0 + 128.0) as u8)
            })
            .collect();
        files.insert(b"waveform.bin".to_vec(), binary);

        let polyglot = zipng::zipng(&files.into());
        fs::write("target/demos/binary_mix.png", &polyglot)?;
        println!("Created binary_mix.png ({} bytes) - various binary formats", polyglot.len());
    }

    // Demo 3: Source code archive
    {
        let mut files = IndexMap::new();
        
        files.insert(b"src/main.rs".to_vec(), br#"//! A Rust program stored in a PNG
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("Hello from PNG-embedded Rust!");
    println!("Arguments: {:?}", args);
    
    let result = fibonacci(10);
    println!("Fibonacci(10) = {}", result);
}

fn fibonacci(n: u32) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fib() {
        assert_eq!(fibonacci(0), 0);
        assert_eq!(fibonacci(1), 1);
        assert_eq!(fibonacci(10), 55);
    }
}"#.to_vec());

        files.insert(b"src/lib.rs".to_vec(), br#"//! Library code from a PNG
pub mod utils {
    pub fn greet(name: &str) -> String {
        format!("Hello, {}!", name)
    }
}
"#.to_vec());

        files.insert(b"Cargo.toml".to_vec(), br#"[package]
name = "png-embedded"
version = "0.1.0"
edition = "2021"

[dependencies]
# This Cargo.toml came from a PNG file!
"#.to_vec());

        files.insert(b"README.md".to_vec(), br#"# PNG-Embedded Project

This entire Rust project is stored inside a PNG image!

## Usage

```bash
# Extract the image
unzip image.png -d project/
cd project/
cargo run
```

## How it works

The PNG image is also a valid ZIP archive. The pixel data
contains the compressed source files.
"#.to_vec());

        let polyglot = zipng::zipng(&files.into());
        fs::write("target/demos/rust_project.png", &polyglot)?;
        println!("Created rust_project.png ({} bytes) - complete Rust project", polyglot.len());
    }

    // Demo 4: Multi-language source
    {
        let mut files = IndexMap::new();
        
        files.insert(b"hello.py".to_vec(), br#"#!/usr/bin/env python3
"""Python from a PNG!"""

def main():
    print("Hello from PNG-embedded Python!")
    for i in range(5):
        print(f"  Line {i + 1}")

if __name__ == "__main__":
    main()
"#.to_vec());

        files.insert(b"hello.js".to_vec(), br#"#!/usr/bin/env node
// JavaScript from a PNG!

console.log("Hello from PNG-embedded JavaScript!");
const data = { source: "PNG", format: "polyglot" };
console.log(JSON.stringify(data, null, 2));
"#.to_vec());

        files.insert(b"hello.rb".to_vec(), br#"#!/usr/bin/env ruby
# Ruby from a PNG!

puts "Hello from PNG-embedded Ruby!"
5.times { |i| puts "  Line #{i + 1}" }
"#.to_vec());

        files.insert(b"hello.sh".to_vec(), br#"#!/bin/bash
# Shell script from a PNG!

echo "Hello from PNG-embedded Bash!"
for i in 1 2 3 4 5; do
    echo "  Line $i"
done
"#.to_vec());

        files.insert(b"hello.c".to_vec(), br#"/* C from a PNG! */
#include <stdio.h>

int main() {
    printf("Hello from PNG-embedded C!\n");
    for (int i = 0; i < 5; i++) {
        printf("  Line %d\n", i + 1);
    }
    return 0;
}
"#.to_vec());

        let polyglot = zipng::zipng(&files.into());
        fs::write("target/demos/polyglot_code.png", &polyglot)?;
        println!("Created polyglot_code.png ({} bytes) - multi-language hello world", polyglot.len());
    }

    // Demo 5: Config file collection
    {
        let mut files = IndexMap::new();

        files.insert(b".gitignore".to_vec(), br#"# Git ignore from PNG
target/
*.log
.env
node_modules/
"#.to_vec());

        files.insert(b".editorconfig".to_vec(), br#"root = true

[*]
indent_style = space
indent_size = 4
end_of_line = lf
charset = utf-8
trim_trailing_whitespace = true
insert_final_newline = true
"#.to_vec());

        files.insert(b"config.yaml".to_vec(), br#"# YAML config from PNG
server:
  host: localhost
  port: 8080
  
database:
  url: postgres://localhost/mydb
  pool_size: 10
  
features:
  - authentication
  - logging
  - caching
"#.to_vec());

        files.insert(b"config.toml".to_vec(), br#"# TOML config from PNG
[server]
host = "localhost"
port = 8080

[database]
url = "postgres://localhost/mydb"
pool_size = 10
"#.to_vec());

        files.insert(b".env.example".to_vec(), br#"# Environment variables template
DATABASE_URL=postgres://user:pass@localhost/db
SECRET_KEY=your-secret-key-here
DEBUG=false
"#.to_vec());

        let polyglot = zipng::zipng(&files.into());
        fs::write("target/demos/config_bundle.png", &polyglot)?;
        println!("Created config_bundle.png ({} bytes) - config file collection", polyglot.len());
    }

    // Demo 6: Large mixed content (stress test)
    {
        let mut files = IndexMap::new();

        // Large text file
        let lorem = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. ";
        let large_text: Vec<u8> = lorem.repeat(200).into_bytes();
        files.insert(b"large_text.txt".to_vec(), large_text);

        // Large binary pattern
        let large_binary: Vec<u8> = (0..20_000)
            .map(|i| ((i * 7 + i / 256) % 256) as u8)
            .collect();
        files.insert(b"large_binary.dat".to_vec(), large_binary);

        // Gradient image data (raw pixels)
        let gradient: Vec<u8> = (0..10_000)
            .map(|i| (i % 256) as u8)
            .collect();
        files.insert(b"gradient.raw".to_vec(), gradient);

        // Some smaller files too
        files.insert(b"small.txt".to_vec(), b"Small file for contrast".to_vec());
        files.insert(b"tiny.txt".to_vec(), b"Tiny!".to_vec());

        let polyglot = zipng::zipng(&files.into());
        fs::write("target/demos/mixed_large.png", &polyglot)?;
        println!("Created mixed_large.png ({} bytes) - large mixed content", polyglot.len());
    }

    println!("\nAll demos created in target/demos/");
    println!("\nTo explore:");
    println!("  file target/demos/*.png      # Verify they're valid PNGs");
    println!("  unzip -l target/demos/*.png  # List ZIP contents");
    println!("  unzip -d /tmp/demo target/demos/webapp.png  # Extract webapp");
    
    Ok(())
}
