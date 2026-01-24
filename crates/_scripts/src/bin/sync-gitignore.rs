use {
    anyhow::{
        Context,
        Result,
    },
    std::{
        collections::HashSet,
        fs,
        path::{
            Path,
            PathBuf,
        },
    },
};

/// Read lines from a file, filtering out empty lines and comments
fn read_gitignore_lines(path: &Path) -> Result<Vec<String>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;

    Ok(content
        .lines()
        .map(|line| line.to_string())
        .collect())
}

/// Normalize a gitignore line by removing leading slashes if present
/// Lines starting with / are root-relative, but at the crate level they should be
/// relative to the crate root
fn normalize_line(line: &str) -> String {
    if line.starts_with('/') {
        line[1..].to_string()
    } else {
        line.to_string()
    }
}

/// Get all crate directories under crates/
fn get_crate_dirs() -> Result<Vec<PathBuf>> {
    let crates_dir = Path::new("crates");
    if !crates_dir.exists() {
        anyhow::bail!("crates/ directory not found");
    }

    let mut crate_dirs = Vec::new();

    for entry in fs::read_dir(crates_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // Check if it has a Cargo.toml to confirm it's a crate
            if path.join("Cargo.toml").exists() {
                crate_dirs.push(path);
            }
        }
    }

    crate_dirs.sort();
    Ok(crate_dirs)
}

/// Check if a line matches any existing line (accounting for normalization)
fn line_exists(normalized_line: &str, existing_lines: &[String]) -> bool {
    for existing in existing_lines {
        let normalized_existing = normalize_line(existing.trim());
        if normalized_existing == normalized_line {
            return true;
        }
    }
    false
}

fn sync_gitignore_to_crate(root_gitignore: &Path, crate_dir: &Path) -> Result<bool> {
    let root_lines = read_gitignore_lines(root_gitignore)?;
    let crate_gitignore = crate_dir.join(".gitignore");
    let mut existing_lines = read_gitignore_lines(&crate_gitignore)?;

    // Track lines to add
    let mut lines_to_add = Vec::new();

    for line in &root_lines {
        let trimmed = line.trim();

        // Skip empty lines and comments for checking, but we'll preserve them in existing
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Normalize the line (remove leading /)
        let normalized = normalize_line(trimmed);

        // Check if this normalized line already exists
        if !line_exists(&normalized, &existing_lines) {
            lines_to_add.push(normalized);
        }
    }

    if lines_to_add.is_empty() {
        return Ok(false); // No changes needed
    }

    // Add new lines to the end
    for line in &lines_to_add {
        existing_lines.push(line.clone());
    }

    // Write back, ensuring a trailing newline
    let mut content = existing_lines.join("\n");
    if !content.is_empty() && !content.ends_with('\n') {
        content.push('\n');
    }

    fs::write(&crate_gitignore, content)
        .with_context(|| format!("Failed to write {}", crate_gitignore.display()))?;

    Ok(true) // Changes were made
}

fn main() -> Result<()> {
    let root_gitignore = Path::new(".gitignore");
    if !root_gitignore.exists() {
        anyhow::bail!("Root .gitignore not found");
    }

    let crate_dirs = get_crate_dirs()?;
    println!("Found {} crate directories", crate_dirs.len());

    let mut updated_count = 0;

    for crate_dir in &crate_dirs {
        let crate_name = crate_dir.file_name().unwrap().to_string_lossy();

        match sync_gitignore_to_crate(root_gitignore, crate_dir) {
            Ok(true) => {
                println!("  ✓ Updated {}", crate_name);
                updated_count += 1;
            }
            Ok(false) => {
                // No changes needed - can be silent or verbose based on preference
            }
            Err(e) => {
                eprintln!("  ✗ Failed to update {}: {}", crate_name, e);
            }
        }
    }

    println!("\nUpdated {} crate(s)", updated_count);
    Ok(())
}
