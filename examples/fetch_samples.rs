//! Creates polyglot samples from real-world data.
//!
//! Generates a small number of larger, diverse samples by combining
//! multiple data sources into each archive.
//!
//! Usage:
//!   cargo run --example fetch_samples
//!   cargo run --example fetch_samples -- --include-network

use indexmap::IndexMap;
use std::fs;
use std::io::Read;
use std::path::Path;
use zipng::{panic, Files};

fn main() -> Result<(), panic> {
    let include_network = std::env::args().any(|a| a == "--include-network");
    let output_dir = "target/samples";

    fs::create_dir_all(output_dir)?;

    // Clean existing samples
    for entry in fs::read_dir(output_dir)? {
        if let Ok(entry) = entry {
            if entry.path().extension().map(|e| e == "png").unwrap_or(false) {
                let _ = fs::remove_file(entry.path());
            }
        }
    }

    println!("Creating polyglot samples from real-world data...\n");

    let mut generated: Vec<(String, usize, String)> = Vec::new();
    let home = std::env::var("HOME").unwrap_or_default();

    // === SAMPLE 1: This entire project (source + config + git) ===
    {
        let mut files = IndexMap::new();

        // All Rust source files (max 60KB each for polyglot compatibility)
        collect_glob(&mut files, "src", &["*.rs"], Some(60_000), Some("src/"));

        // Project config files
        collect_files(&mut files, &[
            ("Cargo.toml", "Cargo.toml"),
            ("Cargo.lock", "Cargo.lock"),
            ("README.md", "README.md"),
        ], None);

        // Git metadata
        collect_files(&mut files, &[
            ("git/HEAD", ".git/HEAD"),
            ("git/config", ".git/config"),
            ("git/index", ".git/index"),
            ("git/COMMIT_EDITMSG", ".git/COMMIT_EDITMSG"),
        ], None);

        // Git objects (binary)
        collect_git_objects(&mut files, ".git/objects", 20, 50000);

        // Examples (max 60KB each)
        collect_glob(&mut files, "examples", &["*.rs"], Some(60_000), Some("examples/"));

        generated.push(save_polyglot(output_dir, "zipng_project", files)?);
    }

    // === SAMPLE 2: System and user config files ===
    {
        let mut files = IndexMap::new();

        // System files
        collect_files(&mut files, &[
            ("etc/hosts", "/etc/hosts"),
            ("etc/passwd", "/etc/passwd"),
            ("etc/shells", "/etc/shells"),
            ("etc/resolv.conf", "/etc/resolv.conf"),
            ("etc/paths", "/etc/paths"),
        ], None);

        // User configs
        collect_files(&mut files, &[
            ("home/.bashrc", &format!("{}/.bashrc", home)),
            ("home/.zshrc", &format!("{}/.zshrc", home)),
            ("home/.profile", &format!("{}/.profile", home)),
            ("home/.gitconfig", &format!("{}/.gitconfig", home)),
            ("home/.vimrc", &format!("{}/.vimrc", home)),
            ("home/.tmux.conf", &format!("{}/.tmux.conf", home)),
        ], None);

        // SSH configs (public only)
        collect_files(&mut files, &[
            ("home/.ssh/config", &format!("{}/.ssh/config", home)),
            ("home/.ssh/known_hosts", &format!("{}/.ssh/known_hosts", home)),
        ], Some(50000));

        #[cfg(target_os = "macos")]
        {
            collect_files(&mut files, &[
                ("macos/SystemVersion.plist", "/System/Library/CoreServices/SystemVersion.plist"),
                ("macos/finder.plist", &format!("{}/Library/Preferences/com.apple.finder.plist", home)),
                ("macos/dock.plist", &format!("{}/Library/Preferences/com.apple.dock.plist", home)),
            ], Some(50000));
        }

        generated.push(save_polyglot(output_dir, "system_configs", files)?);
    }

    // === SAMPLE 3: Nearby Rust projects ===
    {
        let mut files = IndexMap::new();

        // Find Cargo.toml files in parent directories
        for entry in walkdir::WalkDir::new("..")
            .max_depth(3)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name() == "Cargo.toml")
            .take(10)
        {
            if let Ok(data) = fs::read(entry.path()) {
                let project_name = entry.path()
                    .parent()
                    .and_then(|p| p.file_name())
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "unknown".to_string());

                files.insert(format!("{}/Cargo.toml", project_name).into_bytes(), data);

                // Also grab README if exists
                let readme_path = entry.path().parent().unwrap().join("README.md");
                if let Ok(readme) = fs::read(&readme_path) {
                    let mut truncated = readme;
                    truncated.truncate(30000);
                    files.insert(format!("{}/README.md", project_name).into_bytes(), truncated);
                }

                // And src/lib.rs or src/main.rs
                for src in &["src/lib.rs", "src/main.rs"] {
                    let src_path = entry.path().parent().unwrap().join(src);
                    if let Ok(mut src_data) = fs::read(&src_path) {
                        src_data.truncate(50000);
                        files.insert(format!("{}/{}", project_name, src).into_bytes(), src_data);
                    }
                }
            }
        }

        generated.push(save_polyglot(output_dir, "rust_projects", files)?);
    }

    // === SAMPLE 4: Binary data (git pack files, compiled artifacts) ===
    {
        let mut files = IndexMap::new();

        // Git pack files (highly compressed binary, max 60KB each)
        if let Ok(entries) = fs::read_dir(".git/objects/pack") {
            for entry in entries.filter_map(|e| e.ok()).take(4) {
                if let Ok(data) = fs::read(entry.path()) {
                    if data.len() <= 60_000 {
                        let name = entry.file_name().to_string_lossy().to_string();
                        files.insert(format!("pack/{}", name).into_bytes(), data);
                    }
                }
            }
        }

        // Cargo build artifacts metadata
        collect_files(&mut files, &[
            ("target/CACHEDIR.TAG", "target/CACHEDIR.TAG"),
            ("target/.rustc_info.json", "target/.rustc_info.json"),
        ], None);

        // .rlib or .rmeta files if any exist (truncated)
        collect_glob(&mut files, "target/debug/deps", &["*.rlib", "*.rmeta"], Some(50000), Some("deps/"));

        generated.push(save_polyglot(output_dir, "binary_data", files)?);
    }

    // === SAMPLE 5: Log files ===
    {
        let mut files = IndexMap::new();

        // System logs
        collect_glob(&mut files, "/var/log", &["*.log"], Some(50000), Some("var/log/"));

        // Also try common log locations
        collect_files(&mut files, &[
            ("var/log/system.log", "/var/log/system.log"),
            ("var/log/wifi.log", "/var/log/wifi.log"),
        ], Some(50000));

        if !files.is_empty() {
            generated.push(save_polyglot(output_dir, "log_files", files)?);
        }
    }

    // === SIZE-TARGETED SAMPLES ===
    // Font thresholds: ≤128K (Sky), ≤512K (Sugimori), ≤1M (Mini), ≤3M (Micro), >3M (no labels)
    // Color thresholds: ≤1M (Indexed), ≤3M (RGB), >3M (RGBA)
    // All samples use real-world data, replicated as needed to reach target sizes.

    // Helper: collect real source files up to a target total size
    let collect_source_chunks = |target_bytes: usize| -> IndexMap<Vec<u8>, Vec<u8>> {
        let mut files = IndexMap::new();
        let mut total = 0;
        let mut copy = 0;

        // Cycle through source files until we reach target
        'outer: loop {
            for entry in walkdir::WalkDir::new("src")
                .max_depth(5)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
                .filter(|e| e.path().extension().map(|x| x == "rs").unwrap_or(false))
            {
                if let Ok(data) = fs::read(entry.path()) {
                    if data.len() > 60_000 { continue; }
                    let rel = entry.path().strip_prefix("src").unwrap_or(entry.path());
                    let name = if copy == 0 {
                        format!("src/{}", rel.to_string_lossy())
                    } else {
                        format!("copy{:02}/src/{}", copy, rel.to_string_lossy())
                    };
                    total += data.len();
                    files.insert(name.into_bytes(), data);
                    if total >= target_bytes { break 'outer; }
                }
            }
            copy += 1;
            if copy > 100 { break; } // Safety limit
        }
        files
    };

    // === SAMPLE 6: Tiny - just README.md ===
    {
        let mut files = IndexMap::new();
        collect_files(&mut files, &[("README.md", "README.md")], None);
        generated.push(save_polyglot(output_dir, "readme_only", files)?);
    }

    // === SAMPLE 7: Small (~1-5 KiB) - a few config files ===
    {
        let mut files = IndexMap::new();
        collect_files(&mut files, &[
            (".gitignore", ".gitignore"),
            ("rustfmt.toml", "rustfmt.toml"),
            ("CLAUDE.md", "CLAUDE.md"),
        ], None);
        generated.push(save_polyglot(output_dir, "small_configs", files)?);
    }

    // === SAMPLE 8: ~50 KiB - mid Sky range (source subset) ===
    {
        let files = collect_source_chunks(50_000);
        generated.push(save_polyglot(output_dir, "source_50k", files)?);
    }

    // === SAMPLE 9: ~120 KiB - near Sky/Sugimori boundary (128K) ===
    {
        let files = collect_source_chunks(120_000);
        generated.push(save_polyglot(output_dir, "source_120k", files)?);
    }

    // === SAMPLE 10: ~200 KiB - mid Sugimori range ===
    {
        let files = collect_source_chunks(200_000);
        generated.push(save_polyglot(output_dir, "source_200k", files)?);
    }

    // === SAMPLE 11: ~500 KiB - near Sugimori/Mini boundary (512K) ===
    {
        let files = collect_source_chunks(500_000);
        generated.push(save_polyglot(output_dir, "source_500k", files)?);
    }

    // === SAMPLE 12: ~800 KiB - mid Mini range ===
    {
        let files = collect_source_chunks(800_000);
        generated.push(save_polyglot(output_dir, "source_800k", files)?);
    }

    // === SAMPLE 13: ~1.5 MiB - mid Micro range, RGB mode ===
    {
        let files = collect_source_chunks(1_500_000);
        generated.push(save_polyglot(output_dir, "source_1500k", files)?);
    }

    // === SAMPLE 14: ~2.5 MiB - near Micro/no-label boundary (3M) ===
    {
        let files = collect_source_chunks(2_500_000);
        generated.push(save_polyglot(output_dir, "source_2500k", files)?);
    }

    // === SAMPLE 15: ~4 MiB - no labels, RGBA mode ===
    {
        let files = collect_source_chunks(4_000_000);
        generated.push(save_polyglot(output_dir, "source_4000k", files)?);
    }

    // === MORE VARIED SAMPLES (all real-world data) ===

    // === SAMPLE 16: Single large source file ===
    {
        let mut files = IndexMap::new();
        // Find the largest source file under 60KB
        let mut largest: Option<(String, Vec<u8>)> = None;
        for entry in walkdir::WalkDir::new("src")
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            if let Ok(data) = fs::read(entry.path()) {
                if data.len() <= 60_000 {
                    if largest.as_ref().map(|(_, d)| d.len()).unwrap_or(0) < data.len() {
                        let name = entry.path().to_string_lossy().to_string();
                        largest = Some((name, data));
                    }
                }
            }
        }
        if let Some((name, data)) = largest {
            files.insert(name.into_bytes(), data);
        }
        generated.push(save_polyglot(output_dir, "largest_source", files)?);
    }

    // === SAMPLE 17: Many small source files (polyglot module) ===
    {
        let mut files = IndexMap::new();
        collect_glob(&mut files, "src/polyglot", &["*.rs"], Some(60_000), Some("polyglot/"));
        generated.push(save_polyglot(output_dir, "polyglot_module", files)?);
    }

    // === SAMPLE 18: Text module source ===
    {
        let mut files = IndexMap::new();
        collect_glob(&mut files, "src/text", &["*.rs"], Some(60_000), Some("text/"));
        generated.push(save_polyglot(output_dir, "text_module", files)?);
    }

    // === SAMPLE 19: PNG module source ===
    {
        let mut files = IndexMap::new();
        collect_glob(&mut files, "src/png", &["*.rs"], Some(60_000), Some("png/"));
        generated.push(save_polyglot(output_dir, "png_module", files)?);
    }

    // === SAMPLE 20: All examples ===
    {
        let mut files = IndexMap::new();
        collect_glob(&mut files, "examples", &["*.rs"], Some(60_000), Some("examples/"));
        generated.push(save_polyglot(output_dir, "all_examples", files)?);
    }

    // === SAMPLE 21: System text files (shells, paths, etc) ===
    {
        let mut files = IndexMap::new();
        collect_files(&mut files, &[
            ("shells", "/etc/shells"),
            ("paths", "/etc/paths"),
            ("hosts", "/etc/hosts"),
            ("resolv.conf", "/etc/resolv.conf"),
        ], Some(50_000));
        generated.push(save_polyglot(output_dir, "etc_text", files)?);
    }

    // === SAMPLE 22: User shell configs ===
    {
        let mut files = IndexMap::new();
        collect_files(&mut files, &[
            ("bashrc", &format!("{}/.bashrc", home)),
            ("zshrc", &format!("{}/.zshrc", home)),
            ("profile", &format!("{}/.profile", home)),
            ("bash_profile", &format!("{}/.bash_profile", home)),
        ], Some(50_000));
        generated.push(save_polyglot(output_dir, "shell_configs", files)?);
    }

    // === SAMPLE 23: Deep directory structure from src ===
    {
        let mut files = IndexMap::new();
        // Collect files preserving their full paths to show directory depth
        for entry in walkdir::WalkDir::new("src")
            .max_depth(10)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .take(30)
        {
            if let Ok(data) = fs::read(entry.path()) {
                if data.len() <= 60_000 {
                    files.insert(entry.path().to_string_lossy().into_owned().into_bytes(), data);
                }
            }
        }
        generated.push(save_polyglot(output_dir, "deep_source_tree", files)?);
    }

    // === SAMPLE 24: Long path names from source ===
    {
        let mut files = IndexMap::new();
        // Find files with longest paths
        let mut entries: Vec<_> = walkdir::WalkDir::new("src")
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .collect();
        entries.sort_by_key(|e| std::cmp::Reverse(e.path().to_string_lossy().len()));
        for entry in entries.into_iter().take(15) {
            if let Ok(data) = fs::read(entry.path()) {
                if data.len() <= 60_000 {
                    files.insert(entry.path().to_string_lossy().into_owned().into_bytes(), data);
                }
            }
        }
        generated.push(save_polyglot(output_dir, "long_paths", files)?);
    }

    // === SAMPLE 25: Config file formats (Cargo.toml, json, etc) ===
    {
        let mut files = IndexMap::new();
        collect_files(&mut files, &[
            ("Cargo.toml", "Cargo.toml"),
            ("Cargo.lock", "Cargo.lock"),
        ], None);
        // Also collect any .json files from src/text
        collect_glob(&mut files, "src/text", &["*.json"], Some(60_000), Some(""));
        generated.push(save_polyglot(output_dir, "config_formats", files)?);
    }

    // === SAMPLE 26: Rust source subset ===
    {
        let mut files = IndexMap::new();
        collect_files(&mut files, &[
            ("lib.rs", "src/lib.rs"),
            ("polyglot/mod.rs", "src/polyglot/mod.rs"),
        ], Some(50000));
        generated.push(save_polyglot(output_dir, "rust_subset", files)?);
    }

    // === SAMPLE 27: Just Cargo files ===
    {
        let mut files = IndexMap::new();
        collect_files(&mut files, &[
            ("Cargo.toml", "Cargo.toml"),
            ("Cargo.lock", "Cargo.lock"),
        ], None);
        generated.push(save_polyglot(output_dir, "cargo_files", files)?);
    }

    // === SAMPLE 28: Font sprites ===
    {
        let mut files = IndexMap::new();
        collect_glob(&mut files, "src/text", &["*.png"], Some(60_000), Some("sprites/"));
        generated.push(save_polyglot(output_dir, "font_sprites", files)?);
    }

    // === SAMPLE 29: Font metadata ===
    {
        let mut files = IndexMap::new();
        collect_glob(&mut files, "src/text", &["*.json"], Some(60_000), Some("meta/"));
        generated.push(save_polyglot(output_dir, "font_metadata", files)?);
    }

    // === SAMPLE 30: ~64K boundary test ===
    {
        let mut files = IndexMap::new();
        files.insert(b"before_boundary.bin".to_vec(), vec![0x11; 30000]);
        files.insert(b"crosses_boundary.bin".to_vec(), vec![0x22; 40000]);
        files.insert(b"after_boundary.bin".to_vec(), vec![0x33; 30000]);
        generated.push(save_polyglot(output_dir, "boundary_test", files)?);
    }

    // === SAMPLE 31: Repeated content ===
    {
        let mut files = IndexMap::new();
        let repeated = vec![0x42; 5000];
        for i in 0..20 {
            files.insert(format!("repeat_{:02}.dat", i).into_bytes(), repeated.clone());
        }
        generated.push(save_polyglot(output_dir, "repeated_content", files)?);
    }

    // === SAMPLE 32: Random-ish content ===
    {
        let mut files = IndexMap::new();
        for i in 0..15 {
            // Pseudo-random using simple LCG
            let mut val = (i as u32).wrapping_mul(1103515245).wrapping_add(12345);
            let content: Vec<u8> = (0..8000).map(|_| {
                val = val.wrapping_mul(1103515245).wrapping_add(12345);
                (val >> 16) as u8
            }).collect();
            files.insert(format!("random_{:02}.bin", i).into_bytes(), content);
        }
        generated.push(save_polyglot(output_dir, "pseudorandom", files)?);
    }

    // === SAMPLE 33: All zeros ===
    {
        let mut files = IndexMap::new();
        for i in 0..10 {
            files.insert(format!("zeros_{:02}.bin", i).into_bytes(), vec![0x00; 8000]);
        }
        generated.push(save_polyglot(output_dir, "all_zeros", files)?);
    }

    // === SAMPLE 34: All ones (0xFF) ===
    {
        let mut files = IndexMap::new();
        for i in 0..10 {
            files.insert(format!("ones_{:02}.bin", i).into_bytes(), vec![0xFF; 8000]);
        }
        generated.push(save_polyglot(output_dir, "all_ones", files)?);
    }

    // === SAMPLE 35: Alternating bytes ===
    {
        let mut files = IndexMap::new();
        for i in 0..10 {
            let content: Vec<u8> = (0..8000).map(|j| if j % 2 == 0 { 0x55 } else { 0xAA }).collect();
            files.insert(format!("alt_{:02}.bin", i).into_bytes(), content);
        }
        generated.push(save_polyglot(output_dir, "alternating", files)?);
    }

    // === SAMPLE 36: Source with examples ===
    {
        let mut files = IndexMap::new();
        collect_glob(&mut files, "src/polyglot", &["*.rs"], Some(60_000), Some("src/"));
        collect_glob(&mut files, "examples", &["*.rs"], Some(60_000), Some("examples/"));
        generated.push(save_polyglot(output_dir, "src_and_examples", files)?);
    }

    // === SAMPLE 37: PNG palette files (filtered to <60KB each) ===
    {
        let mut files = IndexMap::new();
        collect_glob(&mut files, "src/png/palettes", &["*.rs"], Some(60_000), Some("palettes/"));
        generated.push(save_polyglot(output_dir, "palette_source", files)?);
    }

    // === SAMPLE 38: Git objects only ===
    {
        let mut files = IndexMap::new();
        collect_git_objects(&mut files, ".git/objects", 30, 30000);
        if !files.is_empty() {
            generated.push(save_polyglot(output_dir, "git_objects", files)?);
        }
    }

    // === SAMPLE 39: Incrementing bytes ===
    {
        let mut files = IndexMap::new();
        for i in 0..8 {
            let content: Vec<u8> = (0..10000).map(|j| (j % 256) as u8).collect();
            files.insert(format!("incr_{}.bin", i).into_bytes(), content);
        }
        generated.push(save_polyglot(output_dir, "incrementing", files)?);
    }

    // === SAMPLE 40: Empty-ish files ===
    {
        let mut files = IndexMap::new();
        files.insert(b"one_byte.txt".to_vec(), b"X".to_vec());
        files.insert(b"two_bytes.txt".to_vec(), b"XY".to_vec());
        files.insert(b"three_bytes.txt".to_vec(), b"XYZ".to_vec());
        for i in 1..=10 {
            files.insert(format!("bytes_{:02}.txt", i).into_bytes(), vec![b'.' ; i]);
        }
        generated.push(save_polyglot(output_dir, "minimal_files", files)?);
    }

    // === NETWORK SAMPLES (optional) ===
    if include_network {
        println!("\nFetching data from the internet...\n");

        // === SAMPLE 6: Literature and text ===
        {
            let mut files = IndexMap::new();

            // Project Gutenberg texts
            if let Ok(data) = fetch_url("https://www.gutenberg.org/cache/epub/1041/pg1041.txt", Some(100000)) {
                files.insert(b"literature/shakespeare_sonnets.txt".to_vec(), data);
            }
            if let Ok(data) = fetch_url("https://www.gutenberg.org/files/1342/1342-0.txt", Some(100000)) {
                files.insert(b"literature/pride_and_prejudice.txt".to_vec(), data);
            }
            if let Ok(data) = fetch_url("https://www.gutenberg.org/files/84/84-0.txt", Some(100000)) {
                files.insert(b"literature/frankenstein.txt".to_vec(), data);
            }

            if !files.is_empty() {
                generated.push(save_polyglot(output_dir, "literature", files)?);
            }
        }

        // === SAMPLE 7: API data (JSON/CSV) ===
        {
            let mut files = IndexMap::new();

            // JSONPlaceholder
            if let Ok(data) = fetch_url("https://jsonplaceholder.typicode.com/users", None) {
                files.insert(b"api/users.json".to_vec(), data);
            }
            if let Ok(data) = fetch_url("https://jsonplaceholder.typicode.com/posts", None) {
                files.insert(b"api/posts.json".to_vec(), data);
            }
            if let Ok(data) = fetch_url("https://jsonplaceholder.typicode.com/comments", None) {
                files.insert(b"api/comments.json".to_vec(), data);
            }

            // USGS earthquake data
            if let Ok(data) = fetch_url("https://earthquake.usgs.gov/earthquakes/feed/v1.0/summary/all_week.csv", Some(200000)) {
                files.insert(b"data/earthquakes_week.csv".to_vec(), data);
            }

            if !files.is_empty() {
                generated.push(save_polyglot(output_dir, "api_data", files)?);
            }
        }

        // === SAMPLE 8: Images ===
        {
            let mut files = IndexMap::new();

            // Random photos from Lorem Picsum
            for (i, size) in [(400, 300), (300, 400), (500, 500), (600, 400)].iter().enumerate() {
                let url = format!("https://picsum.photos/{}/{}", size.0, size.1);
                if let Ok(data) = fetch_url(&url, None) {
                    files.insert(format!("photos/photo_{}.jpg", i + 1).into_bytes(), data);
                }
            }

            if !files.is_empty() {
                generated.push(save_polyglot(output_dir, "photos", files)?);
            }
        }
    }

    // Filter out empty/failed samples
    let generated: Vec<_> = generated.into_iter().filter(|(_, size, _)| *size > 0).collect();

    // Print summary
    println!("\n{}", "=".repeat(75));
    println!("Generated {} sample files:\n", generated.len());
    println!("{:<25} {:>12} {}", "Name", "Size", "Description");
    println!("{}", "-".repeat(75));
    for (name, size, desc) in &generated {
        println!("{:<25} {:>12} {}", name, format_size(*size), desc);
    }

    let total_size: usize = generated.iter().map(|(_, s, _)| s).sum();
    println!("{}", "-".repeat(75));
    println!("{:<25} {:>12}", "TOTAL", format_size(total_size));

    println!("\nAll files saved to {}/", output_dir);
    if !include_network {
        println!("\nTip: Run with --include-network for additional samples from the internet");
    }

    Ok(())
}

