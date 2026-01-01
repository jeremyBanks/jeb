use anyhow::{Context, Result};
use glob::glob;
use semver::Version;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use toml_edit::{value, DocumentMut, InlineTable, Item, Value};

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

    // Group into equivalence classes and vote
    let mut workspace_updates: HashMap<String, (ResolutionFields, String)> = HashMap::new();

    eprintln!("DEBUG: Found {} unique dependency names", all_deps.len());

    for (dep_name, occurrences) in &all_deps {
        eprintln!("DEBUG: Processing dependency '{}' with {} occurrences", dep_name, occurrences.len());

        // Group by equivalence class
        let mut equivalence_classes: Vec<EquivalenceClass> = Vec::new();

        for (member_path, _section, dep) in occurrences {
            // Find or create equivalence class
            let mut found = false;
            for ec in &mut equivalence_classes {
                if ec.matches(dep) {
                    ec.add_vote(member_path.clone(), dep.clone());
                    found = true;
                    break;
                }
            }

            if !found {
                let mut ec = EquivalenceClass::new(dep.resolution.clone());
                ec.add_vote(member_path.clone(), dep.clone());
                equivalence_classes.push(ec);
            }
        }

        eprintln!("DEBUG: Found {} equivalence classes", equivalence_classes.len());

        // Find the winning equivalence class
        if let Some(winner) = find_winner(&equivalence_classes) {
            // Determine the key to use in workspace.dependencies
            let key = winner.get_preferred_key();
            eprintln!("DEBUG: Winner for '{}': key='{}', votes={}", dep_name, key, winner.vote_count());
            workspace_updates.insert(key, (winner.resolution.clone(), dep_name.clone()));
        } else {
            eprintln!("DEBUG: No winner found for '{}'", dep_name);
        }
    }

    eprintln!("DEBUG: workspace_updates has {} entries", workspace_updates.len());

    // Update workspace Cargo.toml
    update_workspace_toml(&mut workspace_doc, &workspace_updates, workspace_root)?;
    std::fs::write(&workspace_toml_path, workspace_doc.to_string())?;

    // Update member Cargo.toml files
    for member_path in &members {
        update_member_toml(member_path, &all_deps, &workspace_updates)?;
    }

    Ok(())
}

/// Represents an equivalence class of dependencies
#[derive(Debug, Clone)]
struct EquivalenceClass {
    resolution: ResolutionFields,
    votes: HashMap<PathBuf, Vec<Dependency>>,
}

impl EquivalenceClass {
    fn new(resolution: ResolutionFields) -> Self {
        Self {
            resolution,
            votes: HashMap::new(),
        }
    }

    fn matches(&self, dep: &Dependency) -> bool {
        self.resolution.matches_except_version(&dep.resolution) &&
            versions_compatible_opt(&self.resolution.version, &dep.resolution.version)
    }

    fn add_vote(&mut self, member: PathBuf, dep: Dependency) {
        // Update to max version if this dep has a higher version
        if let (Some(current), Some(new)) = (&self.resolution.version, &dep.resolution.version) {
            if new > current {
                self.resolution.version = Some(new.clone());
            }
        }

        self.votes.entry(member).or_default().push(dep);
    }

    fn vote_count(&self) -> usize {
        self.votes.len()
    }

    fn get_preferred_key(&self) -> String {
        // Return the first key alphabetically from all dependencies
        let mut keys: Vec<String> = self.votes.values()
            .flatten()
            .map(|d| d.key.clone())
            .collect();
        keys.sort();
        keys.into_iter().next().unwrap_or_else(|| "unknown".to_string())
    }
}

fn versions_compatible_opt(v1: &Option<Version>, v2: &Option<Version>) -> bool {
    match (v1, v2) {
        (Some(a), Some(b)) => versions_compatible(a, b),
        (None, None) => true,
        _ => false,
    }
}

