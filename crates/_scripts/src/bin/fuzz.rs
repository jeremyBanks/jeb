use {
    _chosen::{bytes_to_text, text_to_bytes},
    anyhow::{Context, Result},
    jeb_tracing::{error, info, warn},
    sha1::{Digest, Sha1},
    std::{
        collections::BTreeSet,
        env,
        fs,
        path::{Path, PathBuf},
        process::{Command, ExitCode, Stdio},
    },
};

fn main() -> ExitCode {
    match run() {
        Ok(success) => {
            if success {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Err(e) => {
            error!("Error: {e:?}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<bool> {
    // Parse arguments
    let mut seconds: i32 = 1;
    let mut max_len: u32 = 313;
    let mut target_filter: Option<String> = None;

    let args: Vec<String> = env::args().skip(1).collect();
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if let Some(value) = arg.strip_prefix("--seconds=") {
            seconds = value.parse().context("invalid --seconds value")?;
        } else if let Some(value) = arg.strip_prefix("-s=") {
            seconds = value.parse().context("invalid -s value")?;
        } else if let Some(value) = arg.strip_prefix("-s") {
            if value.is_empty() {
                // -s N form
                i += 1;
                if i < args.len() {
                    seconds = args[i].parse().context("invalid -s value")?;
                } else {
                    error!("-s requires a value");
                    return Ok(false);
                }
            } else {
                // -sN form
                seconds = value.parse().context("invalid -s value")?;
            }
        } else if let Some(value) = arg.strip_prefix("--max-len=") {
            max_len = value.parse().context("invalid --max-len value")?;
        } else if arg == "--help" || arg == "-h" {
            println!("Usage: _fuzz [OPTIONS] [FILTER]");
            println!();
            println!("Arguments:");
            println!("  [FILTER]            Only run targets containing this substring");
            println!();
            println!("Options:");
            println!("  -s, --seconds=N     Fuzz each target for N seconds (default: 1)");
            println!("                      If N <= 0, only replay corpus (no fuzzing)");
            println!("  --max-len=N         Maximum input length in bytes (default: 313)");
            println!("  --pack-only         Only pack corpus files (no fuzzing)");
            println!("  --unpack-only       Only unpack corpus files (no fuzzing)");
            return Ok(true);
        } else if arg == "--pack-only" {
            return pack_all_corpora();
        } else if arg == "--unpack-only" {
            return unpack_all_corpora();
        } else if !arg.starts_with('-') {
            target_filter = Some(arg.clone());
        } else {
            error!("Unknown argument: {}", arg);
            return Ok(false);
        }
        i += 1;
    }

    let workspace_root = get_workspace_root();

    // Find all crates with fuzz directories
    let fuzz_dirs: Vec<PathBuf> = glob::glob(
        workspace_root
            .join("crates/*/fuzz")
            .to_str()
            .expect("valid path"),
    )?
    .filter_map(|p| p.ok())
    .filter(|p| p.is_dir())
    .collect();

    if fuzz_dirs.is_empty() {
        info!("No fuzz directories found");
        return Ok(true);
    }

    let mut any_failed = false;

    for fuzz_dir in fuzz_dirs {
        let crate_dir = fuzz_dir.parent().expect("fuzz dir has parent");
        let crate_name = crate_dir
            .file_name()
            .expect("crate dir has name")
            .to_string_lossy();

        // Get list of fuzz targets by running cargo fuzz list
        let output = Command::new("cargo")
            .args(["+nightly", "fuzz", "list"])
            .current_dir(&fuzz_dir)
            .output()
            .context("failed to run cargo fuzz list")?;

        if !output.status.success() {
            warn!(
                "cargo fuzz list failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            any_failed = true;
            continue;
        }

        let targets: Vec<String> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .filter(|s| {
                target_filter
                    .as_ref()
                    .map(|f| s.contains(f.as_str()))
                    .unwrap_or(true)
            })
            .collect();

        if targets.is_empty() {
            if target_filter.is_some() {
                // Silent skip if filtering
                continue;
            }
            info!("No fuzz targets found in {}", crate_name);
            continue;
        }

        info!("=== Fuzzing crate: {} ===", crate_name);
        info!(
            "Found {} target(s): {}",
            targets.len(),
            targets.join(", ")
        );

        for target in &targets {
            info!("--- {}/{} ---", crate_name, target);

            // Unpack corpus before fuzzing
            info!("[unpack]");
            if let Err(e) = unpack_corpus(&fuzz_dir, target) {
                warn!("unpack warning: {}", e);
            }

            // Run fuzzing or corpus replay
            let status = if seconds > 0 {
                info!("[fuzz] running for {}s...", seconds);
                Command::new("cargo")
                    .args([
                        "+nightly",
                        "fuzz",
                        "run",
                        target,
                        "--",
                        &format!("-max_total_time={}", seconds),
                        &format!("-max_len={}", max_len),
                    ])
                    .current_dir(&fuzz_dir)
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .status()
                    .context("failed to run cargo fuzz run")?
            } else {
                info!("[replay] replaying corpus only...");
                Command::new("cargo")
                    .args([
                        "+nightly",
                        "fuzz",
                        "run",
                        target,
                        "--",
                        "-runs=0",
                        &format!("-max_len={}", max_len),
                    ])
                    .current_dir(&fuzz_dir)
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .status()
                    .context("failed to run cargo fuzz run")?
            };

            if !status.success() {
                warn!("[fuzz] FAILED (exit {})", status);
                any_failed = true;
                // Still try to pack corpus on failure
                info!("[pack]");
                if let Err(e) = pack_corpus(&fuzz_dir, target) {
                    warn!("pack warning: {}", e);
                }
                continue;
            }

            // Run corpus minimization (only if we did actual fuzzing)
            if seconds > 0 {
                info!("[cmin] minimizing corpus...");
                // Set TMPDIR to fuzz dir to avoid cross-device link errors
                // Use absolute path to avoid issues with cargo fuzz cmin
                let tmp_dir = fuzz_dir.join(".tmp");
                fs::create_dir_all(&tmp_dir).ok();
                let tmp_dir = tmp_dir.canonicalize().unwrap_or(tmp_dir);
                let status = Command::new("cargo")
                    .args([
                        "+nightly",
                        "fuzz",
                        "cmin",
                        target,
                        "--",
                        &format!("-max_len={}", max_len),
                    ])
                    .env("TMPDIR", &tmp_dir)
                    .current_dir(&fuzz_dir)
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .status()
                    .context("failed to run cargo fuzz cmin")?;

                if !status.success() {
                    warn!("[cmin] FAILED");
                    any_failed = true;
                }
            }

            // Pack corpus after fuzzing
            info!("[pack]");
            if let Err(e) = pack_corpus(&fuzz_dir, target) {
                warn!("pack warning: {}", e);
            }
        }
    }

    if any_failed {
        warn!("=== Fuzz complete (with failures) ===");
    } else {
        info!("=== Fuzz complete ===");
    }

    Ok(!any_failed)
}

fn get_workspace_root() -> PathBuf {
    env::var("CARGO_WORKSPACE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

/// Entry in a .corpus file
#[derive(Debug, Clone, PartialEq, Eq)]
struct CorpusEntry {
    /// "corpus" for corpus entries, or artifact type like "crash", "timeout", etc.
    entry_type: String,
    /// Raw bytes of the input
    data: Vec<u8>,
}

impl CorpusEntry {
    /// Parse a line from .corpus file
    /// Format: TYPE:DATA (e.g., "corpus:Hello\nWorld" or "crash:\x00\xFF")
    fn parse_line(line: &str) -> Result<Self> {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            anyhow::bail!("empty or comment line");
        }

        let (entry_type, encoded) = line.split_once(':').context("missing colon separator")?;

        // Validate entry_type is a simple identifier (alphanumeric + underscore)
        if entry_type.is_empty()
            || !entry_type
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            anyhow::bail!("invalid entry type: {}", entry_type);
        }

        let data = text_to_bytes(encoded).context("invalid encoded data")?;

        Ok(CorpusEntry {
            entry_type: entry_type.to_string(),
            data,
        })
    }

    /// Format as a line for .corpus file
    /// Format: TYPE:DATA
    fn to_line(&self) -> String {
        let encoded = bytes_to_text(&self.data);
        format!("{}:{}", self.entry_type, encoded)
    }
}

impl Ord for CorpusEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Sort by hex value (data), ignoring artifact_type
        self.data.cmp(&other.data)
    }
}

impl PartialOrd for CorpusEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Get the path to the .corpus file for a target
fn corpus_file_path(fuzz_dir: &Path, target: &str) -> PathBuf {
    fuzz_dir.join("fuzz_targets").join(format!("{}.corpus", target))
}

/// Pack corpus and artifacts into a .corpus text file
fn pack_corpus(fuzz_dir: &Path, target: &str) -> Result<()> {
    let mut entries: BTreeSet<CorpusEntry> = BTreeSet::new();

    // Read corpus files
    let corpus_dir = fuzz_dir.join("corpus").join(target);
    if corpus_dir.is_dir() {
        for entry in fs::read_dir(&corpus_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let data = fs::read(&path)?;
                entries.insert(CorpusEntry {
                    entry_type: "corpus".to_string(),
                    data,
                });
            }
        }
    }

    // Read artifact files (crash-*, timeout-*, leak-*, oom-*)
    let artifacts_dir = fuzz_dir.join("artifacts").join(target);
    if artifacts_dir.is_dir() {
        for entry in fs::read_dir(&artifacts_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    // Parse artifact type from filename (e.g., "crash-abc123")
                    if let Some(dash_pos) = name.find('-') {
                        let entry_type = &name[..dash_pos];
                        // Validate entry_type is a simple identifier
                        if entry_type.is_empty()
                            || !entry_type
                                .chars()
                                .all(|c| c.is_ascii_alphanumeric() || c == '_')
                        {
                            warn!("skipping artifact with invalid type: {:?}", name);
                            continue;
                        }
                        let data = fs::read(&path)?;
                        entries.insert(CorpusEntry {
                            entry_type: entry_type.to_string(),
                            data,
                        });
                    }
                }
            }
        }
    }

    // Write .corpus file
    let corpus_file = corpus_file_path(fuzz_dir, target);
    let content: String = entries.iter().map(|e| e.to_line()).collect::<Vec<_>>().join("\n");

    // Count corpus vs artifacts
    let corpus_count = entries
        .iter()
        .filter(|e| e.entry_type == "corpus")
        .count();
    let artifact_count = entries.len() - corpus_count;

    // Only write if there's content, and add trailing newline
    if !entries.is_empty() {
        fs::write(&corpus_file, format!("{}\n", content))?;
        info!(
            "  packed {} corpus + {} artifacts -> {}",
            corpus_count,
            artifact_count,
            corpus_file.file_name().unwrap_or_default().to_string_lossy()
        );
    } else if corpus_file.exists() {
        // Remove empty corpus file
        fs::remove_file(&corpus_file)?;
        info!("  removed empty corpus file");
    } else {
        info!("  no entries to pack");
    }

    Ok(())
}

