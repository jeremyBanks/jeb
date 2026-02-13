use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::process::{Command, ExitStatus};

#[derive(Parser)]
#[clap(name = "_ci", about = "Type-safe CI tasks")]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run build checks (cargo build + publish dry-run + deno test)
    Build,
    /// Run build with warnings-as-errors
    BuildWarnings,
    /// Run all tests
    Test,
    /// Run all CI checks (build + build-warnings + test)
    All,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build => build()?,
        Commands::BuildWarnings => build_warnings()?,
        Commands::Test => test()?,
        Commands::All => {
            build()?;
            build_warnings()?;
            test()?;
        }
    }

    Ok(())
}

fn build() -> Result<()> {
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

fn build_warnings() -> Result<()> {
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

fn test() -> Result<()> {
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