fn collect_files(files: &mut IndexMap<Vec<u8>, Vec<u8>>, paths: &[(&str, &str)], max_size: Option<usize>) {
    for (archive_path, local_path) in paths {
        if let Ok(data) = fs::read(local_path) {
            // Skip files over max_size (60KB limit for polyglot compatibility)
            if let Some(max) = max_size {
                if data.len() > max {
                    continue;
                }
            }
            if !data.is_empty() {
                files.insert(archive_path.as_bytes().to_vec(), data);
            }
        }
    }
}

fn collect_glob(
    files: &mut IndexMap<Vec<u8>, Vec<u8>>,
    base_dir: &str,
    patterns: &[&str],
    max_size: Option<usize>,
    prefix: Option<&str>,
) {
    let prefix = prefix.unwrap_or("");

    for entry in walkdir::WalkDir::new(base_dir)
        .max_depth(5)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .take(50)
    {
        let path = entry.path();
        let path_str = path.to_string_lossy();

        let matches = patterns.iter().any(|p| {
            if p.starts_with("*.") {
                path_str.ends_with(&p[1..])
            } else {
                path_str.ends_with(p)
            }
        });

        if matches {
            if let Ok(data) = fs::read(path) {
                // Skip files over max_size (60KB limit for polyglot compatibility)
                if let Some(max) = max_size {
                    if data.len() > max {
                        continue;
                    }
                }
                if !data.is_empty() {
                    let rel_path = path.strip_prefix(base_dir)
                        .unwrap_or(path)
                        .to_string_lossy()
                        .to_string();
                    files.insert(format!("{}{}", prefix, rel_path).into_bytes(), data);
                }
            }
        }
    }
}

