use {
    anyhow::{Context, Result},
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
            eprintln!("Error: {e:?}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<bool> {
    // Parse arguments
    let mut seconds: i32 = 2;
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
                    eprintln!("-s requires a value");
                    return Ok(false);
                }
            } else {
                // -sN form
                seconds = value.parse().context("invalid -s value")?;
            }
        } else if arg == "--help" || arg == "-h" {
            println!("Usage: _fuzz [OPTIONS] [FILTER]");
            println!();
            println!("Arguments:");
            println!("  [FILTER]            Only run targets containing this substring");
            println!();
            println!("Options:");
            println!("  -s, --seconds=N     Fuzz each target for N seconds (default: 2)");
            println!("                      If N <= 0, only replay corpus (no fuzzing)");
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
            eprintln!("Unknown argument: {}", arg);
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
        println!("No fuzz directories found");
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
            eprintln!(
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
            println!("No fuzz targets found in {}", crate_name);
            continue;
        }

        println!("\n=== Fuzzing crate: {} ===\n", crate_name);
        println!(
            "Found {} target(s): {}",
            targets.len(),
            targets.join(", ")
        );

        for target in &targets {
            println!("\n--- {} ---", target);

            // Unpack corpus before fuzzing
            if let Err(e) = unpack_corpus(&fuzz_dir, target) {
                eprintln!("Warning: failed to unpack corpus: {}", e);
            }

            // Run fuzzing or corpus replay
            let status = if seconds > 0 {
                println!("Fuzzing for {} seconds...", seconds);
                Command::new("cargo")
                    .args([
                        "+nightly",
                        "fuzz",
                        "run",
                        target,
                        "--",
                        &format!("-seconds={}", seconds),
                    ])
                    .current_dir(&fuzz_dir)
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .status()
                    .context("failed to run cargo fuzz run")?
            } else {
                println!("Replaying corpus...");
                Command::new("cargo")
                    .args(["+nightly", "fuzz", "run", target, "--", "-runs=0"])
                    .current_dir(&fuzz_dir)
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .status()
                    .context("failed to run cargo fuzz run")?
            };

            if !status.success() {
                eprintln!("Fuzzing {} failed with status: {}", target, status);
                any_failed = true;
                // Still try to pack corpus on failure
                if let Err(e) = pack_corpus(&fuzz_dir, target) {
                    eprintln!("Warning: failed to pack corpus: {}", e);
                }
                continue;
            }

            // Run corpus minimization (only if we did actual fuzzing)
            if seconds > 0 {
                println!("\nMinimizing corpus...");
                // Set TMPDIR to fuzz dir to avoid cross-device link errors
                // Use absolute path to avoid issues with cargo fuzz cmin
                let tmp_dir = fuzz_dir.join(".tmp");
                fs::create_dir_all(&tmp_dir).ok();
                let tmp_dir = tmp_dir.canonicalize().unwrap_or(tmp_dir);
                let status = Command::new("cargo")
                    .args(["+nightly", "fuzz", "cmin", target])
                    .env("TMPDIR", &tmp_dir)
                    .current_dir(&fuzz_dir)
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .status()
                    .context("failed to run cargo fuzz cmin")?;

                if !status.success() {
                    eprintln!(
                        "Corpus minimization for {} failed with status: {}",
                        target, status
                    );
                    any_failed = true;
                }
            }

            // Pack corpus after fuzzing
            if let Err(e) = pack_corpus(&fuzz_dir, target) {
                eprintln!("Warning: failed to pack corpus: {}", e);
            }
        }
    }

    if any_failed {
        println!("\n=== Fuzz smoke test complete (with failures) ===");
    } else {
        println!("\n=== Fuzz smoke test complete ===");
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
    /// None for corpus, Some("crash"), Some("timeout"), etc. for artifacts
    artifact_type: Option<String>,
    /// Raw bytes of the input
    data: Vec<u8>,
}

impl CorpusEntry {
    /// Encode bytes: printable ASCII (0x21-0x7E) as " X", others as "XX" hex
    fn to_encoded(&self) -> String {
        self.data
            .iter()
            .map(|&b| {
                if (0x21..=0x7E).contains(&b) {
                    // Printable ASCII (excluding space): space + character (maintains 2-char width)
                    format!(" {}", b as char)
                } else {
                    // Non-printable or space: uppercase hex
                    format!("{:02X}", b)
                }
            })
            .collect::<String>()
    }