/// Unpack a .corpus text file into corpus and artifact files
fn unpack_corpus(fuzz_dir: &Path, target: &str) -> Result<()> {
    let corpus_file = corpus_file_path(fuzz_dir, target);
    if !corpus_file.exists() {
        return Ok(()); // Nothing to unpack
    }

    let content = fs::read_to_string(&corpus_file)?;
    let corpus_dir = fuzz_dir.join("corpus").join(target);
    let artifacts_dir = fuzz_dir.join("artifacts").join(target);

    let mut corpus_count = 0;
    let mut artifact_count = 0;
    let mut skipped_count = 0;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let entry = match CorpusEntry::parse_line(line) {
            Ok(e) => e,
            Err(e) => {
                warn!("skipping invalid line: {}", e);
                continue;
            }
        };

        // Compute SHA-1 hash for filename
        let hash = Sha1::digest(&entry.data);
        let hash_hex = hash
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();

        if entry.entry_type == "corpus" {
            // Corpus entry
            fs::create_dir_all(&corpus_dir)?;
            let file_path = corpus_dir.join(&hash_hex);
            if !file_path.exists() {
                fs::write(&file_path, &entry.data)?;
                corpus_count += 1;
            } else {
                skipped_count += 1;
            }
        } else {
            // Artifact entry
            fs::create_dir_all(&artifacts_dir)?;
            let file_path = artifacts_dir.join(format!("{}-{}", entry.entry_type, hash_hex));
            if !file_path.exists() {
                fs::write(&file_path, &entry.data)?;
                artifact_count += 1;
            } else {
                skipped_count += 1;
            }
        }
    }

    info!(
        "  unpacked {} corpus, {} artifacts ({} already exist)",
        corpus_count, artifact_count, skipped_count
    );

    Ok(())
}

