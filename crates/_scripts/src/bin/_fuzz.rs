use {
    anyhow::{Context, Result},
    std::{
        env,
        path::PathBuf,
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
    // Parse --max-total-time=N argument (default 4 seconds, 0 or negative = replay only)
    let mut max_total_time: i32 = 4;
    for arg in env::args().skip(1) {
        if let Some(value) = arg.strip_prefix("--max-total-time=") {
            max_total_time = value.parse().context("invalid --max-total-time value")?;
        } else if arg == "--help" || arg == "-h" {
            println!("Usage: _fuzz [--max-total-time=SECONDS]");
            println!();
            println!("Options:");
            println!("  --max-total-time=N  Fuzz each target for N seconds (default: 4)");
            println!("                      If N <= 0, only replay corpus (no fuzzing)");
            return Ok(true);
        } else {
            eprintln!("Unknown argument: {}", arg);
            return Ok(false);
        }
    }

    let workspace_root = env::var("CARGO_WORKSPACE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));

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

        println!("\n=== Fuzzing crate: {} ===\n", crate_name);

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
            .collect();

        if targets.is_empty() {
            println!("No fuzz targets found in {}", crate_name);
            continue;
        }

        println!("Found {} targets: {}", targets.len(), targets.join(", "));

        for target in &targets {
            println!("\n--- {} ---", target);

            // Run fuzzing or corpus replay
            let status = if max_total_time > 0 {
                println!("Fuzzing for {} seconds...", max_total_time);
                Command::new("cargo")
                    .args([
                        "+nightly",
                        "fuzz",
                        "run",
                        target,
                        "--",
                        &format!("-max_total_time={}", max_total_time),
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
                continue;
            }

            // Run corpus minimization (only if we did actual fuzzing)
            if max_total_time > 0 {
                println!("\nMinimizing corpus...");
                // Set TMPDIR to fuzz dir to avoid cross-device link errors
                let tmp_dir = fuzz_dir.join(".tmp");
                std::fs::create_dir_all(&tmp_dir).ok();
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
        }
    }

    if any_failed {
        println!("\n=== Fuzz smoke test complete (with failures) ===");
    } else {
        println!("\n=== Fuzz smoke test complete ===");
    }

    Ok(!any_failed)
}
