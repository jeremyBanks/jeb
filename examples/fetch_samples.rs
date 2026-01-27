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

        // All Rust source files (no size limit)
        collect_glob(&mut files, "src", &["*.rs"], None, Some("src/"));

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

        // Examples
        collect_glob(&mut files, "examples", &["*.rs"], None, Some("examples/"));

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

        // Git pack files (highly compressed binary)
        if let Ok(entries) = fs::read_dir(".git/objects/pack") {
            for entry in entries.filter_map(|e| e.ok()).take(4) {
                if let Ok(mut data) = fs::read(entry.path()) {
                    data.truncate(100000); // 100KB max per pack file
                    let name = entry.file_name().to_string_lossy().to_string();
                    files.insert(format!("pack/{}", name).into_bytes(), data);
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
        if let Ok(mut data) = fs::read(local_path) {
            if let Some(max) = max_size {
                data.truncate(max);
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
            if let Ok(mut data) = fs::read(path) {
                if let Some(max) = max_size {
                    data.truncate(max);
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
