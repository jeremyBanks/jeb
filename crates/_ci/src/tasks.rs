use anyhow::{Context, Result};
use std::process::{Command, ExitStatus};

pub fn build() -> Result<()> {
    println!("==> Running build checks");

    run_command("cargo", &["build"])
        .context("cargo build failed")?;

    run_command("cargo", &["publish", "--dry-run"])
        .context("cargo publish --dry-run failed")?;

    run_command("deno", &["test"])
        .context("deno test failed")?;

    println!("✓ Build checks passed");
    Ok(())
}

pub fn build_warnings() -> Result<()> {
    println!("==> Running build with warnings-as-errors");

    // Set RUSTFLAGS to treat warnings as errors
    let status = Command::new("cargo")
        .args(&["build"])
        .env("RUSTFLAGS", "-D warnings")
        .status()
        .context("Failed to run cargo build")?;

    if !status.success() {
        anyhow::bail!("cargo build with -D warnings failed");
    }

    run_command("cargo", &["publish", "--dry-run"])
        .context("cargo publish --dry-run failed")?;

    println!("✓ Build warnings check passed");
    Ok(())
}

pub fn test() -> Result<()> {
    println!("==> Running tests");

    run_command("deno", &["test"])
        .context("deno test failed")?;

    println!("✓ Tests passed");
    Ok(())
}

/// Run a command and ensure it succeeds
fn run_command(program: &str, args: &[&str]) -> Result<ExitStatus> {
    println!("  $ {} {}", program, args.join(" "));

    let status = Command::new(program)
        .args(args)
        .status()
        .with_context(|| format!("Failed to run {}", program))?;

    if !status.success() {
        anyhow::bail!("{} failed with {}", program, status);
    }

    Ok(status)
}
