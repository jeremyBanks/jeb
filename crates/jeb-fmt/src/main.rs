use anyhow::{Context, Result};
use glob::glob;
use semver::Version;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use toml_edit::{DocumentMut, Item, Value};

fn main() -> Result<()> {
    let workspace_root = find_workspace_root(".")?;
    println!("Found workspace root: {}", workspace_root.display());

    normalize_workspace_dependencies(&workspace_root)?;

    println!("Workspace dependencies normalized successfully!");
    Ok(())
}

/// Find the workspace root by looking for a Cargo.toml with [workspace]
fn find_workspace_root(start_dir: &str) -> Result<PathBuf> {
    let mut current = PathBuf::from(start_dir).canonicalize()?;

    loop {
        let cargo_toml = current.join("Cargo.toml");
        if cargo_toml.exists() {
            let content = std::fs::read_to_string(&cargo_toml)?;
            let doc = content.parse::<DocumentMut>()?;

            if doc.get("workspace").is_some() {
                return Ok(current);
            }
        }

        if !current.pop() {
            anyhow::bail!("Could not find workspace root");
        }
    }
}

/// Resolve workspace members using glob patterns
fn resolve_workspace_members(workspace_root: &Path) -> Result<Vec<PathBuf>> {
    let cargo_toml_path = workspace_root.join("Cargo.toml");
    let content = std::fs::read_to_string(&cargo_toml_path)?;
    let doc = content.parse::<DocumentMut>()?;

    let members = doc
        .get("workspace")
        .and_then(|w| w.get("members"))
        .and_then(|m| m.as_array())
        .context("workspace.members not found or not an array")?;

    let mut member_paths = Vec::new();

    for member in members.iter() {
        let pattern = member
            .as_str()
            .context("member pattern is not a string")?;

        let full_pattern = workspace_root.join(pattern);
        let pattern_str = full_pattern.to_str().context("invalid path")?;

        let mut found_any = false;
        for entry in glob(pattern_str)? {
            let path = entry?;
            let cargo_toml = path.join("Cargo.toml");
            if cargo_toml.exists() {
                member_paths.push(path);
                found_any = true;
            }
        }

        if !found_any {
            anyhow::bail!("No members found matching pattern: {}", pattern);
        }
    }

    Ok(member_paths)
}

