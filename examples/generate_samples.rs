//! Generates a variety of sample polyglot PNG+ZIP files for testing.
//!
//! Creates files with different content types, sizes, and characteristics
//! to exercise all the palette selection and encoding paths.

use indexmap::IndexMap;
use std::fs;
use zipng::{panic, Files};

fn main() -> Result<(), panic> {
    fs::create_dir_all("target/samples")?;

    // Track generated files for summary
    let mut generated: Vec<(String, usize, String)> = Vec::new();

    // 1. Tiny single file
    generated.push(create_sample(
        "tiny_single",
        vec![("hello.txt", b"Hello, World!".to_vec())],
    )?);

    // 2. Multiple small text files
    generated.push(create_sample(
        "small_text_files",
        vec![
            ("readme.txt", b"This is a readme file.".to_vec()),
            ("license.txt", b"MIT License\n\nPermission is hereby granted...".to_vec()),
            ("changelog.md", b"# Changelog\n\n## v1.0.0\n- Initial release".to_vec()),
        ],
    )?);

    // 3. Binary data patterns
    generated.push(create_sample(
        "binary_patterns",
        vec![
            ("zeros.bin", vec![0u8; 1000]),
            ("ones.bin", vec![0xFFu8; 1000]),
            ("alternating.bin", (0..1000).map(|i| if i % 2 == 0 { 0x55 } else { 0xAA }).collect()),
            ("sequential.bin", (0..256).cycle().take(1000).map(|x| x as u8).collect()),
        ],
    )?);

    // 4. Source code files (from this project)
    generated.push(create_sample(
        "source_code",
        vec![
            ("Cargo.toml", include_bytes!("../Cargo.toml").to_vec()),
            ("src/lib.rs", include_bytes!("../src/lib.rs").to_vec()),
            ("README.md", include_bytes!("../README.md").to_vec()),
        ],
    )?);

    // 5. Nested directory structure
    generated.push(create_sample(
        "nested_directories",
        vec![
            ("project/src/main.rs", b"fn main() { println!(\"Hello\"); }".to_vec()),
            ("project/src/lib.rs", b"pub fn greet() -> &'static str { \"Hello\" }".to_vec()),
            ("project/tests/test_lib.rs", b"#[test] fn test_greet() { assert_eq!(lib::greet(), \"Hello\"); }".to_vec()),
            ("project/Cargo.toml", b"[package]\nname = \"demo\"\nversion = \"0.1.0\"".to_vec()),
            ("project/.gitignore", b"/target\n*.swp".to_vec()),
        ],
    )?);

    // 6. Long filenames
    generated.push(create_sample(
        "long_filenames",
        vec![
            ("this-is-a-very-long-filename-that-tests-our-row-width-calculation.txt", b"Content A".to_vec()),
            ("another/deeply/nested/path/with/many/directory/components/file.dat", b"Content B".to_vec()),
            ("short.txt", b"Content C".to_vec()),
        ],
    )?);

    // 7. JSON data
    generated.push(create_sample(
        "json_data",
        vec![
            ("config.json", br#"{"name": "test", "version": "1.0.0", "enabled": true}"#.to_vec()),
            ("data/users.json", br#"[{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]"#.to_vec()),
            ("data/settings.json", br#"{"theme": "dark", "language": "en", "notifications": true}"#.to_vec()),
        ],
    )?);

    // 8. Various sizes (to test different palette selections via hash)
    for size in [100, 500, 1000, 5000, 10000, 25000, 50000] {
        let name = format!("size_{size}");
        let content: Vec<u8> = (0..size).map(|i| ((i * 17 + 31) % 256) as u8).collect();
        generated.push(create_sample(
            &name,
            vec![("data.bin", content)],
        )?);
    }

    // 9. Random-looking data (different seeds for different palettes)
    for seed in [42u8, 123, 200, 7, 99] {
        let name = format!("random_seed_{seed}");
        let content: Vec<u8> = (0..5000)
            .map(|i| seed.wrapping_mul(i as u8).wrapping_add((i / 256) as u8))
            .collect();
        generated.push(create_sample(
            &name,
            vec![(&format!("random_{seed}.bin"), content)],
        )?);
    }

    // 10. Text with various encodings/content
    generated.push(create_sample(
        "text_variety",
        vec![
            ("ascii.txt", (32..127).cycle().take(500).map(|x| x as u8).collect()),
            ("lorem.txt", LOREM_IPSUM.as_bytes().to_vec()),
            ("numbers.txt", (0..1000).map(|i| format!("{i}\n")).collect::<String>().into_bytes()),
        ],
    )?);

    // 11. Empty files mixed with content
    generated.push(create_sample(
        "with_empty_files",
        vec![
            ("empty1.txt", vec![]),
            ("content.txt", b"This file has content".to_vec()),
            ("empty2.txt", vec![]),
            ("more_content.txt", b"More content here".to_vec()),
        ],
    )?);

    // 12. Single large file (under 60KB limit)
    generated.push(create_sample(
        "single_large",
        vec![("large.bin", (0..55000).map(|i| (i % 256) as u8).collect())],
    )?);

    // 13. Many small files
    let many_files: Vec<(String, Vec<u8>)> = (0..50)
        .map(|i| (format!("file_{i:03}.txt"), format!("Content of file {i}").into_bytes()))
        .collect();
    generated.push(create_sample(
        "many_small_files",
        many_files.iter().map(|(k, v)| (k.as_str(), v.clone())).collect(),
    )?);

    // 14. Mixed binary and text
    generated.push(create_sample(
        "mixed_content",
        vec![
            ("image_placeholder.png", vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]), // PNG sig
            ("document.txt", b"This is a text document.".to_vec()),
            ("data.bin", (0u16..256).map(|x| x as u8).collect()),
            ("script.sh", b"#!/bin/bash\necho \"Hello World\"".to_vec()),
        ],
    )?);

    // 15. Various path styles
    generated.push(create_sample(
        "path_styles",
        vec![
            ("normal.txt", b"Normal filename".to_vec()),
            ("with-dashes.txt", b"Dashes in name".to_vec()),
            ("with_underscores.txt", b"Underscores in name".to_vec()),
            ("CamelCase.txt", b"CamelCase name".to_vec()),
            ("UPPERCASE.TXT", b"Uppercase name".to_vec()),
        ],
    )?);

    // 16. Compressible vs incompressible data
    generated.push(create_sample(
        "compressibility",
        vec![
            ("highly_compressible.txt", "AAAA".repeat(1000).into_bytes()),
            ("moderately_compressible.txt", LOREM_IPSUM.repeat(5).into_bytes()),
            ("incompressible.bin", (0..4000).map(|i| ((i * 997) % 256) as u8).collect()),
        ],
    )?);

    // 17. Web assets
    generated.push(create_sample(
        "web_assets",
        vec![
            ("index.html", br#"<!DOCTYPE html><html><head><title>Test</title></head><body><h1>Hello</h1></body></html>"#.to_vec()),
            ("style.css", b"body { font-family: sans-serif; margin: 2em; }".to_vec()),
            ("script.js", b"console.log('Hello from polyglot!');".to_vec()),
            ("manifest.json", br#"{"name": "Test App", "version": "1.0"}"#.to_vec()),
        ],
    )?);

    // 18. Config files
    generated.push(create_sample(
        "config_files",
        vec![
            (".gitignore", b"/target\n*.log\n.env".to_vec()),
            (".editorconfig", b"root = true\n[*]\nindent_style = space\nindent_size = 4".to_vec()),
            ("Makefile", b"all:\n\tcargo build\n\ntest:\n\tcargo test".to_vec()),
            ("Dockerfile", b"FROM rust:latest\nWORKDIR /app\nCOPY . .\nRUN cargo build".to_vec()),
        ],
    )?);

    // 19. Large multi-file (tests IDAT boundary handling)
    generated.push(create_sample(
        "large_multi",
        vec![
            ("data1.bin", (0..30_000).map(|i| (i % 256) as u8).collect()),
            ("data2.bin", (0..30_000).map(|i| ((i * 7) % 256) as u8).collect()),
            ("data3.bin", (0..30_000).map(|i| ((i * 13) % 256) as u8).collect()),
        ],
    )?);

    // 20. Very large (200KB+, multiple IDAT boundaries)
    generated.push(create_sample(
        "very_large",
        (0..5).map(|i| {
            let name = format!("chunk_{i}.bin");
            let data: Vec<u8> = (0..40_000).map(|j| ((i * 17 + j * 7) % 256) as u8).collect();
            (name, data)
        }).map(|(n, d)| (n.leak() as &str, d)).collect(),
    )?);

    // 21. Poetry/Literature
    generated.push(create_sample(
        "literature",
        vec![
            ("shakespeare.txt", b"To be, or not to be, that is the question:\nWhether 'tis nobler in the mind to suffer\nThe slings and arrows of outrageous fortune,\nOr to take arms against a sea of troubles".to_vec()),
            ("dickinson.txt", b"Hope is the thing with feathers\nThat perches in the soul,\nAnd sings the tune without the words,\nAnd never stops at all".to_vec()),
            ("frost.txt", b"Two roads diverged in a yellow wood,\nAnd sorry I could not travel both\nAnd be one traveler, long I stood\nAnd looked down one as far as I could".to_vec()),
        ],
    )?);

    // 22. Code samples in different languages
    generated.push(create_sample(
        "polyglot_code",
        vec![
            ("hello.py", b"print('Hello, World!')".to_vec()),
            ("hello.js", b"console.log('Hello, World!');".to_vec()),
            ("hello.rb", b"puts 'Hello, World!'".to_vec()),
            ("hello.go", b"package main\nimport \"fmt\"\nfunc main() { fmt.Println(\"Hello, World!\") }".to_vec()),
            ("hello.rs", b"fn main() { println!(\"Hello, World!\"); }".to_vec()),
            ("hello.c", b"#include <stdio.h>\nint main() { printf(\"Hello, World!\\n\"); return 0; }".to_vec()),
        ],
    )?);

    // 23. Structured data formats
    generated.push(create_sample(
        "data_formats",
        vec![
            ("data.json", br#"{"items": [1, 2, 3], "nested": {"key": "value"}}"#.to_vec()),
            ("data.yaml", b"items:\n  - 1\n  - 2\n  - 3\nnested:\n  key: value".to_vec()),
            ("data.toml", b"[package]\nname = \"test\"\nversion = \"1.0.0\"\n\n[dependencies]\nserde = \"1.0\"".to_vec()),
            ("data.xml", b"<?xml version=\"1.0\"?><root><item>1</item><item>2</item></root>".to_vec()),
            ("data.csv", b"name,age,city\nAlice,30,NYC\nBob,25,LA\nCarol,35,Chicago".to_vec()),
        ],
    )?);

    // 24. Logs and output
    generated.push(create_sample(
        "logs",
        vec![
            ("access.log", (0..100).map(|i| format!("127.0.0.1 - - [01/Jan/2024:00:{i:02}:00] \"GET /page{i} HTTP/1.1\" 200 1234\n")).collect::<String>().into_bytes()),
            ("error.log", b"[ERROR] 2024-01-01 Failed to connect\n[WARN] 2024-01-01 Retrying...\n[INFO] 2024-01-01 Connected successfully".to_vec()),
            ("debug.log", (0..50).map(|i| format!("DEBUG: Processing item {i}\n")).collect::<String>().into_bytes()),
        ],
    )?);

    // 25. Certificates and keys (fake, for structure testing)
    generated.push(create_sample(
        "crypto_formats",
        vec![
            ("cert.pem", b"-----BEGIN CERTIFICATE-----\nMIIBkTCB+wIJAKHBfpegPjMCMA0GCSqGSIb3DQEBCwUA\n(fake certificate data for testing structure)\n-----END CERTIFICATE-----".to_vec()),
            ("key.pem", b"-----BEGIN PRIVATE KEY-----\n(fake key data - never put real keys in test files)\n-----END PRIVATE KEY-----".to_vec()),
            ("pubkey.pem", b"-----BEGIN PUBLIC KEY-----\n(fake public key data for structure testing)\n-----END PUBLIC KEY-----".to_vec()),
        ],
    )?);

    // Print summary
    println!("\nGenerated {} sample files:\n", generated.len());
    println!("{:<25} {:>10} {}", "Name", "Size", "Description");
    println!("{}", "-".repeat(75));
    for (name, size, desc) in &generated {
        println!("{:<25} {:>10} {}", name, format_size(*size), desc);
    }

    let total_size: usize = generated.iter().map(|(_, s, _)| s).sum();
    println!("{}", "-".repeat(75));
    println!("{:<25} {:>10}", "TOTAL", format_size(total_size));

    println!("\nAll files saved to target/samples/");
    println!("\nVerify with:");
    println!("  file target/samples/*.png");
    println!("  unzip -l target/samples/source_code.png");

    Ok(())
}

fn create_sample(
    name: &str,
    files: Vec<(&str, Vec<u8>)>,
) -> Result<(String, usize, String), panic> {
    let file_count = files.len();
    let total_content: usize = files.iter().map(|(_, v)| v.len()).sum();

    let index_map: IndexMap<Vec<u8>, Vec<u8>> = files
        .into_iter()
        .map(|(k, v)| (k.as_bytes().to_vec(), v))
        .collect();
    let files_struct: Files = index_map.into();

    let polyglot = zipng::zipng(&files_struct);
    let path = format!("target/samples/{name}.png");
    fs::write(&path, &polyglot)?;

    let desc = format!("{} files, {} content", file_count, format_size(total_content));
    Ok((name.to_string(), polyglot.len(), desc))
}

fn format_size(size: usize) -> String {
    if size >= 1024 * 1024 {
        format!("{:.1} MiB", size as f64 / (1024.0 * 1024.0))
    } else if size >= 1024 {
        format!("{:.1} KiB", size as f64 / 1024.0)
    } else {
        format!("{} B", size)
    }
}

const LOREM_IPSUM: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. \
Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim \
veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. \
Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat \
nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia \
deserunt mollit anim id est laborum.";
