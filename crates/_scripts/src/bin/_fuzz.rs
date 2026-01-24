use {
    anyhow::{Context, Result},
    std::{
        env,
        path::PathBuf,
        process::{Command, Stdio},
    },
};

fn main() -> Result<()> {
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
        return Ok(());
    }

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

            // Run fuzzing for 20 seconds
            println!("Fuzzing for 20 seconds...");
            let status = Command::new("cargo")
                .args([
                    "+nightly",
                    "fuzz",
                    "run",
                    target,
                    "--",
                    "-max_total_time=20",
                ])
                .current_dir(&fuzz_dir)
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status()
                .context("failed to run cargo fuzz run")?;

            if !status.success() {
                eprintln!("Fuzzing {} failed with status: {}", target, status);
                // Continue to next target rather than aborting
                continue;
            }

            // Run corpus minimization
            println!("\nMinimizing corpus...");
            let status = Command::new("cargo")
                .args(["+nightly", "fuzz", "cmin", target])
                .current_dir(&fuzz_dir)
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status()
                .context("failed to run cargo fuzz cmin")?;

            if !status.success() {
                eprintln!("Corpus minimization for {} failed with status: {}", target, status);
            }
        }
    }

    println!("\n=== Fuzz smoke test complete ===");
    Ok(())
}
