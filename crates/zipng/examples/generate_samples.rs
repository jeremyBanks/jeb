//! Generates sample polyglot PNG+ZIP files using REAL project files.
//!
//! Uses actual source code, fonts, configs, and binary assets from the fic.is
//! project to create diverse, realistic sample archives.

use indexmap::IndexMap;
use std::fs;
use zipng::{panic, Files};
use zipng::polyglot::{assert_valid_polyglot_with, Expectations};

fn main() -> Result<(), panic> {
    fs::create_dir_all("target/samples")?;

    let mut generated: Vec<(String, usize, String)> = Vec::new();

    // 1. Font assets - PNG bitmaps + JSON metadata
    generated.push(create_sample(
        "bitmap_fonts",
        vec![
            ("micro.png", include_bytes!("../src/text/micro.png").to_vec()),
            ("micro.json", include_bytes!("../src/text/micro.json").to_vec()),
            ("mini.png", include_bytes!("../src/text/mini.png").to_vec()),
            ("mini.json", include_bytes!("../src/text/mini.json").to_vec()),
            ("swiss.png", include_bytes!("../src/text/swiss.png").to_vec()),
            ("swiss.json", include_bytes!("../src/text/swiss.json").to_vec()),
        ],
    )?);

    // 2. All font files together
    generated.push(create_sample(
        "all_fonts",
        vec![
            ("micro.png", include_bytes!("../src/text/micro.png").to_vec()),
            ("micro.json", include_bytes!("../src/text/micro.json").to_vec()),
            ("mini.png", include_bytes!("../src/text/mini.png").to_vec()),
            ("mini.json", include_bytes!("../src/text/mini.json").to_vec()),
            ("sixth.png", include_bytes!("../src/text/sixth.png").to_vec()),
            ("sixth.json", include_bytes!("../src/text/sixth.json").to_vec()),
            ("monte.png", include_bytes!("../src/text/monte.png").to_vec()),
            ("monte.json", include_bytes!("../src/text/monte.json").to_vec()),
            ("sky.png", include_bytes!("../src/text/sky.png").to_vec()),
            ("sky.json", include_bytes!("../src/text/sky.json").to_vec()),
            ("sugimori.png", include_bytes!("../src/text/sugimori.png").to_vec()),
            ("sugimori.json", include_bytes!("../src/text/sugimori.json").to_vec()),
            ("swiss.png", include_bytes!("../src/text/swiss.png").to_vec()),
            ("swiss.json", include_bytes!("../src/text/swiss.json").to_vec()),
        ],
    )?);

    // 3. Core Rust source files
    generated.push(create_sample(
        "rust_core",
        vec![
            ("lib.rs", include_bytes!("../src/lib.rs").to_vec()),
            ("checksums.rs", include_bytes!("../src/checksums.rs").to_vec()),
            ("deflate.rs", include_bytes!("../src/deflate.rs").to_vec()),
            ("zlib.rs", include_bytes!("../src/zlib.rs").to_vec()),
        ],
    )?);

    // 4. PNG module source
    generated.push(create_sample(
        "png_module",
        vec![
            ("png/mod.rs", include_bytes!("../src/png.rs").to_vec()),
            ("png/data.rs", include_bytes!("../src/png/data.rs").to_vec()),
            ("png/to_png.rs", include_bytes!("../src/png/to_png.rs").to_vec()),
            ("png/write_png.rs", include_bytes!("../src/png/write_png.rs").to_vec()),
            ("png/sizes.rs", include_bytes!("../src/png/sizes.rs").to_vec()),
        ],
    )?);

    // 5. ZIP module source
    generated.push(create_sample(
        "zip_module",
        vec![
            ("zip/mod.rs", include_bytes!("../src/zip.rs").to_vec()),
            ("zip/data.rs", include_bytes!("../src/zip/data.rs").to_vec()),
            ("zip/to_zip.rs", include_bytes!("../src/zip/to_zip.rs").to_vec()),
            ("zip/write_zip.rs", include_bytes!("../src/zip/write_zip.rs").to_vec()),
            ("zip/configuration.rs", include_bytes!("../src/zip/configuration.rs").to_vec()),
        ],
    )?);

    // 6. Polyglot module support files (mod.rs is too large at 65KB)
    generated.push(create_sample(
        "polyglot_support",
        vec![
            ("polyglot/fonts.rs", include_bytes!("../src/polyglot/fonts.rs").to_vec()),
            ("polyglot/validate.rs", include_bytes!("../src/polyglot/validate.rs").to_vec()),
        ],
    )?);

    // 7. Palette definitions
    generated.push(create_sample(
        "palettes",
        vec![
            ("palettes/mod.rs", include_bytes!("../src/png/palettes.rs").to_vec()),
            ("palettes/viridis.rs", include_bytes!("../src/png/palettes/viridis.rs").to_vec()),
            ("palettes/singles.rs", include_bytes!("../src/png/palettes/singles.rs").to_vec()),
            ("palettes/diagnostic.rs", include_bytes!("../src/png/palettes/diagnostic.rs").to_vec()),
            ("palettes/mappings.rs", include_bytes!("../src/png/palettes/mappings.rs").to_vec()),
        ],
    )?);

    // 8. Project configuration
    generated.push(create_sample(
        "project_config",
        vec![
            ("Cargo.toml", include_bytes!("../Cargo.toml").to_vec()),
            ("Cargo.lock", include_bytes!("../Cargo.lock").to_vec()),
            ("rustfmt.toml", include_bytes!("../rustfmt.toml").to_vec()),
            ("README.md", include_bytes!("../README.md").to_vec()),
            ("CLAUDE.md", include_bytes!("../CLAUDE.md").to_vec()),
        ],
    )?);

    // 9. Documentation
    generated.push(create_sample(
        "documentation",
        vec![
            ("docs/polyglot-architecture.md", include_bytes!("../docs/polyglot-architecture.md").to_vec()),
            ("docs/polyglot-constraints.md", include_bytes!("../docs/polyglot-constraints.md").to_vec()),
        ],
    )?);

    // 10. Test data files (binary)
    generated.push(create_sample(
        "test_data",
        vec![
            ("test_data/zip.zip", include_bytes!("../test_data/zip.zip").to_vec()),
            ("test_data/zip.htm", include_bytes!("../test_data/zip.htm").to_vec()),
            ("test_data/poc.png.zip", include_bytes!("../test_data/poc.png.zip").to_vec()),
        ],
    )?);

    // 11. Example Rust files
    generated.push(create_sample(
        "examples",
        vec![
            ("examples/zipng.rs", include_bytes!("zipng.rs").to_vec()),
            ("examples/rgb.rs", include_bytes!("rgb.rs").to_vec()),
            ("examples/indexed8bit.rs", include_bytes!("indexed8bit.rs").to_vec()),
            ("examples/greyscale8bit.rs", include_bytes!("greyscale8bit.rs").to_vec()),
        ],
    )?);

    // 12. Fiction spine JSON data (from parent project)
    generated.push(create_sample(
        "fiction_spines",
        vec![
            ("RYL0035858.json", include_bytes!("../../../data/spines/RYL0035858.json").to_vec()),
            ("RYL0036950.json", include_bytes!("../../../data/spines/RYL0036950.json").to_vec()),
            ("RYL0048012.json", include_bytes!("../../../data/spines/RYL0048012.json").to_vec()),
            ("index.json", include_bytes!("../../../data/spines/index.json").to_vec()),
        ],
    )?);

    // 13. Deno/TypeScript source
    generated.push(create_sample(
        "deno_source",
        vec![
            ("main.ts", include_bytes!("../../../deno/main.ts").to_vec()),
            ("fresh.gen.ts", include_bytes!("../../../deno/fresh.gen.ts").to_vec()),
            ("twind.config.ts", include_bytes!("../../../deno/twind.config.ts").to_vec()),
            ("deno.json", include_bytes!("../../../deno/deno.json").to_vec()),
        ],
    )?);

    // 14. Deno routes (TSX)
    generated.push(create_sample(
        "deno_routes",
        vec![
            ("routes/_404.ts", include_bytes!("../../../deno/routes/_404.ts").to_vec()),
            ("routes/_500.ts", include_bytes!("../../../deno/routes/_500.ts").to_vec()),
            ("routes/_middleware.ts", include_bytes!("../../../deno/routes/_middleware.ts").to_vec()),
        ],
    )?);

    // 15. Web fonts (WOFF2 binary)
    generated.push(create_sample(
        "web_fonts",
        vec![
            ("fonts/sans400.woff2", include_bytes!("../../../deno/static/fonts/sans400.woff2").to_vec()),
            ("fonts/sans700.woff2", include_bytes!("../../../deno/static/fonts/sans700.woff2").to_vec()),
        ],
    )?);

    // 16. Static web assets
    generated.push(create_sample(
        "static_assets",
        vec![
            ("icon.svg", include_bytes!("../../../deno/static/icon.svg").to_vec()),
            ("cover.png", include_bytes!("../../../deno/static/cover.png").to_vec()),
        ],
    )?);

    // 17. Parent project Rust source
    generated.push(create_sample(
        "fic_source",
        vec![
            ("src/lib.rs", include_bytes!("../../../src/lib.rs").to_vec()),
            ("src/backend.rs", include_bytes!("../../../src/backend.rs").to_vec()),
            ("src/engine.rs", include_bytes!("../../../src/engine.rs").to_vec()),
            ("src/query.rs", include_bytes!("../../../src/query.rs").to_vec()),
        ],
    )?);

    // 18. Mixed binary + text from project
    generated.push(create_sample(
        "mixed_project",
        vec![
            ("icon.png", include_bytes!("../../../icon.png").to_vec()),
            ("README.md", include_bytes!("../../../README.md").to_vec()),
            ("Cargo.toml", include_bytes!("../../../Cargo.toml").to_vec()),
            ("CLAUDE.md", include_bytes!("../../../CLAUDE.md").to_vec()),
        ],
    )?);

    // 19. VSCode settings (JSON)
    generated.push(create_sample(
        "vscode_configs",
        vec![
            (".vscode/settings.json", include_bytes!("../.vscode/settings.json").to_vec()),
            (".vscode/launch.json", include_bytes!("../.vscode/launch.json").to_vec()),
        ],
    )?);

    // 20. Script files
    generated.push(create_sample(
        "scripts",
        vec![
            ("scripts/debug_polyglot.rs", include_bytes!("../scripts/debug_polyglot.rs").to_vec()),
            ("scripts/check_font.rs", include_bytes!("../scripts/check_font.rs").to_vec()),
            ("scripts/test_kerning.rs", include_bytes!("../scripts/test_kerning.rs").to_vec()),
        ],
    )?);

    // 21. IO module
    generated.push(create_sample(
        "io_module",
        vec![
            ("io/mod.rs", include_bytes!("../src/io.rs").to_vec()),
            ("io/alignment.rs", include_bytes!("../src/io/alignment.rs").to_vec()),
        ],
    )?);

    // 22. Exploration docs
    generated.push(create_sample(
        "exploration_docs",
        vec![
            ("idat-boundary-analysis.md", include_bytes!("../docs/exploration/idat-boundary-analysis.md").to_vec()),
            ("variable-width-math.md", include_bytes!("../docs/exploration/variable-width-math.md").to_vec()),
            ("variable-width-plan.md", include_bytes!("../docs/exploration/variable-width-plan.md").to_vec()),
            ("wider-images-analysis.md", include_bytes!("../docs/exploration/wider-images-analysis.md").to_vec()),
        ],
    )?);

    // 23. Deno XML utilities
    generated.push(create_sample(
        "deno_xml",
        vec![
            ("xml/rss.ts", include_bytes!("../../../deno/xml/rss.ts").to_vec()),
            ("xml/xml.ts", include_bytes!("../../../deno/xml/xml.ts").to_vec()),
        ],
    )?);

    // 24. Deno components
    generated.push(create_sample(
        "deno_components",
        vec![
            ("components/Page.tsx", include_bytes!("../../../deno/components/Page.tsx").to_vec()),
            ("utils/data.ts", include_bytes!("../../../deno/utils/data.ts").to_vec()),
        ],
    )?);

    // 25. Cargo.lock is a good large text file
    generated.push(create_sample(
        "large_single",
        vec![
            ("Cargo.lock", include_bytes!("../Cargo.lock").to_vec()),
        ],
    )?);

    // === EDGE CASES ===

    // 26. Many small files (tests width optimization)
    let many_files: Vec<(&str, Vec<u8>)> = (0..50)
        .map(|i| {
            let name: &'static str = Box::leak(format!("file_{i:03}.txt").into_boxed_str());
            let content = format!("Content of file {i}").into_bytes();
            (name, content)
        })
        .collect();
    generated.push(create_sample("many_small_files", many_files)?);

    // 27. Absurdly long filenames
    generated.push(create_sample(
        "long_filenames",
        vec![
            ("this-is-an-extremely-long-filename-that-tests-row-width-calculation-and-label-truncation.txt", b"A".to_vec()),
            ("another/very/deeply/nested/directory/structure/with/many/path/components/file.dat", b"B".to_vec()),
            ("SCREAMING_SNAKE_CASE_FILENAME_THAT_GOES_ON_AND_ON_AND_ON.TXT", b"C".to_vec()),
            ("short.txt", b"D".to_vec()),
        ],
    )?);

    // 28. Empty files mixed with content
    generated.push(create_sample(
        "with_empty_files",
        vec![
            ("empty1.txt", vec![]),
            ("has_content.txt", b"This file has content".to_vec()),
            ("empty2.txt", vec![]),
            ("also_has_content.txt", b"More content here".to_vec()),
            ("empty3.txt", vec![]),
        ],
    )?);

    // 29. Single tiny file
    generated.push(create_sample(
        "tiny_single",
        vec![("hi.txt", b"Hi".to_vec())],
    )?);

    // 30. Unicode and special characters in content (ASCII filenames)
    generated.push(create_sample(
        "unicode_content",
        vec![
            ("japanese.txt", "こんにちは世界".as_bytes().to_vec()),
            ("emoji.txt", "Hello 👋 World 🌍".as_bytes().to_vec()),
            ("math.txt", "∑∫∂∇ × ∞ = πr²".as_bytes().to_vec()),
            ("mixed.txt", "Ça va? Привет! 你好!".as_bytes().to_vec()),
        ],
    )?);

    // 31. Binary patterns (tests visual appearance)
    generated.push(create_sample(
        "binary_patterns",
        vec![
            ("zeros.bin", vec![0u8; 500]),
            ("ones.bin", vec![0xFFu8; 500]),
            ("alternating.bin", (0..500).map(|i| if i % 2 == 0 { 0x55 } else { 0xAA }).collect()),
            ("gradient.bin", (0u8..=255).cycle().take(512).collect()),
        ],
    )?);

    // 32. Stress test: 100 tiny files
    let stress_files: Vec<(&str, Vec<u8>)> = (0..100)
        .map(|i| {
            let name: &'static str = Box::leak(format!("f{i:03}").into_boxed_str());
            (name, vec![i as u8])
        })
        .collect();
    generated.push(create_sample("hundred_files", stress_files)?);

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
    println!("  unzip -l target/samples/rust_core.png");

    Ok(())
}

fn create_sample(
    name: &str,
    files: Vec<(&str, Vec<u8>)>,
) -> Result<(String, usize, String), panic> {
    let file_count = files.len();
    let total_content: usize = files.iter().map(|(_, v)| v.len()).sum();
    let min_file_size = files.iter().map(|(_, v)| v.len()).min().unwrap_or(0);
    let max_file_size = files.iter().map(|(_, v)| v.len()).max().unwrap_or(0);

    let index_map: IndexMap<Vec<u8>, Vec<u8>> = files
        .into_iter()
        .map(|(k, v)| (k.as_bytes().to_vec(), v))
        .collect();
    let files_struct: Files = index_map.into();

    let polyglot = zipng::zipng(&files_struct);

    // Validate the generated polyglot with full expectations
    assert_valid_polyglot_with(&polyglot, Some(
        Expectations::new()
            .file_count(file_count)
            .total_size(total_content)
            .min_file_size(min_file_size)
            .max_file_size(max_file_size)
    ));

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
