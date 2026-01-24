use {
    _chosen::{bytes_to_text, text_to_bytes},
    anyhow::{Context, Result},
    clap::Parser,
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

#[derive(Parser)]
#[clap(name = "fuzz")]
#[clap(about = "Run fuzz tests across the workspace")]
struct Args {
    /// Only run targets containing this substring
    filter: Option<String>,

    /// Fuzz each target for N seconds (0 or negative = replay only)
    #[clap(short, long, default_value = "1")]
    seconds: i32,

    /// Maximum input length in bytes
    #[clap(long, default_value = "313")]
    max_len: u32,

    /// Only pack corpus files (no fuzzing)
    #[clap(long)]
    pack_only: bool,

    /// Only unpack corpus files (no fuzzing)
    #[clap(long)]
    unpack_only: bool,
}

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
    let args = Args::parse();

    if args.pack_only {
        return pack_all_corpora();
    }
    if args.unpack_only {
        return unpack_all_corpora();
    }

    let seconds = args.seconds;
    let max_len = args.max_len;
    let target_filter = args.filter;

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

            // Unpack corpus before fuzzing, tracking original entries for archive
            info!("[unpack]");
            let original_corpus = match unpack_corpus(&fuzz_dir, target) {
                Ok(corpus) => Some(corpus),
                Err(e) => {
                    warn!("unpack warning: {}", e);
                    None
                }
            };

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
                if let Err(e) = pack_corpus(&fuzz_dir, target, original_corpus.as_ref()) {
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
            if let Err(e) = pack_corpus(&fuzz_dir, target, original_corpus.as_ref()) {
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
        // Sort by entry_type first:
        // - Artifacts (crash, timeout, etc.) come first (most interesting)
        // - corpus comes next
        // - archive comes last (preserved historical entries)
        // Then sort by data value within each type
        let type_order = |t: &str| -> u8 {
            match t {
                "corpus" => 1,
                "archive" => 2,
                _ => 0, // artifacts come first
            }
        };
        type_order(&self.entry_type)
            .cmp(&type_order(&other.entry_type))
            .then_with(|| self.entry_type.cmp(&other.entry_type))
            .then_with(|| self.data.cmp(&other.data))
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

/// Pack corpus and artifacts into a .corpus text file.
/// If original_corpus is provided, entries that were in original_corpus but are
/// no longer on the filesystem become "archive:" entries (preserved historical data).
fn pack_corpus(
    fuzz_dir: &Path,
    target: &str,
    original_corpus: Option<&std::collections::HashSet<Vec<u8>>>,
) -> Result<()> {
    use std::collections::HashSet;

    let mut entries: BTreeSet<CorpusEntry> = BTreeSet::new();
    let mut current_corpus_data: HashSet<Vec<u8>> = HashSet::new();

    // Read corpus files
    let corpus_dir = fuzz_dir.join("corpus").join(target);
    if corpus_dir.is_dir() {
        for entry in fs::read_dir(&corpus_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let data = fs::read(&path)?;
                current_corpus_data.insert(data.clone());
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

    // Create archive entries for corpus items that disappeared after reduction
    let mut archive_entries: Vec<CorpusEntry> = Vec::new();
    if let Some(original) = original_corpus {
        for data in original {
            if !current_corpus_data.contains(data) {
                archive_entries.push(CorpusEntry {
                    entry_type: "archive".to_string(),
                    data: data.clone(),
                });
            }
        }
    }

    // Limit corpus + archive entries to 1024 (artifacts are unlimited)
    // If over limit, delete archive entries first, then corpus entries
    // Use SHA-1 hash ordering for deterministic pseudo-random deletion
    const MAX_CORPUS_ENTRIES: usize = 1024;

    let mut corpus_entries: Vec<_> = entries
        .iter()
        .filter(|e| e.entry_type == "corpus")
        .cloned()
        .collect();
    let artifact_entries: Vec<_> = entries
        .iter()
        .filter(|e| e.entry_type != "corpus")
        .cloned()
        .collect();

    // Sort by SHA-1 hash (for deterministic deletion when over limit)
    let hash_key = |e: &CorpusEntry| {
        let hash = Sha1::digest(&e.data);
        hash.iter().map(|b| format!("{:02x}", b)).collect::<String>()
    };
    corpus_entries.sort_by_cached_key(hash_key);
    archive_entries.sort_by_cached_key(hash_key);

    let mut deleted_archive = 0;
    let mut deleted_corpus = 0;
    let total_corpus_like = corpus_entries.len() + archive_entries.len();

    if total_corpus_like > MAX_CORPUS_ENTRIES {
        let to_delete = total_corpus_like - MAX_CORPUS_ENTRIES;

        // Delete archive entries first (from the beginning, lowest hash values)
        if archive_entries.len() >= to_delete {
            deleted_archive = to_delete;
            archive_entries = archive_entries.into_iter().skip(to_delete).collect();
        } else {
            // Delete all archive entries and some corpus entries
            deleted_archive = archive_entries.len();
            let remaining_to_delete = to_delete - deleted_archive;
            archive_entries.clear();
            deleted_corpus = remaining_to_delete;
            corpus_entries = corpus_entries.into_iter().skip(remaining_to_delete).collect();
        }
    }

    // Recombine and sort for output
    let mut final_entries: BTreeSet<CorpusEntry> = BTreeSet::new();
    for e in artifact_entries {
        final_entries.insert(e);
    }
    for e in corpus_entries {
        final_entries.insert(e);
    }
    for e in archive_entries {
        final_entries.insert(e);
    }

    // Write .corpus file
    let corpus_file = corpus_file_path(fuzz_dir, target);
    let content: String = final_entries
        .iter()
        .map(|e| e.to_line())
        .collect::<Vec<_>>()
        .join("\n");

    // Count entries by type
    let corpus_count = final_entries
        .iter()
        .filter(|e| e.entry_type == "corpus")
        .count();
    let archive_count = final_entries
        .iter()
        .filter(|e| e.entry_type == "archive")
        .count();
    let artifact_count = final_entries.len() - corpus_count - archive_count;
    let deleted_count = deleted_archive + deleted_corpus;

    // Only write and clean up if there's content to pack
    if !final_entries.is_empty() {
        fs::write(&corpus_file, format!("{}\n", content))?;

        // Log with appropriate detail
        let mut parts = vec![format!("{} corpus", corpus_count)];
        if archive_count > 0 {
            parts.push(format!("{} archive", archive_count));
        }
        parts.push(format!("{} artifacts", artifact_count));
        if deleted_count > 0 {
            parts.push(format!("deleted {} over limit", deleted_count));
        }
        info!(
            "  packed {} -> {}",
            parts.join(" + "),
            corpus_file.file_name().unwrap_or_default().to_string_lossy()
        );

        // Clean up directories after packing (data is now in .corpus file)
        if corpus_dir.is_dir() {
            fs::remove_dir_all(&corpus_dir)?;
        }
        if artifacts_dir.is_dir() {
            fs::remove_dir_all(&artifacts_dir)?;
        }
        let tmp_dir = fuzz_dir.join(".tmp");
        if tmp_dir.is_dir() {
            fs::remove_dir_all(&tmp_dir)?;
        }

        // Remove empty parent directories
        let corpus_parent = fuzz_dir.join("corpus");
        if corpus_parent.is_dir() && fs::read_dir(&corpus_parent)?.next().is_none() {
            fs::remove_dir(&corpus_parent)?;
        }
        let artifacts_parent = fuzz_dir.join("artifacts");
        if artifacts_parent.is_dir() && fs::read_dir(&artifacts_parent)?.next().is_none() {
            fs::remove_dir(&artifacts_parent)?;
        }
    }
    // If no entries to pack, leave existing corpus file alone

    Ok(())
}

/// Unpack a .corpus text file into corpus and artifact files.
/// Returns a set of corpus entry data (both "corpus" and "archive" types) for tracking.
fn unpack_corpus(fuzz_dir: &Path, target: &str) -> Result<std::collections::HashSet<Vec<u8>>> {
    use std::collections::HashSet;

    let corpus_file = corpus_file_path(fuzz_dir, target);
    let mut original_corpus: HashSet<Vec<u8>> = HashSet::new();

    if !corpus_file.exists() {
        return Ok(original_corpus); // Nothing to unpack
    }

    let content = fs::read_to_string(&corpus_file)?;
    let corpus_dir = fuzz_dir.join("corpus").join(target);
    let artifacts_dir = fuzz_dir.join("artifacts").join(target);

    let mut corpus_count = 0;
    let mut artifact_count = 0;
    let mut archive_count = 0;
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

        if entry.entry_type == "corpus" || entry.entry_type == "archive" {
            // Track corpus and archive entries for archive purposes
            original_corpus.insert(entry.data.clone());
            // Both corpus and archive entries are written as corpus files
            fs::create_dir_all(&corpus_dir)?;
            let file_path = corpus_dir.join(&hash_hex);
            if !file_path.exists() {
                fs::write(&file_path, &entry.data)?;
                if entry.entry_type == "corpus" {
                    corpus_count += 1;
                } else {
                    archive_count += 1;
                }
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

    if archive_count > 0 {
        info!(
            "  unpacked {} corpus, {} artifacts, {} archive ({} already exist)",
            corpus_count, artifact_count, archive_count, skipped_count
        );
    } else {
        info!(
            "  unpacked {} corpus, {} artifacts ({} already exist)",
            corpus_count, artifact_count, skipped_count
        );
    }

    Ok(original_corpus)
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
                // When using --pack-only, we don't have original corpus info
                if let Err(e) = pack_corpus(&fuzz_dir, target, None) {
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