/// Pack all corpora in the workspace
fn pack_all_corpora() -> Result<bool> {
    let workspace_root = get_workspace_root();
    let fuzz_dirs: Vec<PathBuf> = glob::glob(
        workspace_root
            .join("crates/*/fuzz")
            .to_str()
            .expect("valid path"),
    )?
    .filter_map(|p| p.ok())
    .filter(|p| p.is_dir())
    .collect();

    for fuzz_dir in fuzz_dirs {
        let output = Command::new("cargo")
            .args(["+nightly", "fuzz", "list"])
            .current_dir(&fuzz_dir)
            .output()?;

        if !output.status.success() {
            continue;
        }

        for target in String::from_utf8_lossy(&output.stdout).lines() {
            let target = target.trim();
            if !target.is_empty() {
                if let Err(e) = pack_corpus(&fuzz_dir, target) {
                    warn!("Error packing {}: {}", target, e);
                }
            }
        }
    }

    Ok(true)
}

/// Unpack all corpora in the workspace
fn unpack_all_corpora() -> Result<bool> {
    let workspace_root = get_workspace_root();
    let fuzz_dirs: Vec<PathBuf> = glob::glob(
        workspace_root
            .join("crates/*/fuzz")
            .to_str()
            .expect("valid path"),
    )?
    .filter_map(|p| p.ok())
    .filter(|p| p.is_dir())
    .collect();

    for fuzz_dir in fuzz_dirs {
        let output = Command::new("cargo")
            .args(["+nightly", "fuzz", "list"])
            .current_dir(&fuzz_dir)
            .output()?;

        if !output.status.success() {
            continue;
        }

        for target in String::from_utf8_lossy(&output.stdout).lines() {
            let target = target.trim();
            if !target.is_empty() {
                if let Err(e) = unpack_corpus(&fuzz_dir, target) {
                    warn!("Error unpacking {}: {}", target, e);
                }
            }
        }
    }

    Ok(true)
}
