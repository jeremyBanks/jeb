//! Bump version based on current date and time.
//!
//! Format: `MAJOR.MINOR.PATCH-dev-YYYY-MM-DD.xxxx`
//! where `xxxx = (yyyy XOR (dd * 100 + MM)) + (hh * 100 + mm) + 2048`

use {
    anyhow::{Context, Result, bail},
    chrono::Local,
    std::{fs, path::Path},
    toml_edit::DocumentMut,
};

fn main() -> Result<()> {
    let cargo_toml_path = Path::new("Cargo.toml");

    // Read and parse Cargo.toml
    let content = fs::read_to_string(cargo_toml_path)
        .context("Failed to read Cargo.toml")?;
    let mut doc: DocumentMut = content.parse()
        .context("Failed to parse Cargo.toml")?;

    // Get current version from [workspace.package]
    let current_version = doc
        .get("workspace")
        .and_then(|ws| ws.get("package"))
        .and_then(|pkg| pkg.get("version"))
        .and_then(|v| v.as_str())
        .context("No version found in [workspace.package]")?;

    // Parse major.minor.patch from current version
    let (major, minor, patch) = parse_version(current_version)?;

    // Get current date and time
    let now = Local::now();
    let yyyy = now.format("%Y").to_string().parse::<i32>().unwrap();
    let mm = now.format("%m").to_string().parse::<i32>().unwrap();
    let dd = now.format("%d").to_string().parse::<i32>().unwrap();
    let hh = now.format("%H").to_string().parse::<i32>().unwrap();
    let min = now.format("%M").to_string().parse::<i32>().unwrap();

    // Calculate xxxx = (yyyy XOR (dd * 100 + MM)) + (hh * 100 + mm) + 2048
    let first_part = yyyy ^ (dd * 100 + mm);
    let second_part = hh * 100 + min;
    let xxxx = first_part + second_part + 2048;

    // Format the new version
    let date_str = now.format("%Y-%m-%d");
    let new_version = format!("{major}.{minor}.{patch}-dev-{date_str}.{xxxx}");

    println!("Calculated version: {new_version}");
    println!("  Date: {date_str}");
    println!("  Time: {hh:02}:{min:02}");
    println!("  Formula: ({yyyy} XOR ({dd} * 100 + {mm})) + ({hh} * 100 + {min}) + 2048");
    println!("  = ({yyyy} XOR {}) + {} + 2048", dd * 100 + mm, hh * 100 + min);
    println!("  = {first_part} + {second_part} + 2048");
    println!("  = {xxxx}");

    // Update the version in the document
    doc["workspace"]["package"]["version"] = toml_edit::value(&new_version);

    // Write back to Cargo.toml
    fs::write(cargo_toml_path, doc.to_string())
        .context("Failed to write Cargo.toml")?;

    println!("Updated workspace version in Cargo.toml to: {new_version}");

    Ok(())
}

/// Parse major.minor.patch from a version string.
/// Handles both `X.Y.Z-something` and `X.Y.Z` formats.
fn parse_version(version: &str) -> Result<(u32, u32, u32)> {
    // Split off any suffix after the patch number
    let base = version.split('-').next().unwrap_or(version);
    let parts: Vec<&str> = base.split('.').collect();

    if parts.len() < 3 {
        bail!("Could not parse version from: {version}");
    }

    let major: u32 = parts[0].parse()
        .with_context(|| format!("Invalid major version: {}", parts[0]))?;
    let minor: u32 = parts[1].parse()
        .with_context(|| format!("Invalid minor version: {}", parts[1]))?;
    let patch: u32 = parts[2].parse()
        .with_context(|| format!("Invalid patch version: {}", parts[2]))?;

    Ok((major, minor, patch))
}