fn collect_git_objects(files: &mut IndexMap<Vec<u8>, Vec<u8>>, objects_dir: &str, max_count: usize, max_size: usize) {
    if !Path::new(objects_dir).exists() {
        return;
    }

    for entry in walkdir::WalkDir::new(objects_dir)
        .max_depth(2)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            let name = e.file_name().to_string_lossy();
            name.len() >= 38 // Git object files have long hex names
        })
        .take(max_count)
    {
        if let Ok(data) = fs::read(entry.path()) {
            if data.len() <= max_size {
                if let Ok(rel_path) = entry.path().strip_prefix(objects_dir) {
                    files.insert(
                        format!("git/objects/{}", rel_path.to_string_lossy()).into_bytes(),
                        data,
                    );
                }
            }
        }
    }
}

fn save_polyglot(
    output_dir: &str,
    name: &str,
    files: IndexMap<Vec<u8>, Vec<u8>>,
) -> Result<(String, usize, String), panic> {
    if files.is_empty() {
        println!("  {} - no files, skipping", name);
        return Ok((name.to_string(), 0, "skipped".to_string()));
    }

    let total_content: usize = files.values().map(|v| v.len()).sum();
    let file_count = files.len();
    let files_struct: Files = files.into();
    let polyglot = zipng::zipng(&files_struct);

    let path = format!("{}/{}.png", output_dir, name);
    fs::write(&path, &polyglot)?;

    let desc = format!("{} files, {} content", file_count, format_size(total_content));
    println!("  {:<20} {:>12} ({})", name, format_size(polyglot.len()), desc);

    Ok((name.to_string(), polyglot.len(), desc))
}

fn fetch_url(url: &str, max_size: Option<usize>) -> Result<Vec<u8>, String> {
    print!("    Fetching {}... ", &url[..url.len().min(50)]);

    let response = ureq::get(url)
        .call()
        .map_err(|e| format!("HTTP error: {}", e))?;

    let mut data = Vec::new();
    response
        .into_reader()
        .take(max_size.unwrap_or(200_000) as u64)
        .read_to_end(&mut data)
        .map_err(|e| format!("Read error: {}", e))?;

    println!("{}", format_size(data.len()));
    Ok(data)
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