fn find_winner(classes: &[EquivalenceClass]) -> Option<&EquivalenceClass> {
    if classes.is_empty() {
        return None;
    }

    let max_votes = classes.iter().map(|c| c.vote_count()).max().unwrap();
    let mut candidates: Vec<&EquivalenceClass> = classes.iter()
        .filter(|c| c.vote_count() == max_votes)
        .collect();

    if candidates.len() == 1 {
        return Some(candidates[0]);
    }

    // Tie-breaker: compare by version, then by field ordering
    candidates.sort_by(|a, b| {
        // First compare by version (descending)
        match (&a.resolution.version, &b.resolution.version) {
            (Some(v1), Some(v2)) => {
                let cmp = v2.cmp(v1); // Note: reversed for descending
                if cmp != std::cmp::Ordering::Equal {
                    return cmp;
                }
            }
            (Some(_), None) => return std::cmp::Ordering::Less,
            (None, Some(_)) => return std::cmp::Ordering::Greater,
            (None, None) => {}
        }

        // Then compare by sorted field pairs
        a.resolution.as_sorted_pairs().cmp(&b.resolution.as_sorted_pairs())
    });

    candidates.first().copied()
}

fn update_workspace_toml(
    doc: &mut DocumentMut,
    updates: &HashMap<String, (ResolutionFields, String)>,
    workspace_root: &Path,
) -> Result<()> {
    // Ensure workspace.dependencies exists
    if doc.get("workspace").is_none() {
        doc["workspace"] = toml_edit::table();
    }

    let workspace = doc["workspace"].as_table_mut().context("workspace is not a table")?;

    if workspace.get("dependencies").is_none() {
        workspace["dependencies"] = toml_edit::table();
    }

    let deps = workspace["dependencies"].as_table_mut().context("dependencies is not a table")?;

    // Track which dependencies are still used
    let mut used_deps: HashSet<String> = HashSet::new();

    for (key, (resolution, _dep_name)) in updates {
        used_deps.insert(key.clone());

        // Build the value for this dependency
        let value = build_dependency_value(resolution, workspace_root, false)?;

        // Insert or update the dependency
        if deps.contains_key(key.as_str()) {
            deps[key.as_str()] = value;
        } else {
            // Find insertion position (scan up from bottom)
            let mut insert_pos = None;
            let keys: Vec<String> = deps.iter().map(|(k, _)| k.to_string()).collect();

            for (i, existing_key) in keys.iter().enumerate().rev() {
                if existing_key < key {
                    insert_pos = Some(i + 1);
                    break;
                }
            }

            // Insert at the determined position
            if let Some(pos) = insert_pos {
                // toml_edit doesn't have easy positional insert, so we'll just append
                deps.insert(key.as_str(), value);
            } else {
                deps.insert(key.as_str(), value);
            }
        }
    }

    // Remove unused dependencies
    let all_keys: Vec<String> = deps.iter().map(|(k, _)| k.to_string()).collect();
    for key in all_keys {
        if !used_deps.contains(&key) {
            deps.remove(&key);
        }
    }

    Ok(())
}

fn build_dependency_value(
    resolution: &ResolutionFields,
    workspace_root: &Path,
    include_config: bool,
) -> Result<Item> {
    let mut has_extra_fields = false;

    // Check if we have fields other than version
    if resolution.package.is_some()
        || resolution.path.is_some()
        || resolution.git.is_some()
        || resolution.branch.is_some()
        || resolution.tag.is_some()
        || resolution.rev.is_some()
        || resolution.registry.is_some()
    {
        has_extra_fields = true;
    }

    if !has_extra_fields && resolution.version.is_some() {
        // Simple string form
        let version_str = resolution.version.as_ref().unwrap().to_string();
        return Ok(value(version_str).into());
    }

    // Inline table form
    let mut table = InlineTable::new();

    // Add fields in order
    if let Some(ref package) = resolution.package {
        table.insert("package", Value::from(package.as_str()));
    }

    if let Some(ref version) = resolution.version {
        table.insert("version", Value::from(version.to_string()));
    }

    if let Some(ref path) = resolution.path {
        // Convert to relative path from workspace root
        let rel_path = if let Ok(stripped) = path.strip_prefix(workspace_root) {
            stripped
        } else {
            // If not under workspace, try to make relative
            &pathdiff::diff_paths(path, workspace_root)
                .ok_or_else(|| anyhow::anyhow!("Could not create relative path"))?
        };
        table.insert("path", Value::from(rel_path.display().to_string()));
    }

    if let Some(ref git) = resolution.git {
        table.insert("git", Value::from(git.as_str()));
    }

    if let Some(ref branch) = resolution.branch {
        table.insert("branch", Value::from(branch.as_str()));
    }

    if let Some(ref tag) = resolution.tag {
        table.insert("tag", Value::from(tag.as_str()));
    }

    if let Some(ref rev) = resolution.rev {
        table.insert("rev", Value::from(rev.as_str()));
    }

    if let Some(ref registry) = resolution.registry {
        table.insert("registry", Value::from(registry.as_str()));
    }

    Ok(Item::Value(Value::InlineTable(table)))
}