    /// Decode bytes: " X" is literal char, "XX" is hex pair
    fn from_encoded(encoded: &str) -> Result<Vec<u8>> {
        let mut result = Vec::new();
        let chars: Vec<char> = encoded.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            if chars[i] == ' ' {
                // Space followed by literal character
                if i + 1 >= chars.len() {
                    anyhow::bail!("trailing space at end of encoded data");
                }
                let c = chars[i + 1];
                // Accept 0x20-0x7E for backwards compat, but we only emit 0x21-0x7E
                if !c.is_ascii() || (c as u8) < 0x20 || (c as u8) > 0x7E {
                    anyhow::bail!("invalid literal character after space: {:?}", c);
                }
                result.push(c as u8);
                i += 2;
            } else {
                // Two hex digits
                if i + 1 >= chars.len() {
                    anyhow::bail!("odd number of hex characters");
                }
                let hex: String = chars[i..i + 2].iter().collect();
                let byte = u8::from_str_radix(&hex, 16)
                    .with_context(|| format!("invalid hex pair: {:?}", hex))?;
                result.push(byte);
                i += 2;
            }
        }

        Ok(result)
    }

    /// Parse a line from .corpus file
    fn parse_line(line: &str) -> Result<Self> {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            anyhow::bail!("empty or comment line");
        }

        let (artifact_type, encoded) = if let Some(rest) = line.strip_prefix(':') {
            // Corpus entry: ":DATA"
            (None, rest)
        } else if let Some(colon_pos) = line.find(':') {
            // Artifact entry: "type:DATA"
            let (typ, rest) = line.split_at(colon_pos);
            (Some(typ.to_string()), &rest[1..])
        } else {
            anyhow::bail!("line missing colon separator");
        };

        let data = Self::from_encoded(encoded)?;
        Ok(CorpusEntry {
            artifact_type,
            data,
        })
    }

    /// Format as a line for .corpus file
    fn to_line(&self) -> String {
        match &self.artifact_type {
            None => format!(":{}", self.to_encoded()),
            Some(typ) => format!("{}:{}", typ, self.to_encoded()),
        }
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
                    artifact_type: None,
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
                        let artifact_type = &name[..dash_pos];
                        let data = fs::read(&path)?;
                        entries.insert(CorpusEntry {
                            artifact_type: Some(artifact_type.to_string()),
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

    // Only write if there's content, and add trailing newline
    if !entries.is_empty() {
        fs::write(&corpus_file, format!("{}\n", content))?;
        println!("Packed {} entries to {}", entries.len(), corpus_file.display());
    } else if corpus_file.exists() {
        // Remove empty corpus file
        fs::remove_file(&corpus_file)?;
        println!("Removed empty corpus file {}", corpus_file.display());
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

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let entry = match CorpusEntry::parse_line(line) {
            Ok(e) => e,
            Err(e) => {
                eprintln!("Warning: skipping invalid line: {}", e);
                continue;
            }
        };

        // Compute SHA-1 hash for filename
        let hash = Sha1::digest(&entry.data);
        let hash_hex = hash
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();

        match &entry.artifact_type {
            None => {
                // Corpus entry
                fs::create_dir_all(&corpus_dir)?;
                let file_path = corpus_dir.join(&hash_hex);
                if !file_path.exists() {
                    fs::write(&file_path, &entry.data)?;
                    corpus_count += 1;
                }
            }
            Some(typ) => {
                // Artifact entry
                fs::create_dir_all(&artifacts_dir)?;
                let file_path = artifacts_dir.join(format!("{}-{}", typ, hash_hex));
                if !file_path.exists() {
                    fs::write(&file_path, &entry.data)?;
                    artifact_count += 1;
                }
            }
        }
    }

    if corpus_count > 0 || artifact_count > 0 {
        println!(
            "Unpacked {} corpus + {} artifact entries from {}",
            corpus_count,
            artifact_count,
            corpus_file.display()
        );
    }

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
                    eprintln!("Error packing {}: {}", target, e);
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
                    eprintln!("Error unpacking {}: {}", target, e);
                }
            }
        }
    }

    Ok(true)
}