/// Represents a dependency with all its fields
#[derive(Debug, Clone, PartialEq, Eq)]
struct Dependency {
    /// The key used in the TOML table
    key: String,
    /// The actual dependency name (from package field or key)
    name: String,
    /// Resolution fields
    resolution: ResolutionFields,
    /// Configuration fields (not promoted to workspace)
    config: ConfigFields,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ResolutionFields {
    package: Option<String>,
    version: Option<Version>,
    path: Option<PathBuf>,
    git: Option<String>,
    branch: Option<String>,
    tag: Option<String>,
    rev: Option<String>,
    registry: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ConfigFields {
    optional: Option<bool>,
    features: Option<Vec<String>>,
    default_features: Option<bool>,
}

impl ResolutionFields {
    /// Check if two resolution fields are equal except for version compatibility
    fn matches_except_version(&self, other: &Self) -> bool {
        self.package == other.package
            && self.path == other.path
            && self.git == other.git
            && self.branch == other.branch
            && self.tag == other.tag
            && self.rev == other.rev
            && self.registry == other.registry
    }

    /// Get a sorted vector of (key, value) for tie-breaking
    fn as_sorted_pairs(&self) -> Vec<(&str, String)> {
        let mut pairs = Vec::new();

        if let Some(ref v) = self.package {
            pairs.push(("package", v.clone()));
        }
        if let Some(ref v) = self.path {
            pairs.push(("path", v.display().to_string()));
        }
        if let Some(ref v) = self.git {
            pairs.push(("git", v.clone()));
        }
        if let Some(ref v) = self.branch {
            pairs.push(("branch", v.clone()));
        }
        if let Some(ref v) = self.tag {
            pairs.push(("tag", v.clone()));
        }
        if let Some(ref v) = self.rev {
            pairs.push(("rev", v.clone()));
        }
        if let Some(ref v) = self.registry {
            pairs.push(("registry", v.clone()));
        }

        pairs.sort_by_key(|(k, _)| *k);
        pairs
    }
}

/// Check if two versions are compatible (same leftmost non-zero component)
fn versions_compatible(v1: &Version, v2: &Version) -> bool {
    // Pre-release versions must match exactly
    if !v1.pre.is_empty() || !v2.pre.is_empty() {
        return v1 == v2;
    }

    // Check leftmost non-zero component
    if v1.major != 0 {
        v1.major == v2.major
    } else if v1.minor != 0 {
        v1.major == v2.major && v1.minor == v2.minor
    } else {
        v1.major == v2.major && v1.minor == v2.minor && v1.patch == v2.patch
    }
}

/// Parse a dependency from a TOML value
fn parse_dependency(
    key: &str,
    value: &Item,
    base_path: &Path,
) -> Result<Option<Dependency>> {
    // Extract version string and other fields
    let version_str: Option<&str>;
    let mut package: Option<String> = None;
    let mut path_str: Option<String> = None;
    let mut git: Option<String> = None;
    let mut branch: Option<String> = None;
    let mut tag: Option<String> = None;
    let mut rev: Option<String> = None;
    let mut registry: Option<String> = None;
    let mut optional: Option<bool> = None;
    let mut features: Option<Vec<String>> = None;
    let mut default_features: Option<bool> = None;

    match value {
        Item::Value(Value::String(s)) => {
            // Simple string form: dep = "1.0.0"
            version_str = Some(s.value());
        }
        Item::Value(Value::InlineTable(t)) => {
            // Inline table form
            version_str = t.get("version").and_then(|v| v.as_str());
            package = t.get("package").and_then(|v| v.as_str()).map(String::from);
            path_str = t.get("path").and_then(|v| v.as_str()).map(String::from);
            git = t.get("git").and_then(|v| v.as_str()).map(String::from);
            branch = t.get("branch").and_then(|v| v.as_str()).map(String::from);
            tag = t.get("tag").and_then(|v| v.as_str()).map(String::from);
            rev = t.get("rev").and_then(|v| v.as_str()).map(String::from);
            registry = t.get("registry").and_then(|v| v.as_str()).map(String::from);
            optional = t.get("optional").and_then(|v| v.as_bool());
            default_features = t.get("default-features").and_then(|v| v.as_bool());
            features = t.get("features").and_then(|v| {
                v.as_array().map(|arr| {
                    arr.iter()
                        .filter_map(|item| item.as_str().map(String::from))
                        .collect()
                })
            });
        }
        Item::Table(t) => {
            // Section form [dependencies.foo]
            version_str = t.get("version").and_then(|v| v.as_str());
            package = t.get("package").and_then(|v| v.as_str()).map(String::from);
            path_str = t.get("path").and_then(|v| v.as_str()).map(String::from);
            git = t.get("git").and_then(|v| v.as_str()).map(String::from);
            branch = t.get("branch").and_then(|v| v.as_str()).map(String::from);
            tag = t.get("tag").and_then(|v| v.as_str()).map(String::from);
            rev = t.get("rev").and_then(|v| v.as_str()).map(String::from);
            registry = t.get("registry").and_then(|v| v.as_str()).map(String::from);
            optional = t.get("optional").and_then(|v| v.as_bool());
            default_features = t.get("default-features").and_then(|v| v.as_bool());
            features = t.get("features").and_then(|v| {
                v.as_array().map(|arr| {
                    arr.iter()
                        .filter_map(|item| item.as_str().map(String::from))
                        .collect()
                })
            });
        }
        _ => return Ok(None),
    }

    // Parse version if present
    let version = if let Some(v_str) = version_str {
        // Remove optional ^ prefix
        let trimmed = v_str.trim_start_matches('^');

        // Only accept bare version or ^ prefix
        if !v_str.starts_with('^') && v_str != trimmed {
            // Has some other prefix, skip
            return Ok(None);
        }

        match Version::parse(trimmed) {
            Ok(v) => Some(v),
            Err(_) => {
                // Skip dependencies with invalid versions
                return Ok(None);
            }
        }
    } else {
        None
    };

    // Normalize path relative to base_path
    let path = if let Some(p) = path_str {
        Some(base_path.join(p).canonicalize()?)
    } else {
        None
    };

    let name = package.clone().unwrap_or_else(|| key.to_string());

    Ok(Some(Dependency {
        key: key.to_string(),
        name,
        resolution: ResolutionFields {
            package,
            version,
            path,
            git,
            branch,
            tag,
            rev,
            registry,
        },
        config: ConfigFields {
            optional,
            features,
            default_features,
        },
    }))
}

/// Main normalization function
fn normalize_workspace_dependencies(workspace_root: &Path) -> Result<()> {
    // Load workspace Cargo.toml
    let workspace_toml_path = workspace_root.join("Cargo.toml");
    let workspace_content = std::fs::read_to_string(&workspace_toml_path)?;
    let mut workspace_doc = workspace_content.parse::<DocumentMut>()?;

    // Check for configuration fields in workspace.dependencies
    let workspace_deps = workspace_doc
        .get("workspace")
        .and_then(|w| w.get("dependencies"))
        .and_then(|d| d.as_table());

    let mut blocked_deps = HashSet::new();

    if let Some(deps_table) = workspace_deps {
        for (key, value) in deps_table.iter() {
            if has_config_fields(value) {
                eprintln!(
                    "Warning: workspace dependency '{}' has configuration fields, skipping",
                    key
                );
                blocked_deps.insert(key.to_string());
            }
        }
    }

    // Resolve members
    let members = resolve_workspace_members(workspace_root)?;

    // Parse all dependencies from all members
    let mut all_deps: HashMap<String, Vec<(PathBuf, String, Dependency)>> = HashMap::new();

    for member_path in &members {
        let member_toml = member_path.join("Cargo.toml");
        let content = std::fs::read_to_string(&member_toml)?;
        let doc = content.parse::<DocumentMut>()?;

        for section in &["dependencies", "dev-dependencies", "build-dependencies"] {
            if let Some(deps) = doc.get(section).and_then(|s| s.as_table()) {
                for (key, value) in deps.iter() {
                    if blocked_deps.contains(key) {
                        continue;
                    }

                    if let Some(dep) = parse_dependency(key, value, member_path)? {
                        all_deps
                            .entry(dep.name.clone())
                            .or_default()
                            .push((member_path.clone(), section.to_string(), dep));
                    }
                }
            }
        }
    }

    // TODO: Implement equivalence class grouping and voting
    // TODO: Update workspace Cargo.toml
    // TODO: Update member Cargo.toml files

    Ok(())
}

fn has_config_fields(value: &Item) -> bool {
    if let Some(table) = value.as_inline_table() {
        return table.contains_key("optional")
            || table.contains_key("features")
            || table.contains_key("default-features");
    }
    if let Some(table) = value.as_table() {
        return table.contains_key("optional")
            || table.contains_key("features")
            || table.contains_key("default-features");
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_workspace() -> Result<TempDir> {
        let dir = TempDir::new()?;
        let workspace_root = dir.path();

        // Create workspace Cargo.toml
        fs::write(
            workspace_root.join("Cargo.toml"),
            r#"[workspace]
members = ["crate-a", "crate-b"]

[workspace.dependencies]
"#,
        )?;

        // Create crate-a
        fs::create_dir(workspace_root.join("crate-a"))?;
        fs::write(
            workspace_root.join("crate-a/Cargo.toml"),
            r#"[package]
name = "crate-a"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "1.0"
tokio = "1.0"
"#,
        )?;

        // Create crate-b
        fs::create_dir(workspace_root.join("crate-b"))?;
        fs::write(
            workspace_root.join("crate-b/Cargo.toml"),
            r#"[package]
name = "crate-b"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "1.0"
anyhow = "1.0"
"#,
        )?;

        Ok(dir)
    }

    #[test]
    fn test_find_workspace_root() -> Result<()> {
        let temp = create_test_workspace()?;
        let workspace_root = find_workspace_root(temp.path().to_str().unwrap())?;
        assert_eq!(workspace_root, temp.path().canonicalize()?);
        Ok(())
    }

    #[test]
    fn test_resolve_workspace_members() -> Result<()> {
        let temp = create_test_workspace()?;
        let members = resolve_workspace_members(temp.path())?;
        assert_eq!(members.len(), 2);
        Ok(())
    }

    #[test]
    fn test_versions_compatible() {
        let v1 = Version::parse("1.2.3").unwrap();
        let v2 = Version::parse("1.5.0").unwrap();
        let v3 = Version::parse("2.0.0").unwrap();

        assert!(versions_compatible(&v1, &v2));
        assert!(!versions_compatible(&v1, &v3));

        // Test 0.x versions
        let v4 = Version::parse("0.2.3").unwrap();
        let v5 = Version::parse("0.2.7").unwrap();
        let v6 = Version::parse("0.3.0").unwrap();

        assert!(versions_compatible(&v4, &v5));
        assert!(!versions_compatible(&v4, &v6));

        // Test 0.0.x versions
        let v7 = Version::parse("0.0.3").unwrap();
        let v8 = Version::parse("0.0.3").unwrap();
        let v9 = Version::parse("0.0.4").unwrap();

        assert!(versions_compatible(&v7, &v8));
        assert!(!versions_compatible(&v7, &v9));
    }

    #[test]
    fn test_normalize_shared_dependencies() -> Result<()> {
        let temp = create_test_workspace()?;
        normalize_workspace_dependencies(temp.path())?;

        // Read the updated workspace Cargo.toml
        let workspace_content = fs::read_to_string(temp.path().join("Cargo.toml"))?;

        // serde should be promoted (used by both crates)
        assert!(workspace_content.contains("serde"));

        // Check that member Cargo.tomls now use workspace = true
        let crate_a_content = fs::read_to_string(temp.path().join("crate-a/Cargo.toml"))?;
        assert!(crate_a_content.contains("workspace = true"));

        Ok(())
    }

    #[test]
    fn test_prerelease_versions() {
        let v1 = Version::parse("1.0.0-alpha").unwrap();
        let v2 = Version::parse("1.0.0-alpha").unwrap();
        let v3 = Version::parse("1.0.0-beta").unwrap();

        assert!(versions_compatible(&v1, &v2));
        assert!(!versions_compatible(&v1, &v3));
    }
}