fn update_member_toml(
    member_path: &Path,
    all_deps: &HashMap<String, Vec<(PathBuf, String, Dependency)>>,
    workspace_updates: &HashMap<String, (ResolutionFields, String)>,
) -> Result<()> {
    let member_toml = member_path.join("Cargo.toml");
    let content = std::fs::read_to_string(&member_toml)?;
    let mut doc = content.parse::<DocumentMut>()?;

    // Build a map of dep_name -> workspace_key
    let mut dep_name_to_workspace_key: HashMap<String, String> = HashMap::new();
    for (workspace_key, (_resolution, dep_name)) in workspace_updates {
        dep_name_to_workspace_key.insert(dep_name.clone(), workspace_key.clone());
    }

    for section in &["dependencies", "dev-dependencies", "build-dependencies"] {
        if let Some(deps) = doc.get_mut(section).and_then(|s| s.as_table_mut()) {
            let keys: Vec<String> = deps.iter().map(|(k, _)| k.to_string()).collect();

            for key in keys {
                if let Some(dep_item) = deps.get(&key) {
                    if let Ok(Some(dep)) = parse_dependency(&key, dep_item, member_path) {
                        // Check if this dependency is in the winning equivalence class
                        if let Some(workspace_key) = dep_name_to_workspace_key.get(&dep.name) {
                            // Check if this specific occurrence should use workspace = true
                            if should_use_workspace(&dep, workspace_updates, workspace_key) {
                                // Update to use workspace = true
                                let mut table = InlineTable::new();
                                table.insert("workspace", Value::from(true));

                                // Keep configuration fields
                                if let Some(optional) = dep.config.optional {
                                    table.insert("optional", Value::from(optional));
                                }

                                if let Some(ref features) = dep.config.features {
                                    let arr: toml_edit::Array = features.iter()
                                        .map(|s| Value::from(s.as_str()))
                                        .collect();
                                    table.insert("features", Value::Array(arr));
                                }

                                if let Some(default_features) = dep.config.default_features {
                                    table.insert("default-features", Value::from(default_features));
                                }

                                deps[&key] = Item::Value(Value::InlineTable(table));
                            }
                        }
                    }
                }
            }
        }
    }

    std::fs::write(&member_toml, doc.to_string())?;
    Ok(())
}

fn should_use_workspace(
    dep: &Dependency,
    workspace_updates: &HashMap<String, (ResolutionFields, String)>,
    workspace_key: &str,
) -> bool {
    if let Some((workspace_resolution, _)) = workspace_updates.get(workspace_key) {
        // Check if this dependency matches the workspace resolution
        workspace_resolution.matches_except_version(&dep.resolution) &&
            versions_compatible_opt(&workspace_resolution.version, &dep.resolution.version)
    } else {
        false
    }
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
        println!("Workspace Cargo.toml:\n{}", workspace_content);

        // serde should be promoted (used by both crates)
        assert!(workspace_content.contains("serde"), "serde not found in workspace");

        // Check that member Cargo.tomls now use workspace = true
        let crate_a_content = fs::read_to_string(temp.path().join("crate-a/Cargo.toml"))?;
        println!("Crate-a Cargo.toml:\n{}", crate_a_content);
        assert!(crate_a_content.contains("workspace = true"), "workspace = true not found in crate-a");

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
