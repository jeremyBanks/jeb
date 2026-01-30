//! Generates sample polyglot PNG+ZIP files using REAL project files and git blobs.
//!
//! Uses actual source code, fonts, configs, and binary assets from the fic.is
//! project to create diverse, realistic sample archives. Loads binary blobs
//! from git history at runtime and feeds earlier sample outputs into later
//! samples for recursive composition, targeting all font size buckets.

use indexmap::IndexMap;
use std::fs;
use std::panic::catch_unwind;
use std::process::Command;
use zipng::{panic, Files};
use zipng::polyglot::{assert_valid_polyglot_with, Expectations};
use zipng::polyglot::fonts::{self, FontSelection};

/// Load a blob from git history by its hash.
fn git_blob(hash: &str) -> Vec<u8> {
    let output = Command::new("git")
        .args(["cat-file", "-p", hash])
        .output()
        .unwrap_or_else(|e| panic!("failed to run git cat-file: {e}"));
    assert!(
        output.status.success(),
        "git cat-file failed for {hash}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}

fn main() -> Result<(), panic> {
    let _ = fs::remove_dir_all("target/samples");
    fs::create_dir_all("target/samples")?;

    let mut generated: Vec<(String, usize, String)> = Vec::new();
    let mut failures: Vec<(String, String)> = Vec::new();

    macro_rules! sample {
        ($name:expr, $files:expr) => {{
            let name = $name;
            match catch_unwind(|| create_sample(name, $files)) {
                Ok(Ok(info)) => {
                    generated.push(info);
                }
                Ok(Err(e)) => {
                    let msg = format!("{e:?}");
                    eprintln!("FAILED: {name}: {msg}");
                    failures.push((name.to_string(), msg));
                }
                Err(panic_val) => {
                    let msg = if let Some(s) = panic_val.downcast_ref::<String>() {
                        s.clone()
                    } else if let Some(s) = panic_val.downcast_ref::<&str>() {
                        s.to_string()
                    } else {
                        "unknown panic".to_string()
                    };
                    eprintln!("PANICKED: {name}: {msg}");
                    failures.push((name.to_string(), msg));
                }
            }
        }};
    }

    // =========================================================================
    // TIER 1 — Small samples (≤512K content, large fonts)
    // =========================================================================

    // Font assets - PNG bitmaps + JSON metadata
    sample!("bitmap_fonts", vec![
        ("micro.png", include_bytes!("../src/text/micro.png").to_vec()),
        ("micro.json", include_bytes!("../src/text/micro.json").to_vec()),
        ("mini.png", include_bytes!("../src/text/mini.png").to_vec()),
        ("mini.json", include_bytes!("../src/text/mini.json").to_vec()),
        ("swiss.png", include_bytes!("../src/text/swiss.png").to_vec()),
        ("swiss.json", include_bytes!("../src/text/swiss.json").to_vec()),
    ]);

    // All font files together
    sample!("all_fonts", vec![
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
    ]);

    // Core Rust source files
    sample!("rust_core", vec![
        ("lib.rs", include_bytes!("../src/lib.rs").to_vec()),
        ("checksums.rs", include_bytes!("../src/checksums.rs").to_vec()),
        ("deflate.rs", include_bytes!("../src/deflate.rs").to_vec()),
        ("zlib.rs", include_bytes!("../src/zlib.rs").to_vec()),
    ]);

    // PNG module source
    sample!("png_module", vec![
        ("png/mod.rs", include_bytes!("../src/png.rs").to_vec()),
        ("png/data.rs", include_bytes!("../src/png/data.rs").to_vec()),
        ("png/to_png.rs", include_bytes!("../src/png/to_png.rs").to_vec()),
        ("png/write_png.rs", include_bytes!("../src/png/write_png.rs").to_vec()),
        ("png/sizes.rs", include_bytes!("../src/png/sizes.rs").to_vec()),
    ]);

    // ZIP module source
    sample!("zip_module", vec![
        ("zip/mod.rs", include_bytes!("../src/zip.rs").to_vec()),
        ("zip/data.rs", include_bytes!("../src/zip/data.rs").to_vec()),
        ("zip/to_zip.rs", include_bytes!("../src/zip/to_zip.rs").to_vec()),
        ("zip/write_zip.rs", include_bytes!("../src/zip/write_zip.rs").to_vec()),
        ("zip/configuration.rs", include_bytes!("../src/zip/configuration.rs").to_vec()),
    ]);

    // Polyglot module support files
    sample!("polyglot_support", vec![
        ("polyglot/fonts.rs", include_bytes!("../src/polyglot/fonts.rs").to_vec()),
        ("polyglot/validate.rs", include_bytes!("../src/polyglot/validate.rs").to_vec()),
    ]);

    // Palette definitions
    sample!("palettes", vec![
        ("palettes/mod.rs", include_bytes!("../src/png/palettes.rs").to_vec()),
        ("palettes/viridis.rs", include_bytes!("../src/png/palettes/viridis.rs").to_vec()),
        ("palettes/singles.rs", include_bytes!("../src/png/palettes/singles.rs").to_vec()),
        ("palettes/diagnostic.rs", include_bytes!("../src/png/palettes/diagnostic.rs").to_vec()),
        ("palettes/mappings.rs", include_bytes!("../src/png/palettes/mappings.rs").to_vec()),
    ]);

    // Project configuration
    sample!("project_config", vec![
        ("Cargo.toml", include_bytes!("../Cargo.toml").to_vec()),
        ("Cargo.lock", include_bytes!("../Cargo.lock").to_vec()),
        ("rustfmt.toml", include_bytes!("../rustfmt.toml").to_vec()),
        ("README.md", include_bytes!("../README.md").to_vec()),
        ("CLAUDE.md", include_bytes!("../CLAUDE.md").to_vec()),
    ]);

    // Documentation
    sample!("documentation", vec![
        ("docs/polyglot-architecture.md", include_bytes!("../docs/polyglot-architecture.md").to_vec()),
        ("docs/polyglot-constraints.md", include_bytes!("../docs/polyglot-constraints.md").to_vec()),
    ]);

    // Test data files
    sample!("test_data", vec![
        ("test_data/zip.zip", include_bytes!("../test_data/zip.zip").to_vec()),
        ("test_data/zip.htm", include_bytes!("../test_data/zip.htm").to_vec()),
        ("test_data/poc.png.zip", include_bytes!("../test_data/poc.png.zip").to_vec()),
    ]);

    // Example Rust files
    sample!("examples", vec![
        ("examples/zipng.rs", include_bytes!("zipng.rs").to_vec()),
        ("examples/rgb.rs", include_bytes!("rgb.rs").to_vec()),
        ("examples/indexed8bit.rs", include_bytes!("indexed8bit.rs").to_vec()),
        ("examples/greyscale8bit.rs", include_bytes!("greyscale8bit.rs").to_vec()),
    ]);

    // Parent project Rust source
    sample!("fic_source", vec![
        ("src/lib.rs", include_bytes!("../../../src/lib.rs").to_vec()),
        ("src/backend.rs", include_bytes!("../../../src/backend.rs").to_vec()),
        ("src/engine.rs", include_bytes!("../../../src/engine.rs").to_vec()),
        ("src/query.rs", include_bytes!("../../../src/query.rs").to_vec()),
    ]);

    // Mixed binary + text from project
    sample!("mixed_project", vec![
        ("README.md", include_bytes!("../../../README.md").to_vec()),
        ("Cargo.toml", include_bytes!("../../../Cargo.toml").to_vec()),
        ("CLAUDE.md", include_bytes!("../../../CLAUDE.md").to_vec()),
    ]);

    // VSCode settings
    sample!("vscode_configs", vec![
        (".vscode/settings.json", include_bytes!("../.vscode/settings.json").to_vec()),
        (".vscode/launch.json", include_bytes!("../.vscode/launch.json").to_vec()),
    ]);

    // Script files
    sample!("scripts", vec![
        ("scripts/debug_polyglot.rs", include_bytes!("../scripts/debug_polyglot.rs").to_vec()),
        ("scripts/check_font.rs", include_bytes!("../scripts/check_font.rs").to_vec()),
        ("scripts/test_kerning.rs", include_bytes!("../scripts/test_kerning.rs").to_vec()),
    ]);

    // IO module
    sample!("io_module", vec![
        ("io/mod.rs", include_bytes!("../src/io.rs").to_vec()),
        ("io/alignment.rs", include_bytes!("../src/io/alignment.rs").to_vec()),
    ]);

    // Exploration docs
    sample!("exploration_docs", vec![
        ("idat-boundary-analysis.md", include_bytes!("../docs/exploration/idat-boundary-analysis.md").to_vec()),
        ("variable-width-math.md", include_bytes!("../docs/exploration/variable-width-math.md").to_vec()),
        ("variable-width-plan.md", include_bytes!("../docs/exploration/variable-width-plan.md").to_vec()),
        ("wider-images-analysis.md", include_bytes!("../docs/exploration/wider-images-analysis.md").to_vec()),
    ]);

    // Large single file
    sample!("large_single", vec![
        ("Cargo.lock", include_bytes!("../Cargo.lock").to_vec()),
    ]);

    // NEW: Binary mix — woff2 fonts + favicon + cover + zip files (all from git blobs)
    sample!("binary_mix", vec![
        ("sans701.woff2", git_blob("43f253e52c22ddc961267a7422f4f0a3fbdad3ff")),
        ("sans401.woff2", git_blob("d35d3a78dc5df918f102b566115f22845bfcce31")),
        ("sans700.woff2", git_blob("19a58eace2f37f645b2e724046288a979efafd84")),
        ("sans400.woff2", git_blob("99b3c6f5e440bc94bb70b1727928066cadfd1bbf")),
        ("favicon.ico", git_blob("b99cca8acdeaab2246318b3489fdda5c9544f425")),
        ("cover.png", git_blob("b3fc3f510f4f6c0b832bdc04c31daeab28602412")),
        ("zip.zip", git_blob("00f315606dd99dd49565bdd92ccbd72f32ddca9f")),
        ("default.zip", git_blob("434715bf08cdbbc21eda2a3c0bad1f8d1277344f")),
    ]);

    // NEW: Images — various PNG blobs from history at different sizes
    sample!("images", vec![
        ("hybrid.png", git_blob("4f77dc8bc5201e9b4ae092b4c569b1cbac0aedb8")),
        ("revolution.png", git_blob("cb4d4bb40c47238570b7fe0ce8ae5980ef7b7c6b")),
        ("icon.png", git_blob("518ab5baa722247ee4b9311c43f251abdf7f908e")),
        ("cover.png", git_blob("b3fc3f510f4f6c0b832bdc04c31daeab28602412")),
        ("poc-gray.png", git_blob("ef554fe90423bdd8212bf1c1b7fb31470c612ba9")),
        ("poc-turbo.png", git_blob("7d17ae102570957cfd7a9acfdee22088efa50219")),
        ("sizes-2048.png", git_blob("cda9599db284f7bc413a8e0990f6e896e83ef829")),
        ("fantasy14pt.png", git_blob("af47e0675796fd5e5e48f74230c83c6c91c1d316")),
        ("sans7pt.png", git_blob("a961936492531d211497582601589f13704dd3f1")),
        ("sizes-512.png", git_blob("36c049e6e23520140698f5e79888053473a179dc")),
        ("sizes-32.png", git_blob("bf483fdaa9ad9febe5e27023260e9e8370b48779")),
        ("default.png", git_blob("d076cc17619ce89823a22bf8ac2186211a89e39d")),
    ]);

    // NEW: Web app files from Deno app history
    sample!("web_app", vec![
        ("ChapterPlayer.tsx", git_blob("6775460dc8205391fcfe8e2fcba1376fb4585d71")),
        ("rss.ts", git_blob("b4bc50599277271eb20ab3ebaba00b5580078b7d")),
        ("oklab.ts", git_blob("edc33782a6fcb821b3146fb8cad50fb72282b0c6")),
        ("tts.rs", git_blob("417a417289e71d71159c54cd8a1bf3f761b2e8ff")),
    ]);

    // === EDGE CASES ===

    // Many small files (tests width optimization)
    let many_files: Vec<(&str, Vec<u8>)> = (0..50)
        .map(|i| {
            let name: &'static str = Box::leak(format!("file_{i:03}.txt").into_boxed_str());
            let content = format!("Content of file {i}").into_bytes();
            (name, content)
        })
        .collect();
    sample!("many_small_files", many_files);

    // Absurdly long filenames
    sample!("long_filenames", vec![
        ("this-is-an-extremely-long-filename-that-tests-row-width-calculation-and-label-truncation.txt", b"A".to_vec()),
        ("another/very/deeply/nested/directory/structure/with/many/path/components/file.dat", b"B".to_vec()),
        ("SCREAMING_SNAKE_CASE_FILENAME_THAT_GOES_ON_AND_ON_AND_ON.TXT", b"C".to_vec()),
        ("short.txt", b"D".to_vec()),
    ]);

    // Empty files mixed with content
    sample!("with_empty_files", vec![
        ("empty1.txt", vec![]),
        ("has_content.txt", b"This file has content".to_vec()),
        ("empty2.txt", vec![]),
        ("also_has_content.txt", b"More content here".to_vec()),
        ("empty3.txt", vec![]),
    ]);

    // Single tiny file
    sample!("tiny_single", vec![("hi.txt", b"Hi".to_vec())]);

    // Unicode and special characters in content
    sample!("unicode_content", vec![
        ("japanese.txt", "こんにちは世界".as_bytes().to_vec()),
        ("emoji.txt", "Hello 👋 World 🌍".as_bytes().to_vec()),
        ("math.txt", "∑∫∂∇ × ∞ = πr²".as_bytes().to_vec()),
        ("mixed.txt", "Ça va? Привет! 你好!".as_bytes().to_vec()),
    ]);

    // Binary patterns
    sample!("binary_patterns", vec![
        ("zeros.bin", vec![0u8; 500]),
        ("ones.bin", vec![0xFFu8; 500]),
        ("alternating.bin", (0..500).map(|i| if i % 2 == 0 { 0x55 } else { 0xAA }).collect()),
        ("gradient.bin", (0u8..=255).cycle().take(512).collect()),
    ]);

    // Stress test: 100 tiny files
    let stress_files: Vec<(&str, Vec<u8>)> = (0..100)
        .map(|i| {
            let name: &'static str = Box::leak(format!("f{i:03}").into_boxed_str());
            (name, vec![i as u8])
        })
        .collect();
    sample!("hundred_files", stress_files);

    // =========================================================================
    // TIER 2 — Medium sample (512K-1M content, Mini font)
    // Target ~700KB total content. Two snapshots of source + git history blobs.
    // =========================================================================

    {
        let files: Vec<(&str, Vec<u8>)> = vec![
            // Git history text blobs
            ("history/tts.rs", git_blob("417a417289e71d71159c54cd8a1bf3f761b2e8ff")),
            ("history/ChapterPlayer.tsx", git_blob("6775460dc8205391fcfe8e2fcba1376fb4585d71")),
            ("history/rss.ts", git_blob("b4bc50599277271eb20ab3ebaba00b5580078b7d")),
            ("history/oklab.ts", git_blob("edc33782a6fcb821b3146fb8cad50fb72282b0c6")),
            ("history/RYL0051925.json", git_blob("6429e34ed36a6cc563167626d89973d64ab538d9")),
            ("history/RYL0021220.json", git_blob("8568558f3115ccd2859741b0b7ea1822beb169b5")),
            ("history/poc.htm", git_blob("5e2c1c3f950b74db76c34df838363d6a6be2c55a")),
            ("history/zip.xml", git_blob("9be3db2625f4ebc85ed17078eb22ecacf687071f")),
            // Historic polyglot/mod.rs versions (large, under 60KB)
            ("history/polyglot_v1.rs", git_blob("105696cda2fa45fa53c98b403dc681f9d3ffd2d9")),
            ("history/polyglot_v2.rs", git_blob("77e6918a8f3b0128ea404045c323398f464914d5")),
            ("history/polyglot_v3.rs", git_blob("da359b39faf049612742a212af282893e096bfe9")),
            // Historic source versions
            ("history/write_zipng_v1.rs", git_blob("580135c539bc0138087100c1dabcb0f7b7577592")),
            ("history/write_zipng_v2.rs", git_blob("fc2ebd0b541bf25d0459c57dfc9392e26442418b")),
            ("history/png_v1.rs", git_blob("01e49c4108c13a34a3fed61037916ac4dd98b401")),
            ("history/png_v2.rs", git_blob("ec87a5b3261fd6faf07c94929ee1f8cad5d386b5")),
            ("history/zip_v1.rs", git_blob("649a65c68e685b194c03bbd97d942bae2914bd6f")),
            // Binary blobs
            ("assets/hybrid.png", git_blob("4f77dc8bc5201e9b4ae092b4c569b1cbac0aedb8")),
            ("assets/revolution.png", git_blob("cb4d4bb40c47238570b7fe0ce8ae5980ef7b7c6b")),
            ("assets/sans701.woff2", git_blob("43f253e52c22ddc961267a7422f4f0a3fbdad3ff")),
            ("assets/sans401.woff2", git_blob("d35d3a78dc5df918f102b566115f22845bfcce31")),
            ("assets/sans700.woff2", git_blob("19a58eace2f37f645b2e724046288a979efafd84")),
            ("assets/sans400.woff2", git_blob("99b3c6f5e440bc94bb70b1727928066cadfd1bbf")),
            ("assets/favicon.ico", git_blob("b99cca8acdeaab2246318b3489fdda5c9544f425")),
            ("assets/cover.png", git_blob("b3fc3f510f4f6c0b832bdc04c31daeab28602412")),
            ("assets/icon.png", git_blob("518ab5baa722247ee4b9311c43f251abdf7f908e")),
            // Current source snapshot
            ("current/lib.rs", include_bytes!("../src/lib.rs").to_vec()),
            ("current/checksums.rs", include_bytes!("../src/checksums.rs").to_vec()),
            ("current/deflate.rs", include_bytes!("../src/deflate.rs").to_vec()),
            ("current/zlib.rs", include_bytes!("../src/zlib.rs").to_vec()),
            ("current/png/mod.rs", include_bytes!("../src/png.rs").to_vec()),
            ("current/png/data.rs", include_bytes!("../src/png/data.rs").to_vec()),
            ("current/png/to_png.rs", include_bytes!("../src/png/to_png.rs").to_vec()),
            ("current/png/write_png.rs", include_bytes!("../src/png/write_png.rs").to_vec()),
            ("current/png/sizes.rs", include_bytes!("../src/png/sizes.rs").to_vec()),
            ("current/zip/mod.rs", include_bytes!("../src/zip.rs").to_vec()),
            ("current/zip/data.rs", include_bytes!("../src/zip/data.rs").to_vec()),
            ("current/zip/to_zip.rs", include_bytes!("../src/zip/to_zip.rs").to_vec()),
            ("current/zip/write_zip.rs", include_bytes!("../src/zip/write_zip.rs").to_vec()),
            ("current/zip/configuration.rs", include_bytes!("../src/zip/configuration.rs").to_vec()),
            ("current/polyglot/fonts.rs", include_bytes!("../src/polyglot/fonts.rs").to_vec()),
            ("current/polyglot/validate.rs", include_bytes!("../src/polyglot/validate.rs").to_vec()),
            ("current/io/mod.rs", include_bytes!("../src/io.rs").to_vec()),
            ("current/io/alignment.rs", include_bytes!("../src/io/alignment.rs").to_vec()),
            ("current/palettes/mod.rs", include_bytes!("../src/png/palettes.rs").to_vec()),
            ("current/palettes/viridis.rs", include_bytes!("../src/png/palettes/viridis.rs").to_vec()),
            ("current/palettes/singles.rs", include_bytes!("../src/png/palettes/singles.rs").to_vec()),
            ("current/palettes/diagnostic.rs", include_bytes!("../src/png/palettes/diagnostic.rs").to_vec()),
            ("current/palettes/mappings.rs", include_bytes!("../src/png/palettes/mappings.rs").to_vec()),
            // Config
            ("Cargo.toml", include_bytes!("../Cargo.toml").to_vec()),
            ("Cargo.lock", include_bytes!("../Cargo.lock").to_vec()),
            ("README.md", include_bytes!("../README.md").to_vec()),
            // Docs
            ("docs/polyglot-architecture.md", include_bytes!("../docs/polyglot-architecture.md").to_vec()),
            ("docs/polyglot-constraints.md", include_bytes!("../docs/polyglot-constraints.md").to_vec()),
            ("docs/idat-boundary-analysis.md", include_bytes!("../docs/exploration/idat-boundary-analysis.md").to_vec()),
            ("docs/variable-width-math.md", include_bytes!("../docs/exploration/variable-width-math.md").to_vec()),
            ("docs/variable-width-plan.md", include_bytes!("../docs/exploration/variable-width-plan.md").to_vec()),
            ("docs/wider-images-analysis.md", include_bytes!("../docs/exploration/wider-images-analysis.md").to_vec()),
        ];

        sample!("mini_font_collection", files);
    }

    // =========================================================================
    // TIER 3 — Large sample (1M-3M content, Micro font)
    // Target ~1.5MB total content. Three snapshots + all blobs.
    // =========================================================================

    {
        let files: Vec<(&str, Vec<u8>)> = vec![
            // Git history text blobs
            ("history/tts.rs", git_blob("417a417289e71d71159c54cd8a1bf3f761b2e8ff")),
            ("history/ChapterPlayer.tsx", git_blob("6775460dc8205391fcfe8e2fcba1376fb4585d71")),
            ("history/rss.ts", git_blob("b4bc50599277271eb20ab3ebaba00b5580078b7d")),
            ("history/oklab.ts", git_blob("edc33782a6fcb821b3146fb8cad50fb72282b0c6")),
            ("history/RYL0051925.json", git_blob("6429e34ed36a6cc563167626d89973d64ab538d9")),
            ("history/RYL0021220.json", git_blob("8568558f3115ccd2859741b0b7ea1822beb169b5")),
            ("history/poc.htm", git_blob("5e2c1c3f950b74db76c34df838363d6a6be2c55a")),
            ("history/zip.xml", git_blob("9be3db2625f4ebc85ed17078eb22ecacf687071f")),
            ("history/polyglot_v1.rs", git_blob("105696cda2fa45fa53c98b403dc681f9d3ffd2d9")),
            ("history/polyglot_v2.rs", git_blob("77e6918a8f3b0128ea404045c323398f464914d5")),
            ("history/polyglot_v3.rs", git_blob("da359b39faf049612742a212af282893e096bfe9")),
            ("history/write_zipng_v1.rs", git_blob("580135c539bc0138087100c1dabcb0f7b7577592")),
            ("history/write_zipng_v2.rs", git_blob("fc2ebd0b541bf25d0459c57dfc9392e26442418b")),
            ("history/png_v1.rs", git_blob("01e49c4108c13a34a3fed61037916ac4dd98b401")),
            ("history/png_v2.rs", git_blob("ec87a5b3261fd6faf07c94929ee1f8cad5d386b5")),
            ("history/zip_v1.rs", git_blob("649a65c68e685b194c03bbd97d942bae2914bd6f")),
            // Binary blobs
            ("assets/hybrid.png", git_blob("4f77dc8bc5201e9b4ae092b4c569b1cbac0aedb8")),
            ("assets/revolution.png", git_blob("cb4d4bb40c47238570b7fe0ce8ae5980ef7b7c6b")),
            ("assets/icon.png", git_blob("518ab5baa722247ee4b9311c43f251abdf7f908e")),
            ("assets/cover.png", git_blob("b3fc3f510f4f6c0b832bdc04c31daeab28602412")),
            ("assets/poc-gray.png", git_blob("ef554fe90423bdd8212bf1c1b7fb31470c612ba9")),
            ("assets/poc-turbo.png", git_blob("7d17ae102570957cfd7a9acfdee22088efa50219")),
            ("assets/sizes-2048.png", git_blob("cda9599db284f7bc413a8e0990f6e896e83ef829")),
            ("assets/sans701.woff2", git_blob("43f253e52c22ddc961267a7422f4f0a3fbdad3ff")),
            ("assets/sans401.woff2", git_blob("d35d3a78dc5df918f102b566115f22845bfcce31")),
            ("assets/sans700.woff2", git_blob("19a58eace2f37f645b2e724046288a979efafd84")),
            ("assets/sans400.woff2", git_blob("99b3c6f5e440bc94bb70b1727928066cadfd1bbf")),
            ("assets/favicon.ico", git_blob("b99cca8acdeaab2246318b3489fdda5c9544f425")),
            ("assets/poc.png.zip", git_blob("7d5e5c629b918f8dcbec25164b12484e949253dd")),
            // Snapshot A — current zipng source
            ("snapshot-a/lib.rs", include_bytes!("../src/lib.rs").to_vec()),
            ("snapshot-a/checksums.rs", include_bytes!("../src/checksums.rs").to_vec()),
            ("snapshot-a/deflate.rs", include_bytes!("../src/deflate.rs").to_vec()),
            ("snapshot-a/zlib.rs", include_bytes!("../src/zlib.rs").to_vec()),
            ("snapshot-a/png/mod.rs", include_bytes!("../src/png.rs").to_vec()),
            ("snapshot-a/png/data.rs", include_bytes!("../src/png/data.rs").to_vec()),
            ("snapshot-a/png/to_png.rs", include_bytes!("../src/png/to_png.rs").to_vec()),
            ("snapshot-a/png/write_png.rs", include_bytes!("../src/png/write_png.rs").to_vec()),
            ("snapshot-a/png/sizes.rs", include_bytes!("../src/png/sizes.rs").to_vec()),
            ("snapshot-a/zip/mod.rs", include_bytes!("../src/zip.rs").to_vec()),
            ("snapshot-a/zip/data.rs", include_bytes!("../src/zip/data.rs").to_vec()),
            ("snapshot-a/zip/to_zip.rs", include_bytes!("../src/zip/to_zip.rs").to_vec()),
            ("snapshot-a/zip/write_zip.rs", include_bytes!("../src/zip/write_zip.rs").to_vec()),
            ("snapshot-a/zip/configuration.rs", include_bytes!("../src/zip/configuration.rs").to_vec()),
            ("snapshot-a/polyglot/fonts.rs", include_bytes!("../src/polyglot/fonts.rs").to_vec()),
            ("snapshot-a/polyglot/validate.rs", include_bytes!("../src/polyglot/validate.rs").to_vec()),
            ("snapshot-a/io/mod.rs", include_bytes!("../src/io.rs").to_vec()),
            ("snapshot-a/io/alignment.rs", include_bytes!("../src/io/alignment.rs").to_vec()),
            ("snapshot-a/palettes/mod.rs", include_bytes!("../src/png/palettes.rs").to_vec()),
            ("snapshot-a/palettes/viridis.rs", include_bytes!("../src/png/palettes/viridis.rs").to_vec()),
            ("snapshot-a/palettes/singles.rs", include_bytes!("../src/png/palettes/singles.rs").to_vec()),
            ("snapshot-a/palettes/diagnostic.rs", include_bytes!("../src/png/palettes/diagnostic.rs").to_vec()),
            ("snapshot-a/palettes/mappings.rs", include_bytes!("../src/png/palettes/mappings.rs").to_vec()),
            // Snapshot A2 — duplicate source under different prefix for padding
            ("snapshot-a2/lib.rs", include_bytes!("../src/lib.rs").to_vec()),
            ("snapshot-a2/checksums.rs", include_bytes!("../src/checksums.rs").to_vec()),
            ("snapshot-a2/deflate.rs", include_bytes!("../src/deflate.rs").to_vec()),
            ("snapshot-a2/zlib.rs", include_bytes!("../src/zlib.rs").to_vec()),
            ("snapshot-a2/png/mod.rs", include_bytes!("../src/png.rs").to_vec()),
            ("snapshot-a2/png/data.rs", include_bytes!("../src/png/data.rs").to_vec()),
            ("snapshot-a2/png/to_png.rs", include_bytes!("../src/png/to_png.rs").to_vec()),
            ("snapshot-a2/png/write_png.rs", include_bytes!("../src/png/write_png.rs").to_vec()),
            ("snapshot-a2/png/sizes.rs", include_bytes!("../src/png/sizes.rs").to_vec()),
            ("snapshot-a2/zip/mod.rs", include_bytes!("../src/zip.rs").to_vec()),
            ("snapshot-a2/zip/data.rs", include_bytes!("../src/zip/data.rs").to_vec()),
            ("snapshot-a2/zip/to_zip.rs", include_bytes!("../src/zip/to_zip.rs").to_vec()),
            ("snapshot-a2/zip/write_zip.rs", include_bytes!("../src/zip/write_zip.rs").to_vec()),
            ("snapshot-a2/zip/configuration.rs", include_bytes!("../src/zip/configuration.rs").to_vec()),
            ("snapshot-a2/polyglot/fonts.rs", include_bytes!("../src/polyglot/fonts.rs").to_vec()),
            ("snapshot-a2/polyglot/validate.rs", include_bytes!("../src/polyglot/validate.rs").to_vec()),
            ("snapshot-a2/io/mod.rs", include_bytes!("../src/io.rs").to_vec()),
            ("snapshot-a2/io/alignment.rs", include_bytes!("../src/io/alignment.rs").to_vec()),
            ("snapshot-a2/palettes/mod.rs", include_bytes!("../src/png/palettes.rs").to_vec()),
            ("snapshot-a2/palettes/viridis.rs", include_bytes!("../src/png/palettes/viridis.rs").to_vec()),
            ("snapshot-a2/palettes/singles.rs", include_bytes!("../src/png/palettes/singles.rs").to_vec()),
            ("snapshot-a2/palettes/diagnostic.rs", include_bytes!("../src/png/palettes/diagnostic.rs").to_vec()),
            ("snapshot-a2/palettes/mappings.rs", include_bytes!("../src/png/palettes/mappings.rs").to_vec()),
            // Snapshot B — parent project + config + docs + examples
            ("snapshot-b/fic/lib.rs", include_bytes!("../../../src/lib.rs").to_vec()),
            ("snapshot-b/fic/backend.rs", include_bytes!("../../../src/backend.rs").to_vec()),
            ("snapshot-b/fic/engine.rs", include_bytes!("../../../src/engine.rs").to_vec()),
            ("snapshot-b/fic/query.rs", include_bytes!("../../../src/query.rs").to_vec()),
            ("snapshot-b/Cargo.toml", include_bytes!("../Cargo.toml").to_vec()),
            ("snapshot-b/Cargo.lock", include_bytes!("../Cargo.lock").to_vec()),
            ("snapshot-b/README.md", include_bytes!("../README.md").to_vec()),
            ("snapshot-b/CLAUDE.md", include_bytes!("../CLAUDE.md").to_vec()),
            ("snapshot-b/examples/zipng.rs", include_bytes!("zipng.rs").to_vec()),
            ("snapshot-b/examples/rgb.rs", include_bytes!("rgb.rs").to_vec()),
            ("snapshot-b/examples/indexed8bit.rs", include_bytes!("indexed8bit.rs").to_vec()),
            ("snapshot-b/examples/greyscale8bit.rs", include_bytes!("greyscale8bit.rs").to_vec()),
            ("snapshot-b/scripts/debug_polyglot.rs", include_bytes!("../scripts/debug_polyglot.rs").to_vec()),
            ("snapshot-b/scripts/check_font.rs", include_bytes!("../scripts/check_font.rs").to_vec()),
            ("snapshot-b/scripts/test_kerning.rs", include_bytes!("../scripts/test_kerning.rs").to_vec()),
            // Snapshot C — docs
            ("snapshot-c/polyglot-architecture.md", include_bytes!("../docs/polyglot-architecture.md").to_vec()),
            ("snapshot-c/polyglot-constraints.md", include_bytes!("../docs/polyglot-constraints.md").to_vec()),
            ("snapshot-c/idat-boundary-analysis.md", include_bytes!("../docs/exploration/idat-boundary-analysis.md").to_vec()),
            ("snapshot-c/variable-width-math.md", include_bytes!("../docs/exploration/variable-width-math.md").to_vec()),
            ("snapshot-c/variable-width-plan.md", include_bytes!("../docs/exploration/variable-width-plan.md").to_vec()),
            ("snapshot-c/wider-images-analysis.md", include_bytes!("../docs/exploration/wider-images-analysis.md").to_vec()),
            // Duplicate large history blobs under different paths for padding
            ("archive/poc.htm", git_blob("5e2c1c3f950b74db76c34df838363d6a6be2c55a")),
            ("archive/zip.xml", git_blob("9be3db2625f4ebc85ed17078eb22ecacf687071f")),
            ("archive/RYL0051925.json", git_blob("6429e34ed36a6cc563167626d89973d64ab538d9")),
            ("archive/RYL0021220.json", git_blob("8568558f3115ccd2859741b0b7ea1822beb169b5")),
            ("archive/polyglot_v2.rs", git_blob("77e6918a8f3b0128ea404045c323398f464914d5")),
            ("archive/polyglot_v3.rs", git_blob("da359b39faf049612742a212af282893e096bfe9")),
        ];

        sample!("micro_font_collection", files);
    }

    // =========================================================================
    // RGBA with labels (2-3M content: triggers RGBA mode but stays under label threshold)
    // =========================================================================

    {
        let mut files: Vec<(&str, Vec<u8>)> = vec![
            ("src/lib.rs", include_bytes!("../src/lib.rs").to_vec()),
            ("src/checksums.rs", include_bytes!("../src/checksums.rs").to_vec()),
            ("src/deflate.rs", include_bytes!("../src/deflate.rs").to_vec()),
            ("src/zlib.rs", include_bytes!("../src/zlib.rs").to_vec()),
            ("src/png.rs", include_bytes!("../src/png.rs").to_vec()),
            ("src/zip.rs", include_bytes!("../src/zip.rs").to_vec()),
            ("src/io.rs", include_bytes!("../src/io.rs").to_vec()),
            ("docs/README.md", include_bytes!("../README.md").to_vec()),
            ("docs/CLAUDE.md", include_bytes!("../CLAUDE.md").to_vec()),
            ("config/Cargo.toml", include_bytes!("../Cargo.toml").to_vec()),
        ];

        // Add padding files to reach ~2.5 MiB total content
        let content_so_far: usize = files.iter().map(|(_, v)| v.len()).sum();
        let target = 2 * 1024 * 1024 + 512 * 1024; // 2.5 MiB
        if content_so_far < target {
            let remaining = target - content_so_far;
            let chunk_size = 50_000;
            let chunks = (remaining + chunk_size - 1) / chunk_size;
            for i in 0..chunks {
                let size = if i < chunks - 1 { chunk_size } else { remaining - i * chunk_size };
                let name: &'static str = Box::leak(format!("padding/pad_{i:03}.bin").into_boxed_str());
                files.push((name, vec![0x42; size]));
            }
        }

        sample!("rgba_with_labels", files);
    }

    // =========================================================================
    // TIER 4 — Huge sample (>3M content, no labels)
    // Target ~3.5MB total content. Everything multiplied across snapshots.
    // =========================================================================

    {
        // Build a base set of source files we'll replicate under multiple prefixes
        let source_snapshot: Vec<(&str, &[u8])> = vec![
            ("lib.rs", include_bytes!("../src/lib.rs")),
            ("checksums.rs", include_bytes!("../src/checksums.rs")),
            ("deflate.rs", include_bytes!("../src/deflate.rs")),
            ("zlib.rs", include_bytes!("../src/zlib.rs")),
            ("png/mod.rs", include_bytes!("../src/png.rs")),
            ("png/data.rs", include_bytes!("../src/png/data.rs")),
            ("png/to_png.rs", include_bytes!("../src/png/to_png.rs")),
            ("png/write_png.rs", include_bytes!("../src/png/write_png.rs")),
            ("png/sizes.rs", include_bytes!("../src/png/sizes.rs")),
            ("zip/mod.rs", include_bytes!("../src/zip.rs")),
            ("zip/data.rs", include_bytes!("../src/zip/data.rs")),
            ("zip/to_zip.rs", include_bytes!("../src/zip/to_zip.rs")),
            ("zip/write_zip.rs", include_bytes!("../src/zip/write_zip.rs")),
            ("zip/configuration.rs", include_bytes!("../src/zip/configuration.rs")),
            ("polyglot/fonts.rs", include_bytes!("../src/polyglot/fonts.rs")),
            ("polyglot/validate.rs", include_bytes!("../src/polyglot/validate.rs")),
            ("io/mod.rs", include_bytes!("../src/io.rs")),
            ("io/alignment.rs", include_bytes!("../src/io/alignment.rs")),
            ("palettes/mod.rs", include_bytes!("../src/png/palettes.rs")),
            ("palettes/viridis.rs", include_bytes!("../src/png/palettes/viridis.rs")),
            ("palettes/singles.rs", include_bytes!("../src/png/palettes/singles.rs")),
            ("palettes/diagnostic.rs", include_bytes!("../src/png/palettes/diagnostic.rs")),
            ("palettes/mappings.rs", include_bytes!("../src/png/palettes/mappings.rs")),
        ];

        let mut files: Vec<(String, Vec<u8>)> = Vec::new();

        // Add source snapshots under multiple prefixes (~250KB each × 7 = ~1.75MB)
        for prefix in ["v1", "v2", "v3", "v4", "v5", "v6", "v7"] {
            for (name, data) in &source_snapshot {
                files.push((format!("{prefix}/{name}"), data.to_vec()));
            }
        }

        // Git history text blobs (~150KB)
        let history_blobs: Vec<(&str, &str)> = vec![
            ("tts.rs", "417a417289e71d71159c54cd8a1bf3f761b2e8ff"),
            ("ChapterPlayer.tsx", "6775460dc8205391fcfe8e2fcba1376fb4585d71"),
            ("rss.ts", "b4bc50599277271eb20ab3ebaba00b5580078b7d"),
            ("oklab.ts", "edc33782a6fcb821b3146fb8cad50fb72282b0c6"),
            ("RYL0051925.json", "6429e34ed36a6cc563167626d89973d64ab538d9"),
            ("RYL0021220.json", "8568558f3115ccd2859741b0b7ea1822beb169b5"),
            ("poc.htm", "5e2c1c3f950b74db76c34df838363d6a6be2c55a"),
            ("zip.xml", "9be3db2625f4ebc85ed17078eb22ecacf687071f"),
            ("polyglot_v1.rs", "105696cda2fa45fa53c98b403dc681f9d3ffd2d9"),
            ("polyglot_v2.rs", "77e6918a8f3b0128ea404045c323398f464914d5"),
            ("polyglot_v3.rs", "da359b39faf049612742a212af282893e096bfe9"),
        ];

        // Replicate history blobs under multiple prefixes (~150KB × 4 = ~600KB)
        for prefix in ["era-a", "era-b", "era-c", "era-d"] {
            for (name, hash) in &history_blobs {
                files.push((format!("{prefix}/{name}"), git_blob(hash)));
            }
        }

        // Binary assets (~200KB)
        for (name, hash) in [
            ("assets/hybrid.png", "4f77dc8bc5201e9b4ae092b4c569b1cbac0aedb8"),
            ("assets/revolution.png", "cb4d4bb40c47238570b7fe0ce8ae5980ef7b7c6b"),
            ("assets/icon.png", "518ab5baa722247ee4b9311c43f251abdf7f908e"),
            ("assets/cover.png", "b3fc3f510f4f6c0b832bdc04c31daeab28602412"),
            ("assets/poc-gray.png", "ef554fe90423bdd8212bf1c1b7fb31470c612ba9"),
            ("assets/poc-turbo.png", "7d17ae102570957cfd7a9acfdee22088efa50219"),
            ("assets/sizes-2048.png", "cda9599db284f7bc413a8e0990f6e896e83ef829"),
            ("assets/fantasy14pt.png", "af47e0675796fd5e5e48f74230c83c6c91c1d316"),
            ("assets/sans7pt.png", "a961936492531d211497582601589f13704dd3f1"),
            ("assets/sans701.woff2", "43f253e52c22ddc961267a7422f4f0a3fbdad3ff"),
            ("assets/sans401.woff2", "d35d3a78dc5df918f102b566115f22845bfcce31"),
            ("assets/sans700.woff2", "19a58eace2f37f645b2e724046288a979efafd84"),
            ("assets/sans400.woff2", "99b3c6f5e440bc94bb70b1727928066cadfd1bbf"),
            ("assets/favicon.ico", "b99cca8acdeaab2246318b3489fdda5c9544f425"),
        ] {
            files.push((name.to_string(), git_blob(hash)));
        }

        // Config + docs + examples (~200KB)
        for (name, data) in [
            ("config/Cargo.toml", include_bytes!("../Cargo.toml").as_slice()),
            ("config/Cargo.lock", include_bytes!("../Cargo.lock").as_slice()),
            ("config/README.md", include_bytes!("../README.md").as_slice()),
            ("config/CLAUDE.md", include_bytes!("../CLAUDE.md").as_slice()),
            ("config/rustfmt.toml", include_bytes!("../rustfmt.toml").as_slice()),
            ("docs/polyglot-architecture.md", include_bytes!("../docs/polyglot-architecture.md").as_slice()),
            ("docs/polyglot-constraints.md", include_bytes!("../docs/polyglot-constraints.md").as_slice()),
            ("docs/idat-boundary-analysis.md", include_bytes!("../docs/exploration/idat-boundary-analysis.md").as_slice()),
            ("docs/variable-width-math.md", include_bytes!("../docs/exploration/variable-width-math.md").as_slice()),
            ("docs/variable-width-plan.md", include_bytes!("../docs/exploration/variable-width-plan.md").as_slice()),
            ("docs/wider-images-analysis.md", include_bytes!("../docs/exploration/wider-images-analysis.md").as_slice()),
            ("examples/zipng.rs", include_bytes!("zipng.rs").as_slice()),
            ("examples/rgb.rs", include_bytes!("rgb.rs").as_slice()),
            ("examples/indexed8bit.rs", include_bytes!("indexed8bit.rs").as_slice()),
            ("examples/greyscale8bit.rs", include_bytes!("greyscale8bit.rs").as_slice()),
            ("scripts/debug_polyglot.rs", include_bytes!("../scripts/debug_polyglot.rs").as_slice()),
            ("scripts/check_font.rs", include_bytes!("../scripts/check_font.rs").as_slice()),
            ("scripts/test_kerning.rs", include_bytes!("../scripts/test_kerning.rs").as_slice()),
            ("parent/lib.rs", include_bytes!("../../../src/lib.rs").as_slice()),
            ("parent/backend.rs", include_bytes!("../../../src/backend.rs").as_slice()),
            ("parent/engine.rs", include_bytes!("../../../src/engine.rs").as_slice()),
            ("parent/query.rs", include_bytes!("../../../src/query.rs").as_slice()),
        ] {
            files.push((name.to_string(), data.to_vec()));
        }

        // Add padding files to push total content over 3 MiB (label threshold)
        let content_so_far: usize = files.iter().map(|(_, v)| v.len()).sum();
        let target = 3 * 1024 * 1024 + 1024; // just over 3 MiB
        if content_so_far < target {
            let remaining = target - content_so_far;
            // Split into ~50KB chunks (under MAX_FILE_CONTENT_SIZE)
            let chunk_size = 50_000;
            let chunks = (remaining + chunk_size - 1) / chunk_size;
            for i in 0..chunks {
                let size = if i < chunks - 1 { chunk_size } else { remaining - i * chunk_size };
                files.push((format!("padding/pad_{i:03}.bin"), vec![0x42; size]));
            }
        }

        // Convert String keys to &str via leaked strings (needed for create_sample)
        let files: Vec<(&str, Vec<u8>)> = files
            .into_iter()
            .map(|(k, v)| {
                let k: &'static str = Box::leak(k.into_boxed_str());
                (k, v)
            })
            .collect();

        sample!("no_labels_collection", files);
    }

    // =========================================================================
    // CHUNKING STRESS TESTS — Files sized to provoke significant gaps
    // =========================================================================
    // IDAT bucket capacity is ~60-64KB of data (depends on row width).
    // Files just over half that size (~33KB content + overhead) can't share a
    // bucket, forcing each into its own bucket with ~half the space wasted as
    // gaps. These samples exercise the bin packing and spacing logic heavily.

    // 1. All files just over half bucket size — every file gets its own bucket
    {
        let files: Vec<(&str, Vec<u8>)> = (0..8)
            .map(|i| {
                let name: &'static str = Box::leak(format!("chunk_{i:02}.bin").into_boxed_str());
                // ~33KB each: just over half of ~64KB bucket capacity
                let content: Vec<u8> = (0..33_000u32).map(|j| ((i * 37 + j * 13) % 256) as u8).collect();
                (name, content)
            })
            .collect();
        sample!("chunking_half_plus", files);
    }

    // 2. Half-plus files mixed with small files that fill the gaps
    {
        let mut files: Vec<(&str, Vec<u8>)> = Vec::new();
        for i in 0..6 {
            let name: &'static str = Box::leak(format!("big_{i:02}.bin").into_boxed_str());
            let content: Vec<u8> = (0..34_000u32).map(|j| ((i * 41 + j * 11) % 256) as u8).collect();
            files.push((name, content));
        }
        for i in 0..12 {
            let name: &'static str = Box::leak(format!("small_{i:02}.txt").into_boxed_str());
            // ~2-5KB each, random-ish sizes
            let size = 2000 + (i * 271) % 3000;
            let content: Vec<u8> = (0..size).map(|j| ((i * 19 + j * 7) % 256) as u8).collect();
            files.push((name, content));
        }
        sample!("chunking_mixed_half", files);
    }

    // 3. Files at exactly varying fractions of bucket size (1/3, 1/2+, 2/3)
    //    to create irregular gap patterns
    {
        let files: Vec<(&str, Vec<u8>)> = vec![
            ("third_a.bin", vec![0xAA; 20_000]),  // ~1/3 bucket
            ("third_b.bin", vec![0xBB; 20_000]),  // ~1/3 bucket (fits with third_a)
            ("half_plus_a.bin", vec![0xCC; 35_000]),  // just over 1/2
            ("third_c.bin", vec![0xDD; 20_000]),  // ~1/3 (can it fit with half_plus?)
            ("half_plus_b.bin", vec![0xEE; 35_000]),  // just over 1/2
            ("half_plus_c.bin", vec![0xFF; 35_000]),  // just over 1/2
            ("quarter_a.bin", vec![0x11; 15_000]),  // ~1/4
            ("quarter_b.bin", vec![0x22; 15_000]),  // ~1/4
            ("quarter_c.bin", vec![0x33; 15_000]),  // ~1/4
            ("tiny.bin", vec![0x44; 500]),          // tiny filler
        ];
        sample!("chunking_mixed_fractions", files);
    }

    // 4. Near-max files (~58KB) that barely fit one per bucket, with small
    //    files that can only squeeze into leftover scraps
    {
        let mut files: Vec<(&str, Vec<u8>)> = Vec::new();
        for i in 0..4 {
            let name: &'static str = Box::leak(format!("huge_{i}.bin").into_boxed_str());
            let content: Vec<u8> = (0..58_000u32).map(|j| ((i * 53 + j * 3) % 256) as u8).collect();
            files.push((name, content));
        }
        for i in 0..20 {
            let name: &'static str = Box::leak(format!("crumb_{i:02}.txt").into_boxed_str());
            let content: Vec<u8> = (0..200u32).map(|j| ((i * 23 + j) % 256) as u8).collect();
            files.push((name, content));
        }
        sample!("chunking_near_max", files);
    }

    // 5. Many half-plus files with random sub-half files — larger scale stress
    {
        let mut files: Vec<(&str, Vec<u8>)> = Vec::new();
        for i in 0..10 {
            let name: &'static str = Box::leak(format!("block_{i:02}.bin").into_boxed_str());
            // Vary slightly around 33KB to create different gap sizes
            let size = 31_000 + (i * 997) % 5000;
            let content: Vec<u8> = (0..size as u32).map(|j| ((i as u32 * 43 + j * 17) % 256) as u8).collect();
            files.push((name, content));
        }
        for i in 0..15 {
            let name: &'static str = Box::leak(format!("fill_{i:02}.dat").into_boxed_str());
            // Random sizes from 500B to 25KB
            let size = 500 + (i * 1733) % 25_000;
            let content: Vec<u8> = (0..size as u32).map(|j| ((i as u32 * 31 + j * 9) % 256) as u8).collect();
            files.push((name, content));
        }
        sample!("chunking_stress", files);
    }

    // =========================================================================
    // VARIANT GALLERY — png_module with each font and color scheme
    // =========================================================================

    {
        fs::create_dir_all("target/samples/png_module")?;

        let png_module_files: Vec<(&str, Vec<u8>)> = vec![
            ("png/mod.rs", include_bytes!("../src/png.rs").to_vec()),
            ("png/data.rs", include_bytes!("../src/png/data.rs").to_vec()),
            ("png/to_png.rs", include_bytes!("../src/png/to_png.rs").to_vec()),
            ("png/write_png.rs", include_bytes!("../src/png/write_png.rs").to_vec()),
            ("png/sizes.rs", include_bytes!("../src/png/sizes.rs").to_vec()),
        ];

        let sorted_files: Vec<(&[u8], &[u8])> = {
            let mut v: Vec<(&[u8], &[u8])> = png_module_files.iter()
                .map(|(k, v)| (k.as_bytes() as &[u8], v.as_slice() as &[u8]))
                .collect();
            v.sort_by_key(|(a, _)| *a);
            v
        };

        // Default size font for all font variants
        let size_font: &'static fonts::BitmapFont = &fonts::SUGIMORI;

        // Font variants: each of the 7 fonts as the name font
        let font_variants: &[(&str, &once_cell::sync::Lazy<fonts::BitmapFont>)] = &[
            ("micro", &fonts::MICRO),
            ("mini", &fonts::MINI),
            ("monte", &fonts::MONTE),
            ("sixth", &fonts::SIXTH),
            ("sky", &fonts::SKY),
            ("sugimori", &fonts::SUGIMORI),
            ("swiss", &fonts::SWISS),
        ];

        for (font_name, font_ref) in font_variants {
            let font_sel = FontSelection {
                name_font: font_ref,
                size_font,
            };

            // Use default indexed color with a fixed palette
            let palette = zipng::palettes::viridis::VIRIDIS;
            let deduped = zipng::palettes::perceptual::deduplicate_rgb_palette(palette);
            let polyglot = zipng::polyglot::build_polyglot_with_font(
                &sorted_files,
                0,
                zipng::BitDepth::EightBit,
                zipng::ColorType::Indexed,
                Some(&deduped),
                Some(font_sel),
            );
            let path = format!("target/samples/png_module/font_{font_name}.png");
            fs::write(&path, &polyglot)?;
            println!("  variant: font_{font_name}.png ({} bytes)", polyglot.len());
        }

        // Color scheme variants: each palette + RGBA
        let palette_variants: &[(&str, &[u8])] = &[
            ("amp", zipng::palettes::oceanic::AMP),
            ("ice", zipng::palettes::oceanic::ICE),
            ("oxy", zipng::palettes::oceanic::OXY),
            ("buda", zipng::palettes::crameri::BUDA),
            ("nuuk", zipng::palettes::crameri::NUUK),
            ("oslo", zipng::palettes::crameri::OSLO),
            ("deep", zipng::palettes::oceanic::DEEP),
            ("rain", zipng::palettes::oceanic::RAIN),
            ("acton", zipng::palettes::crameri::ACTON),
            ("davos", zipng::palettes::crameri::DAVOS),
            ("devon", zipng::palettes::crameri::DEVON),
            ("imola", zipng::palettes::crameri::IMOLA),
            ("lapaz", zipng::palettes::crameri::LAPAZ),
            ("tokyo", zipng::palettes::crameri::TOKYO),
            ("turku", zipng::palettes::crameri::TURKU),
            ("algae", zipng::palettes::oceanic::ALGAE),
            ("dense", zipng::palettes::oceanic::DENSE),
            ("solar", zipng::palettes::oceanic::SOLAR),
            ("speed", zipng::palettes::oceanic::SPEED),
            ("tempo", zipng::palettes::oceanic::TEMPO),
            ("magma", zipng::palettes::viridis::MAGMA),
            ("bamako", zipng::palettes::crameri::BAMAKO),
            ("batlow", zipng::palettes::crameri::BATLOW),
            ("bilbao", zipng::palettes::crameri::BILBAO),
            ("hawaii", zipng::palettes::crameri::HAWAII),
            ("haline", zipng::palettes::oceanic::HALINE),
            ("matter", zipng::palettes::oceanic::MATTER),
            ("turbid", zipng::palettes::oceanic::TURBID),
            ("plasma", zipng::palettes::viridis::PLASMA),
            ("lajolla", zipng::palettes::crameri::LAJOLLA),
            ("thermal", zipng::palettes::oceanic::THERMAL),
            ("cividis", zipng::palettes::singles::CIVIDIS),
            ("inferno", zipng::palettes::viridis::INFERNO),
            ("viridis", zipng::palettes::viridis::VIRIDIS),
            ("batlow_k", zipng::palettes::crameri::BATLOW_K),
            ("batlow_w", zipng::palettes::crameri::BATLOW_W),
        ];

        for (palette_name, palette_data) in palette_variants {
            let deduped = zipng::palettes::perceptual::deduplicate_rgb_palette(palette_data);
            let polyglot = zipng::polyglot::build_polyglot(
                &sorted_files,
                0,
                zipng::BitDepth::EightBit,
                zipng::ColorType::Indexed,
                Some(&deduped),
            );
            let path = format!("target/samples/png_module/color_{palette_name}.png");
            fs::write(&path, &polyglot)?;
            println!("  variant: color_{palette_name}.png ({} bytes)", polyglot.len());
        }

        // RGBA variant
        {
            let polyglot = zipng::polyglot::build_polyglot(
                &sorted_files,
                0,
                zipng::BitDepth::EightBit,
                zipng::ColorType::RedGreenBlueAlpha,
                None,
            );
            let path = "target/samples/png_module/color_rgba.png";
            fs::write(path, &polyglot)?;
            println!("  variant: color_rgba.png ({} bytes)", polyglot.len());
        }

        println!("Generated {} font + {} color variants in target/samples/png_module/",
            font_variants.len(), palette_variants.len() + 1);
    }

    // Print summary
    println!("\nGenerated {} sample files:\n", generated.len());
    println!("{:<30} {:>10} {}", "Name", "Size", "Description");
    println!("{}", "-".repeat(80));
    for (name, size, desc) in &generated {
        println!("{:<30} {:>10} {}", name, format_size(*size), desc);
    }

    let total_size: usize = generated.iter().map(|(_, s, _)| s).sum();
    println!("{}", "-".repeat(80));
    println!("{:<30} {:>10}", "TOTAL", format_size(total_size));

    println!("\nAll files saved to target/samples/");
    println!("\nVerify with:");
    println!("  file target/samples/*.png");
    println!("  unzip -l target/samples/rust_core.png");

    if !failures.is_empty() {
        eprintln!("\n{} sample(s) FAILED:\n", failures.len());
        for (name, msg) in &failures {
            eprintln!("  {name}: {msg}");
        }
        std::process::exit(1);
    }

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
