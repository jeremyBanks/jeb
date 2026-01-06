use {
    anyhow::{
        Context,
        Result,
    },
    glob::glob,
    semver::Version,
    std::{
        collections::{
            HashMap,
            HashSet,
        },
        path::{
            Path,
            PathBuf,
        },
    },
    toml_edit::{
        DocumentMut,
        InlineTable,
        Item,
        Key,
        Value,
        value,
    },
};
pub fn main() -> i32 {
    eprintln!("Running: workspace dependency normalization");
    match run_normalization() {
        Ok(()) => {
            eprintln!();
            0
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            eprintln!();
            1
        }
    }
}
fn run_normalization() -> Result<()> {
    let workspace_root = find_workspace_root(".")?;

    // Ensure all crates are in workspace members
    let added_members = ensure_workspace_members(&workspace_root)?;
    if added_members > 0 {
        eprintln!("  Added {} missing workspace member(s)", added_members);
    }

    // Ensure internal crates have publish = false
    let updated_publish = ensure_publish_false_for_internal_crates(&workspace_root)?;
    if updated_publish > 0 {
        eprintln!(
            "  Updated publish = false for {} internal crate(s)",
            updated_publish
        );
    }

    let mut stats = NormalizationStats::default();
    normalize_workspace_dependencies(&workspace_root, &mut stats)?;
    eprintln!("\nSummary:");
    eprintln!("  Workspaces examined: {}", stats.workspaces_processed);
    eprintln!("  Crates examined: {}", stats.crates_examined);
    eprintln!("  Workspace crates found: {}", stats.workspace_crates_found);
    eprintln!(
        "  Patch entries: {} added, {} updated, {} removed",
        stats.patch_entries_added, stats.patch_entries_updated, stats.patch_entries_removed
    );
    eprintln!(
        "  Workspace dep versions synced: {}",
        stats.workspace_dep_versions_synced
    );
    eprintln!("  Lockfiles copied: {}", stats.lockfiles_copied);
    eprintln!(
        "  Workspace Cargo.toml files edited: {}",
        stats.workspace_tomls_edited
    );
    eprintln!(
        "  Member Cargo.toml files edited: {}",
        stats.member_tomls_edited
    );
    eprintln!("  Total files edited: {}", stats.edited_files.len());
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
        let pattern = member.as_str().context("member pattern is not a string")?;
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
/// Ensure all crates in the crates/ directory are referenced in
/// workspace.members
fn ensure_workspace_members(workspace_root: &Path) -> Result<usize> {
    let cargo_toml_path = workspace_root.join("Cargo.toml");
    let content = std::fs::read_to_string(&cargo_toml_path)?;
    let mut doc = content.parse::<DocumentMut>()?;

    // Get existing members patterns
    let existing_members = resolve_workspace_members(workspace_root)?;
    let existing_members_set: HashSet<PathBuf> = existing_members.into_iter().collect();

    // Scan crates/ directory for all Cargo.toml files
    let crates_dir = workspace_root.join("crates");
    if !crates_dir.exists() {
        return Ok(0);
    }

    let mut missing_crates = Vec::new();
    for entry in std::fs::read_dir(&crates_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let cargo_toml = path.join("Cargo.toml");
            if cargo_toml.exists() {
                let canonical_path = path.canonicalize().unwrap_or_else(|_| path.clone());
                if !existing_members_set.contains(&canonical_path) {
                    missing_crates.push(path.clone());
                }
            }
        }
    }

    if missing_crates.is_empty() {
        return Ok(0);
    }

    // Add missing crates to workspace.members
    if doc.get("workspace").is_none() {
        doc["workspace"] = toml_edit::table();
    }
    let workspace = doc["workspace"]
        .as_table_mut()
        .context("workspace is not a table")?;

    if workspace.get("members").is_none() {
        workspace["members"] = Item::Value(Value::Array(toml_edit::Array::new()));
    }
    let members = workspace["members"]
        .as_array_mut()
        .context("members is not an array")?;

    let added_count = missing_crates.len();
    for crate_path in missing_crates {
        let rel_path = if let Ok(stripped) = crate_path.strip_prefix(workspace_root) {
            stripped.display().to_string()
        } else {
            pathdiff::diff_paths(&crate_path, workspace_root)
                .ok_or_else(|| anyhow::anyhow!("Could not create relative path"))?
                .display()
                .to_string()
        };
        eprintln!("  Adding missing workspace member: {}", rel_path);
        members.push(rel_path);
    }

    let doc_str = doc.to_string();
    if doc_str != content {
        std::fs::write(&cargo_toml_path, doc_str)?;
    }

    Ok(added_count)
}
/// Ensure crates with names starting with _ have publish = false
fn ensure_publish_false_for_internal_crates(workspace_root: &Path) -> Result<usize> {
    let members = resolve_workspace_members(workspace_root)?;
    let mut modified_count = 0;

    for member_path in members {
        let member_toml = member_path.join("Cargo.toml");
        let content = std::fs::read_to_string(&member_toml)?;
        let mut doc = content.parse::<DocumentMut>()?;

        // Get the package name
        let package_name = doc
            .get("package")
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str());

        if let Some(name) = package_name
            && name.starts_with('_')
        {
            let name_owned = name.to_string();
            // Check if publish is already set to false
            let needs_update = doc
                .get("package")
                .and_then(|p| p.get("publish"))
                .and_then(|pub_val| pub_val.as_bool())
                != Some(false);

            if needs_update {
                if doc.get("package").is_none() {
                    doc["package"] = toml_edit::table();
                }
                let package = doc["package"]
                    .as_table_mut()
                    .context("package is not a table")?;
                package["publish"] = value(false);

                let doc_str = doc.to_string();
                if doc_str != content {
                    std::fs::write(&member_toml, doc_str)?;
                    modified_count += 1;
                    eprintln!(
                        "  Setting publish = false for internal crate: {}",
                        name_owned
                    );
                }
            }
        }
    }

    Ok(modified_count)
}
/// Collect information about all workspace crates (name, version, path)
/// Used for [patch.crates-io] generation and version syncing
fn collect_workspace_crates(
    workspace_root: &Path,
    workspace_doc: &DocumentMut,
) -> Result<HashMap<String, WorkspaceCrateInfo>> {
    let members = resolve_workspace_members(workspace_root)?;
    let mut workspace_crates = HashMap::new();

    // Get workspace package version for resolving version.workspace = true
    let workspace_version = workspace_doc
        .get("workspace")
        .and_then(|w| w.get("package"))
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str())
        .map(String::from);

    for member_path in members {
        let member_toml = member_path.join("Cargo.toml");
        let content = std::fs::read_to_string(&member_toml)?;
        let doc = content.parse::<DocumentMut>()?;

        // Get package name
        let name = doc
            .get("package")
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str())
            .map(String::from);

        // Get package version (resolve workspace = true if needed)
        let version = doc
            .get("package")
            .and_then(|p| p.get("version"))
            .and_then(|v| {
                // Check if it's a simple string
                if let Some(s) = v.as_str() {
                    return Some(s.to_string());
                }
                // Check if it's { workspace = true } (inline table)
                if let Some(table) = v.as_inline_table() {
                    if table.get("workspace").and_then(|w| w.as_bool()) == Some(true) {
                        return workspace_version.clone();
                    }
                }
                // Check if it's version.workspace = true (dotted key syntax creates a table)
                if let Some(table) = v.as_table() {
                    if table.get("workspace").and_then(|w| w.as_bool()) == Some(true) {
                        return workspace_version.clone();
                    }
                }
                None
            });

        // Get relative path from workspace root
        let relative_path = if let Ok(stripped) = member_path.strip_prefix(workspace_root) {
            stripped.display().to_string()
        } else {
            pathdiff::diff_paths(&member_path, workspace_root)
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| member_path.display().to_string())
        };

        if let (Some(name), Some(version)) = (name, version) {
            workspace_crates.insert(name.clone(), WorkspaceCrateInfo {
                name,
                version,
                relative_path,
            });
        }
    }

    Ok(workspace_crates)
}
/// Ensure workspace crates have standard metadata fields set
fn ensure_workspace_metadata_inheritance(doc: &mut DocumentMut) -> Result<bool> {
    let mut modified = false;

    if doc.get("package").is_none() {
        doc["package"] = toml_edit::table();
    }
    let package = doc["package"]
        .as_table_mut()
        .context("package is not a table")?;

    // Workspace-inherited fields
    // Note: Using inline table format { workspace = true } instead of dotted key
    // because toml_edit doesn't support dotted key format for package fields
    for field in ["repository", "license", "version", "edition"] {
        if package.get(field).is_none() {
            let mut table = InlineTable::new();
            table.insert("workspace", Value::from(true));
            package[field] = Item::Value(Value::InlineTable(table));
            modified = true;
        }
    }

    // Empty default fields
    if package.get("description").is_none() {
        package["description"] = value("");
        modified = true;
    }
    if package.get("categories").is_none() {
        package["categories"] = Item::Value(Value::Array(toml_edit::Array::new()));
        modified = true;
    }
    if package.get("keywords").is_none() {
        package["keywords"] = Item::Value(Value::Array(toml_edit::Array::new()));
        modified = true;
    }

    Ok(modified)
}
/// Sort package.keywords and package.categories arrays alphabetically
fn sort_package_metadata_arrays(doc: &mut DocumentMut) -> Result<bool> {
    let mut modified = false;

    if let Some(package) = doc.get_mut("package").and_then(|p| p.as_table_mut()) {
        for field in ["keywords", "categories"] {
            if let Some(array) = package.get_mut(field).and_then(|v| v.as_array_mut()) {
                let mut items: Vec<String> = array
                    .iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect();

                let original_items = items.clone();

                // Sort with normalized comparison (- and _ treated as same)
                items.sort_by(|a, b| {
                    let (norm_a, orig_a) = normalized_name_for_sort(a);
                    let (norm_b, orig_b) = normalized_name_for_sort(b);
                    norm_a.cmp(&norm_b).then_with(|| orig_a.cmp(&orig_b))
                });

                if items != original_items {
                    array.clear();
                    for item in items {
                        array.push(item);
                    }
                    modified = true;
                }
            }
        }
    }

    Ok(modified)
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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
struct ResolutionFields {
    package: Option<String>,
    version: Option<String>,
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
/// Information about a workspace crate (for [patch.crates-io] and version
/// syncing)
#[derive(Debug, Clone)]
struct WorkspaceCrateInfo {
    /// Package name from [package].name
    name: String,
    /// Version from [package].version (resolved if workspace-inherited)
    version: String,
    /// Relative path from workspace root
    relative_path: String,
}
impl ResolutionFields {
    /// Check if two resolution fields are equal except for version
    /// compatibility
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
    if !v1.pre.is_empty() || !v2.pre.is_empty() {
        return v1 == v2;
    }
    if v1.major != 0 {
        v1.major == v2.major
    } else if v1.minor != 0 {
        v1.major == v2.major && v1.minor == v2.minor
    } else {
        v1.major == v2.major && v1.minor == v2.minor && v1.patch == v2.patch
    }
}
/// Manually normalize a path by resolving .. and . components
/// This doesn't require filesystem access
fn normalize_path_components(path: &Path) -> PathBuf {
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                if !components.is_empty()
                    && let Some(last) = components.last()
                {
                    match last {
                        std::path::Component::RootDir | std::path::Component::Prefix(_) => {}
                        _ => {
                            components.pop();
                        }
                    }
                }
            }
            std::path::Component::CurDir => {}
            other => {
                components.push(other);
            }
        }
    }
    components.iter().collect()
}
/// Normalize a version string to be parseable by semver crate
/// Handles:
/// - `=1.0.0` (exact) -> `1.0.0`
/// - `^1.0.0` (caret) -> `1.0.0`
/// - `~1.0.0` (tilde) -> `1.0.0`
/// - `1.0` (shortened) -> `1.0.0`
/// - `0.3` (shortened) -> `0.3.0`
///
/// Complex specs like `>=1.0, <2.0` cannot be normalized and will fail parsing,
/// which causes the dependency to be skipped (won't be normalized).
fn normalize_version_string(v_str: &str) -> String {
    let trimmed = v_str
        .trim_start_matches('=')
        .trim_start_matches('^')
        .trim_start_matches('~')
        .trim();
    if trimmed.contains(',') || trimmed.contains(' ') {
        return trimmed.to_string();
    }
    let parts: Vec<&str> = trimmed.split('.').collect();
    match parts.len() {
        1 => format!("{}.0.0", parts[0]),
        2 => format!("{}.{}.0", parts[0], parts[1]),
        _ => trimmed.to_string(),
    }
}
/// Parse a dependency from a TOML value
fn parse_dependency(
    key: &str,
    value: &Item,
    base_path: &Path,
    workspace_root: &Path,
    workspace_doc: Option<&DocumentMut>,
) -> Result<Option<Dependency>> {
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
            version_str = Some(s.value());
        }
        Item::Value(Value::InlineTable(t)) => {
            if let Some(workspace_val) = t.get("workspace")
                && workspace_val.as_bool() == Some(true)
            {
                if let Some(ws_doc) = workspace_doc
                    && let Some(ws_deps) = ws_doc
                        .get("workspace")
                        .and_then(|w| w.get("dependencies"))
                        .and_then(|d| d.as_table())
                    && let Some(ws_dep) = ws_deps.get(key)
                    && let Ok(Some(ws_parsed)) =
                        parse_dependency(key, ws_dep, workspace_root, workspace_root, None)
                {
                    let optional = t.get("optional").and_then(|v| v.as_bool());
                    let default_features = t.get("default-features").and_then(|v| v.as_bool());
                    let features = t.get("features").and_then(|v| {
                        v.as_array().map(|arr| {
                            arr.iter()
                                .filter_map(|item| item.as_str().map(String::from))
                                .collect()
                        })
                    });
                    let name = ws_parsed
                        .resolution
                        .package
                        .clone()
                        .unwrap_or_else(|| key.to_string());
                    return Ok(Some(Dependency {
                        key: key.to_string(),
                        name,
                        resolution: ws_parsed.resolution,
                        config: ConfigFields {
                            optional,
                            features,
                            default_features,
                        },
                    }));
                }
                return Ok(None);
            }
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
            // Handle workspace = true in dotted key syntax (e.g., anyhow.workspace = true)
            if let Some(workspace_val) = t.get("workspace")
                && workspace_val.as_bool() == Some(true)
            {
                if let Some(ws_doc) = workspace_doc
                    && let Some(ws_deps) = ws_doc
                        .get("workspace")
                        .and_then(|w| w.get("dependencies"))
                        .and_then(|d| d.as_table())
                    && let Some(ws_dep) = ws_deps.get(key)
                    && let Ok(Some(ws_parsed)) =
                        parse_dependency(key, ws_dep, workspace_root, workspace_root, None)
                {
                    let optional = t.get("optional").and_then(|v| v.as_bool());
                    let default_features = t.get("default-features").and_then(|v| v.as_bool());
                    let features = t.get("features").and_then(|v| {
                        v.as_array().map(|arr| {
                            arr.iter()
                                .filter_map(|item| item.as_str().map(String::from))
                                .collect()
                        })
                    });
                    let name = ws_parsed
                        .resolution
                        .package
                        .clone()
                        .unwrap_or_else(|| key.to_string());
                    return Ok(Some(Dependency {
                        key: key.to_string(),
                        name,
                        resolution: ws_parsed.resolution,
                        config: ConfigFields {
                            optional,
                            features,
                            default_features,
                        },
                    }));
                }
                return Ok(None);
            }
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
    let version = if let Some(v_str) = version_str {
        if v_str == "*" {
            Some(v_str.to_string())
        } else {
            let normalized = normalize_version_string(v_str);
            match Version::parse(&normalized) {
                Ok(_) => Some(normalized),
                Err(_) => {
                    return Ok(None);
                }
            }
        }
    } else {
        None
    };
    let path = if let Some(p) = path_str {
        let full_path = base_path.join(&p);
        match full_path.canonicalize() {
            Ok(canonical) => Some(canonical),
            Err(_) => Some(normalize_path_components(&full_path)),
        }
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
#[derive(Default)]
struct NormalizationStats {
    workspaces_processed: usize,
    crates_examined: usize,
    workspace_tomls_edited: usize,
    member_tomls_edited: usize,
    edited_files: HashSet<PathBuf>,
    workspace_crates_found: usize,
    patch_entries_added: usize,
    patch_entries_updated: usize,
    patch_entries_removed: usize,
    workspace_dep_versions_synced: usize,
    lockfiles_copied: usize,
}
impl NormalizationStats {
    fn record_file_edit(&mut self, path: &Path, is_workspace: bool) -> bool {
        let newly_edited = self.edited_files.insert(path.to_path_buf());
        if newly_edited {
            if is_workspace {
                self.workspace_tomls_edited += 1;
            } else {
                self.member_tomls_edited += 1;
            }
        }
        newly_edited
    }
}
/// Represents a package entry from Cargo.lock
#[derive(Debug, Clone)]
struct LockPackage {
    name: String,
    version: String,
    source: Option<String>,
    checksum: Option<String>,
    dependencies: Vec<String>,
}

/// Parse Cargo.lock (version 4 format) into a list of packages
fn parse_cargo_lock(lock_path: &Path) -> Result<(i64, Vec<LockPackage>)> {
    let content = std::fs::read_to_string(lock_path)?;
    let doc = content.parse::<DocumentMut>()?;

    let version = doc.get("version").and_then(|v| v.as_integer()).unwrap_or(4);

    let mut packages = Vec::new();

    if let Some(pkg_array) = doc.get("package").and_then(|p| p.as_array_of_tables()) {
        for pkg in pkg_array.iter() {
            let name = pkg
                .get("name")
                .and_then(|n| n.as_str())
                .unwrap_or("")
                .to_string();
            let version = pkg
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let source = pkg.get("source").and_then(|s| s.as_str()).map(String::from);
            let checksum = pkg
                .get("checksum")
                .and_then(|c| c.as_str())
                .map(String::from);

            let dependencies = pkg
                .get("dependencies")
                .and_then(|d| d.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|item| item.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();

            packages.push(LockPackage {
                name,
                version,
                source,
                checksum,
                dependencies,
            });
        }
    }

    Ok((version, packages))
}

/// Build a dependency graph from lock packages.
/// Returns a map from package name to list of package names it depends on.
/// Also returns a set of all package names that exist in the lockfile.
fn build_dependency_graph(
    packages: &[LockPackage],
) -> (HashMap<String, HashSet<String>>, HashSet<String>) {
    let mut graph: HashMap<String, HashSet<String>> = HashMap::new();
    let mut all_names: HashSet<String> = HashSet::new();

    for pkg in packages {
        all_names.insert(pkg.name.clone());
        let deps = graph.entry(pkg.name.clone()).or_default();
        for dep in &pkg.dependencies {
            deps.insert(dep.clone());
        }
    }

    (graph, all_names)
}

/// Get all transitive dependencies starting from a set of root package names.
/// Uses BFS to traverse the dependency graph.
/// Conservative: if we need any version of a package, we include the name.
fn get_transitive_dependencies(
    roots: &HashSet<String>,
    graph: &HashMap<String, HashSet<String>>,
    all_names: &HashSet<String>,
) -> HashSet<String> {
    let mut needed: HashSet<String> = HashSet::new();
    let mut queue: Vec<String> = roots.iter().cloned().collect();

    while let Some(pkg_name) = queue.pop() {
        if needed.contains(&pkg_name) {
            continue;
        }
        // Only add if it exists in the lockfile
        if all_names.contains(&pkg_name) {
            needed.insert(pkg_name.clone());
            if let Some(deps) = graph.get(&pkg_name) {
                for dep in deps {
                    if !needed.contains(dep) {
                        queue.push(dep.clone());
                    }
                }
            }
        }
    }

    needed
}

/// Parse a member's Cargo.toml to extract all dependency names
/// (from dependencies, dev-dependencies, and build-dependencies)
fn get_member_dependency_names(member_path: &Path) -> Result<HashSet<String>> {
    let cargo_toml = member_path.join("Cargo.toml");
    let content = std::fs::read_to_string(&cargo_toml)?;
    let doc = content.parse::<DocumentMut>()?;

    let mut deps = HashSet::new();

    // Get the member's own package name
    if let Some(name) = doc
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
    {
        deps.insert(name.to_string());
    }

    // Collect from all dependency sections
    for section in &["dependencies", "dev-dependencies", "build-dependencies"] {
        if let Some(deps_table) = doc.get(section).and_then(|s| s.as_table()) {
            for (key, _) in deps_table.iter() {
                // Normalize: Cargo.toml uses hyphens, but Cargo.lock uses the actual crate name
                // which may also use hyphens. The key in Cargo.toml is what we need.
                deps.insert(key.to_string());
            }
        }
    }

    Ok(deps)
}

/// Generate a pruned Cargo.lock containing only the specified packages
fn generate_pruned_lockfile(
    version: i64,
    packages: &[LockPackage],
    needed_names: &HashSet<String>,
) -> String {
    let mut output = String::new();
    output.push_str("# This file is automatically @generated by Cargo.\n");
    output.push_str("# It is not intended for manual editing.\n");
    output.push_str(&format!("version = {}\n", version));

    // Filter and sort packages by name then version for deterministic output
    let mut filtered: Vec<&LockPackage> = packages
        .iter()
        .filter(|pkg| needed_names.contains(&pkg.name))
        .collect();
    filtered.sort_by(|a, b| (&a.name, &a.version).cmp(&(&b.name, &b.version)));

    for pkg in filtered {
        output.push_str("\n[[package]]\n");
        output.push_str(&format!("name = \"{}\"\n", pkg.name));
        output.push_str(&format!("version = \"{}\"\n", pkg.version));

        if let Some(source) = &pkg.source {
            output.push_str(&format!("source = \"{}\"\n", source));
        }

        if let Some(checksum) = &pkg.checksum {
            output.push_str(&format!("checksum = \"{}\"\n", checksum));
        }

        // Only include dependencies that are in our needed set
        let filtered_deps: Vec<&String> = pkg
            .dependencies
            .iter()
            .filter(|d| needed_names.contains(*d))
            .collect();

        if !filtered_deps.is_empty() {
            output.push_str("dependencies = [\n");
            for dep in filtered_deps {
                output.push_str(&format!(" \"{}\",\n", dep));
            }
            output.push_str("]\n");
        }
    }

    output
}

/// Copy workspace Cargo.lock to all member crates, pruned to only include
/// packages that each member actually depends on (directly or transitively).
/// Errs on the side of including unnecessary entries to avoid breaking builds.
fn copy_lockfiles(workspace_root: &Path, members: &[PathBuf]) -> Result<usize> {
    let workspace_lock = workspace_root.join("Cargo.lock");

    if !workspace_lock.exists() {
        // No lock file to copy
        return Ok(0);
    }

    let (version, packages) = parse_cargo_lock(&workspace_lock)?;
    let (graph, all_names) = build_dependency_graph(&packages);

    let mut copied = 0;

    for member_path in members {
        let member_lock = member_path.join("Cargo.lock");

        // Get the member's direct dependencies
        let direct_deps = get_member_dependency_names(member_path)?;

        // Get all transitive dependencies
        let needed_names = get_transitive_dependencies(&direct_deps, &graph, &all_names);

        // Generate pruned lockfile
        let pruned_content = generate_pruned_lockfile(version, &packages, &needed_names);

        // Check if we need to write (file doesn't exist or content differs)
        let needs_write = if member_lock.exists() {
            let existing_content = std::fs::read_to_string(&member_lock)?;
            existing_content != pruned_content
        } else {
            true
        };

        if needs_write {
            std::fs::write(&member_lock, &pruned_content)?;
            copied += 1;
        }
    }

    Ok(copied)
}
/// Main normalization function
fn normalize_workspace_dependencies(
    workspace_root: &Path,
    stats: &mut NormalizationStats,
) -> Result<()> {
    eprintln!("Processing workspace: {}", workspace_root.display());
    stats.workspaces_processed += 1;
    let workspace_toml_path = workspace_root.join("Cargo.toml");
    let workspace_content = std::fs::read_to_string(&workspace_toml_path)?;
    let mut workspace_doc = workspace_content.parse::<DocumentMut>()?;
    eprintln!("Running initial cargo check...");
    let initial_check = std::process::Command::new("cargo")
        .arg("check")
        .current_dir(workspace_root)
        .output();
    let initial_build_success = match initial_check {
        Ok(output) => output.status.success(),
        Err(e) => {
            eprintln!("Warning: Failed to run initial cargo check: {}", e);
            false
        }
    };
    if !initial_build_success {
        eprintln!("Warning: Initial cargo check failed, but continuing anyway...");
    }
    let workspace_deps = workspace_doc
        .get("workspace")
        .and_then(|w| w.get("dependencies"))
        .and_then(|d| d.as_table());
    // Collect features from workspace dependencies so we can move them to member
    // crates
    let mut workspace_features: HashMap<String, Vec<String>> = HashMap::new();
    if let Some(deps_table) = workspace_deps {
        for (key, value) in deps_table.iter() {
            if let Some(features) = extract_features_from_value(value) {
                workspace_features.insert(key.to_string(), features);
            }
        }
    }
    let members = resolve_workspace_members(workspace_root)?;
    stats.crates_examined = members.len();
    eprintln!("  Found {} member crate(s)", members.len());

    // Collect workspace crate info for [patch.crates-io] and version syncing
    let workspace_crates = collect_workspace_crates(workspace_root, &workspace_doc)?;
    stats.workspace_crates_found = workspace_crates.len();

    let mut all_deps: HashMap<String, Vec<(PathBuf, String, Dependency)>> = HashMap::new();
    for member_path in &members {
        let member_toml = member_path.join("Cargo.toml");
        let content = std::fs::read_to_string(&member_toml)?;
        let doc = content.parse::<DocumentMut>()?;
        for section in &["dependencies", "dev-dependencies", "build-dependencies"] {
            if let Some(deps) = doc.get(section).and_then(|s| s.as_table()) {
                for (key, value) in deps.iter() {
                    if let Some(dep) = parse_dependency(
                        key,
                        value,
                        member_path,
                        workspace_root,
                        Some(&workspace_doc),
                    )? {
                        all_deps.entry(dep.name.clone()).or_default().push((
                            member_path.clone(),
                            section.to_string(),
                            dep,
                        ));
                    }
                }
            }
        }
    }
    let mut original_contents: HashMap<PathBuf, String> = HashMap::new();
    original_contents.insert(workspace_toml_path.clone(), workspace_content.clone());
    for member_path in &members {
        let member_toml = member_path.join("Cargo.toml");
        if let Ok(content) = std::fs::read_to_string(&member_toml) {
            original_contents.insert(member_toml, content);
        }
    }
    let mut workspace_updates: HashMap<
        String,
        (ResolutionFields, String, bool, Option<Vec<String>>),
    > = HashMap::new();
    for (dep_name, occurrences) in &all_deps {
        let mut equivalence_classes: Vec<EquivalenceClass> = Vec::new();
        for (member_path, _section, dep) in occurrences {
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
        if let Some(winner) = find_winner(&equivalence_classes) {
            let key = winner.get_preferred_key();
            // Get workspace features for this dependency key (will be propagated to
            // members)
            let ws_features = workspace_features.get(&key).cloned();
            workspace_updates.insert(
                key,
                (
                    winner.resolution.clone(),
                    dep_name.clone(),
                    winner.needs_default_features_false,
                    ws_features,
                ),
            );
        }
    }
    let old_workspace_deps = capture_old_workspace_deps(&workspace_doc, workspace_root);
    update_workspace_toml(&mut workspace_doc, &workspace_updates, workspace_root)?;

    // Add workspace crate versions to [workspace.dependencies]
    stats.workspace_dep_versions_synced =
        add_workspace_crate_versions(&mut workspace_doc, &workspace_crates)?;

    // Sort [workspace.dependencies] after all modifications are done
    if let Some(workspace) = workspace_doc.get_mut("workspace") {
        if let Some(deps) = workspace
            .get_mut("dependencies")
            .and_then(|d| d.as_table_mut())
        {
            sort_workspace_dependencies(deps, &workspace_updates)?;
        }
    }

    // Update [patch.crates-io] with all workspace crates
    let (added, updated, removed) = update_patch_crates_io(&mut workspace_doc, &workspace_crates)?;
    stats.patch_entries_added = added;
    stats.patch_entries_updated = updated;
    stats.patch_entries_removed = removed;

    let workspace_doc_str = apply_dotted_key_syntax(workspace_doc.to_string());
    if workspace_doc_str != workspace_content {
        if stats.record_file_edit(&workspace_toml_path, true) {
            eprintln!("  Editing: {}", workspace_toml_path.display());
        }
        std::fs::write(&workspace_toml_path, workspace_doc_str)?;
    }
    // Build workspace crate names set for sorting
    let workspace_crate_names: HashSet<String> = workspace_crates.keys().cloned().collect();

    for member_path in &members {
        update_member_toml(
            member_path,
            &all_deps,
            &workspace_updates,
            workspace_root,
            &workspace_doc,
            &old_workspace_deps,
            &workspace_crate_names,
            stats,
        )?;
    }
    eprintln!("Running final cargo check...");
    let final_check = std::process::Command::new("cargo")
        .arg("check")
        .current_dir(workspace_root)
        .output();
    let final_build_success = match final_check {
        Ok(output) => {
            if !output.status.success() {
                eprintln!("Final cargo check failed with stderr:");
                eprintln!("{}", String::from_utf8_lossy(&output.stderr));
                eprintln!("stdout:");
                eprintln!("{}", String::from_utf8_lossy(&output.stdout));
            }
            output.status.success()
        }
        Err(e) => {
            eprintln!("Error: Failed to run final cargo check: {}", e);
            false
        }
    };
    if initial_build_success && !final_build_success {
        eprintln!("Error: Workspace built before normalization but fails after.");
        eprintln!("NOT ROLLING BACK - DEBUG MODE");
        // eprintln!("Rolling back all changes...");
        // for (path, content) in &original_contents {
        //     if let Err(e) = std::fs::write(path, content) {
        //         eprintln!("Warning: Failed to restore {}: {}",
        // path.display(), e);     }
        // }
        // anyhow::bail!("Normalization broke the build. All changes have been
        // reverted.");
    }

    // Copy Cargo.lock to all member crates (after successful build)
    stats.lockfiles_copied = copy_lockfiles(workspace_root, &members)?;

    eprintln!("  Finished processing workspace");
    Ok(())
}
/// Represents an equivalence class of dependencies
#[derive(Debug, Clone)]
struct EquivalenceClass {
    resolution: ResolutionFields,
    votes: HashMap<PathBuf, Vec<Dependency>>,
    needs_default_features_false: bool,
}
impl EquivalenceClass {
    fn new(resolution: ResolutionFields) -> Self {
        Self {
            resolution,
            votes: HashMap::new(),
            needs_default_features_false: false,
        }
    }

    fn matches(&self, dep: &Dependency) -> bool {
        self.resolution.matches_except_version(&dep.resolution)
            && versions_compatible_opt(&self.resolution.version, &dep.resolution.version)
    }

    fn add_vote(&mut self, member: PathBuf, dep: Dependency) {
        if let (Some(current), Some(new)) = (&self.resolution.version, &dep.resolution.version) {
            match (current.as_str(), new.as_str()) {
                ("*", new_ver) if new_ver != "*" => {
                    self.resolution.version = Some(new.clone());
                }
                (current_ver, "*") if current_ver != "*" => {}
                ("*", "*") => {}
                _ => {
                    if let (Ok(curr_v), Ok(new_v)) = (Version::parse(current), Version::parse(new))
                        && new_v > curr_v
                    {
                        self.resolution.version = Some(new.clone());
                    }
                }
            }
        }
        if dep.config.default_features == Some(false) {
            self.needs_default_features_false = true;
        }
        self.votes.entry(member).or_default().push(dep);
    }

    fn vote_count(&self) -> usize {
        self.votes.len()
    }

    fn get_preferred_key(&self) -> String {
        let mut keys: Vec<String> = self
            .votes
            .values()
            .flatten()
            .map(|d| d.key.clone())
            .collect();
        keys.sort();
        keys.into_iter()
            .next()
            .unwrap_or_else(|| "unknown".to_string())
    }
}
fn versions_compatible_opt(v1: &Option<String>, v2: &Option<String>) -> bool {
    match (v1, v2) {
        (None, None) => true,
        (None, Some(_)) | (Some(_), None) => false,
        (Some(v1_str), Some(v2_str)) => {
            if v1_str == "*" || v2_str == "*" {
                return true;
            }
            match (Version::parse(v1_str), Version::parse(v2_str)) {
                (Ok(v1), Ok(v2)) => versions_compatible(&v1, &v2),
                _ => false,
            }
        }
    }
}
fn find_winner(classes: &[EquivalenceClass]) -> Option<&EquivalenceClass> {
    if classes.is_empty() {
        return None;
    }
    let max_votes = classes.iter().map(|c| c.vote_count()).max().unwrap();
    let mut candidates: Vec<&EquivalenceClass> = classes
        .iter()
        .filter(|c| c.vote_count() == max_votes)
        .collect();
    if candidates.len() == 1 {
        return Some(candidates[0]);
    }
    candidates.sort_by(|a, b| {
        match (&a.resolution.version, &b.resolution.version) {
            (Some(v1_str), Some(v2_str)) => {
                let v1_is_star = v1_str == "*";
                let v2_is_star = v2_str == "*";
                if v1_is_star && !v2_is_star {
                    return std::cmp::Ordering::Greater;
                } else if !v1_is_star && v2_is_star {
                    return std::cmp::Ordering::Less;
                } else if v1_is_star && v2_is_star {
                } else if let (Ok(v1), Ok(v2)) = (Version::parse(v1_str), Version::parse(v2_str)) {
                    let cmp = v2.cmp(&v1);
                    if cmp != std::cmp::Ordering::Equal {
                        return cmp;
                    }
                }
            }
            (Some(_), None) => return std::cmp::Ordering::Less,
            (None, Some(_)) => return std::cmp::Ordering::Greater,
            (None, None) => {}
        }
        a.resolution
            .as_sorted_pairs()
            .cmp(&b.resolution.as_sorted_pairs())
    });
    candidates.first().copied()
}
fn capture_old_workspace_deps(
    doc: &DocumentMut,
    workspace_root: &Path,
) -> HashMap<String, ResolutionFields> {
    let mut old_deps = HashMap::new();
    if let Some(workspace) = doc.get("workspace")
        && let Some(deps) = workspace.get("dependencies").and_then(|d| d.as_table())
    {
        for (key, value) in deps.iter() {
            if let Ok(Some(dep)) =
                parse_dependency(key, value, workspace_root, workspace_root, None)
            {
                old_deps.insert(key.to_string(), dep.resolution);
            }
        }
    }
    old_deps
}
fn update_workspace_toml(
    doc: &mut DocumentMut,
    updates: &HashMap<String, (ResolutionFields, String, bool, Option<Vec<String>>)>,
    workspace_root: &Path,
) -> Result<()> {
    if doc.get("workspace").is_none() {
        doc["workspace"] = toml_edit::table();
    }
    let workspace = doc["workspace"]
        .as_table_mut()
        .context("workspace is not a table")?;
    if workspace.get("dependencies").is_none() {
        workspace["dependencies"] = toml_edit::table();
    }
    let deps = workspace["dependencies"]
        .as_table_mut()
        .context("dependencies is not a table")?;
    let mut used_deps: HashSet<String> = HashSet::new();
    for (key, (resolution, _dep_name, needs_default_features_false, _ws_features)) in updates {
        used_deps.insert(key.clone());
        // Only update if the entry needs changing
        // Check if existing entry matches the resolution
        let needs_update = if let Some(existing) = deps.get(key.as_str()) {
            let existing_resolution = parse_resolution_from_value(existing).ok();
            existing_resolution.as_ref() != Some(resolution)
        } else {
            true // Entry doesn't exist, needs to be added
        };

        if needs_update {
            let value =
                build_dependency_value(resolution, workspace_root, *needs_default_features_false)?;
            deps.insert(key.as_str(), value);
        }
    }
    // DISABLED: This was deleting ALL dependencies not in updates, including
    // external deps that are correctly inherited by members but don't need
    // normalization. TODO: Implement proper cleanup that only removes truly
    // unused workspace dependencies let all_keys: Vec<String> =
    // deps.iter().map(|(k, _)| k.to_string()).collect(); for key in all_keys {
    //     if !used_deps.contains(&key) {
    //         deps.remove(&key);
    //     }
    // }

    // NOTE: Sorting is done AFTER add_workspace_crate_versions() is called
    // (see main pipeline), so it can sort all entries including workspace crates
    Ok(())
}
/// Update [patch.crates-io] section to include all workspace crates
fn update_patch_crates_io(
    doc: &mut DocumentMut,
    workspace_crates: &HashMap<String, WorkspaceCrateInfo>,
) -> Result<(usize, usize, usize)> {
    // Track stats: (added, updated, removed)
    let mut added = 0;
    let mut updated = 0;
    let mut removed = 0;

    // Get or create [patch] table
    if doc.get("patch").is_none() {
        doc["patch"] = toml_edit::table();
    }
    let patch = doc["patch"]
        .as_table_mut()
        .context("patch is not a table")?;

    // Get or create [patch.crates-io] table
    if patch.get("crates-io").is_none() {
        patch["crates-io"] = toml_edit::table();
    }
    let crates_io = patch["crates-io"]
        .as_table_mut()
        .context("crates-io is not a table")?;

    // Track existing entries for removal check
    let existing_keys: HashSet<String> = crates_io.iter().map(|(k, _)| k.to_string()).collect();
    let workspace_crate_names: HashSet<String> = workspace_crates.keys().cloned().collect();

    // Add/update entries for all workspace crates
    for (name, info) in workspace_crates {
        let mut table = InlineTable::new();
        table.insert("path", Value::from(info.relative_path.as_str()));

        if let Some(existing) = crates_io.get(name) {
            // Check if update needed
            let existing_path = existing
                .as_inline_table()
                .and_then(|t| t.get("path"))
                .and_then(|v| v.as_str());
            if existing_path != Some(&info.relative_path) {
                crates_io.insert(name, Item::Value(Value::InlineTable(table)));
                updated += 1;
            }
        } else {
            crates_io.insert(name, Item::Value(Value::InlineTable(table)));
            added += 1;
        }
    }

    // Remove stale entries (crates no longer in workspace)
    for key in &existing_keys {
        if !workspace_crate_names.contains(key) {
            crates_io.remove(key);
            removed += 1;
        }
    }

    // Sort entries alphabetically by name (with - and _ normalized)
    sort_patch_crates_io(crates_io)?;

    Ok((added, updated, removed))
}
/// Add workspace crate versions to [workspace.dependencies]
/// Returns the number of versions synced (updated or added)
fn add_workspace_crate_versions(
    doc: &mut DocumentMut,
    workspace_crates: &HashMap<String, WorkspaceCrateInfo>,
) -> Result<usize> {
    let mut synced = 0;

    // Get or create [workspace.dependencies]
    if doc.get("workspace").is_none() {
        doc["workspace"] = toml_edit::table();
    }
    let workspace = doc["workspace"]
        .as_table_mut()
        .context("workspace is not a table")?;
    if workspace.get("dependencies").is_none() {
        workspace["dependencies"] = toml_edit::table();
    }
    let deps = workspace["dependencies"]
        .as_table_mut()
        .context("dependencies is not a table")?;

    // First pass: Remove versions from internal crates (those starting with _)
    for (name, _info) in workspace_crates.iter() {
        if name.starts_with('_') {
            if let Some(existing) = deps.get(name) {
                if let Some(existing_table) = existing.as_inline_table() {
                    // Check if it has a version field
                    if existing_table.get("version").is_some() {
                        // Rebuild without version
                        let mut new_table = InlineTable::new();
                        for (k, v) in existing_table.iter() {
                            if k != "version" {
                                new_table.insert(k, v.clone());
                            }
                        }
                        deps.insert(name, Item::Value(Value::InlineTable(new_table)));
                        synced += 1;
                    }
                }
            }
        }
    }

    // Second pass: Add/update versions for non-internal crates
    for (name, info) in workspace_crates {
        // Skip internal crates (those starting with _) - they don't get versions
        if name.starts_with('_') {
            continue;
        }

        // Check if we need to add/update the entry
        let needs_update = if let Some(existing) = deps.get(name) {
            // Check existing version
            let existing_version = if let Some(s) = existing.as_str() {
                Some(s.to_string())
            } else if let Some(table) = existing.as_inline_table() {
                table
                    .get("version")
                    .and_then(|v| v.as_str())
                    .map(String::from)
            } else {
                None
            };

            // Check if it has a path field that needs to be removed
            let has_path = if let Some(table) = existing.as_inline_table() {
                table.get("path").is_some()
            } else {
                false
            };

            // Update if version differs OR if path needs to be removed
            existing_version.as_ref() != Some(&info.version) || has_path
        } else {
            true // Doesn't exist, needs to be added
        };

        if needs_update {
            // Check if existing entry has extra fields (besides version and path)
            let has_extra_fields = if let Some(existing) = deps.get(name) {
                if let Some(table) = existing.as_inline_table() {
                    // Check if there are fields other than version and path
                    table.iter().any(|(k, _)| k != "version" && k != "path")
                } else {
                    false
                }
            } else {
                false
            };

            // Need to get extra fields BEFORE removing the entry
            let extra_fields: Vec<(String, toml_edit::Value)> = if has_extra_fields {
                if let Some(existing) = deps.get(name) {
                    if let Some(existing_table) = existing.as_inline_table() {
                        existing_table
                            .iter()
                            .filter(|(k, _)| *k != "version" && *k != "path")
                            .map(|(k, v)| (k.to_string(), v.clone()))
                            .collect()
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            };

            // Remove existing entry first to ensure clean replacement
            deps.remove(name);

            if has_extra_fields {
                // Build inline table with version and preserved extra fields (but NOT path)
                let mut table = InlineTable::new();
                table.insert("version", Value::from(info.version.as_str()));

                for (key, value) in extra_fields {
                    table.insert(&key, value);
                }

                deps.insert(name, Item::Value(Value::InlineTable(table)));
            } else {
                // Simple string version (no extra fields, no path needed)
                deps.insert(name, value(info.version.clone()));
            }

            synced += 1;
        }
    }

    Ok(synced)
}
/// Sort [patch.crates-io] entries alphabetically
fn sort_patch_crates_io(table: &mut dyn toml_edit::TableLike) -> Result<()> {
    let mut entries: Vec<(String, Item)> = table
        .iter()
        .map(|(k, v)| (k.to_string(), v.clone()))
        .collect();

    // Sort by normalized name (- and _ treated as same), then original name as
    // tiebreaker
    entries.sort_by(|a, b| {
        let norm_a = a.0.to_lowercase().replace('-', "_");
        let norm_b = b.0.to_lowercase().replace('-', "_");
        norm_a.cmp(&norm_b).then_with(|| a.0.cmp(&b.0))
    });

    // Remove all and re-insert in sorted order
    let keys: Vec<String> = table.iter().map(|(k, _)| k.to_string()).collect();
    for key in keys {
        table.remove(&key);
    }
    for (key, value) in entries {
        table.insert(&key, value);
    }

    Ok(())
}
/// Helper to get normalized name for sorting (- and _ treated as same)
fn normalized_name_for_sort(name: &str) -> (String, String) {
    let normalized = name.to_lowercase().replace('-', "_");
    (normalized, name.to_string())
}
/// Post-process TOML to use dotted key syntax for workspace = true
/// Only applies when workspace = true is the ONLY field in the inline table
fn apply_dotted_key_syntax(toml_string: String) -> String {
    // Replace " = { workspace = true }" with ".workspace = true"
    // This only matches when workspace = true is the ONLY field (no commas)
    toml_string.replace(" = { workspace = true }", ".workspace = true")
}
/// Sort member dependency sections ([dependencies], [dev-dependencies],
/// [build-dependencies])
fn sort_member_dependencies(
    doc: &mut DocumentMut,
    member_package_name: &str,
    workspace_crate_names: &HashSet<String>,
) -> Result<()> {
    for section in &["dependencies", "dev-dependencies", "build-dependencies"] {
        if let Some(deps_table) = doc.get_mut(section).and_then(|s| s.as_table_mut()) {
            let mut entries: Vec<(String, Item)> = deps_table
                .iter()
                .map(|(k, v)| (k.to_string(), v.clone()))
                .collect();

            entries.sort_by(|a, b| {
                // Build category list for each dependency
                let mut cats_a = Vec::new();
                let mut cats_b = Vec::new();

                // Helper to check if uses workspace = true
                let uses_workspace = |item: &Item| {
                    item.as_inline_table()
                        .and_then(|t| t.get("workspace"))
                        .and_then(|v| v.as_bool())
                        == Some(true)
                };

                // Helper to check if has optional = true
                let is_optional = |item: &Item| {
                    item.as_inline_table()
                        .and_then(|t| t.get("optional"))
                        .and_then(|v| v.as_bool())
                        == Some(true)
                };

                // Helper to check if has git field
                let has_git =
                    |item: &Item| item.as_inline_table().and_then(|t| t.get("git")).is_some();

                // Helper to check if has extra fields
                let has_extra_fields = |item: &Item| {
                    if let Some(table) = item.as_inline_table() {
                        for key in table.iter().map(|(k, _)| k) {
                            if key != "workspace" && key != "features" && key != "default-features"
                            {
                                return true;
                            }
                        }
                    }
                    false
                };

                // Category 1: Self-dependency
                if a.0 == member_package_name {
                    cats_a.push(1);
                }
                if b.0 == member_package_name {
                    cats_b.push(1);
                }

                // Category 2: NOT using workspace = true
                if !uses_workspace(&a.1) {
                    cats_a.push(2);
                }
                if !uses_workspace(&b.1) {
                    cats_b.push(2);
                }

                // Category 3: Internal crates (workspace crates starting with _)
                if workspace_crate_names.contains(&a.0) && a.0.starts_with('_') {
                    cats_a.push(3);
                }
                if workspace_crate_names.contains(&b.0) && b.0.starts_with('_') {
                    cats_b.push(3);
                }

                // Category 4: Other workspace crates
                if workspace_crate_names.contains(&a.0) && !a.0.starts_with('_') {
                    cats_a.push(4);
                }
                if workspace_crate_names.contains(&b.0) && !b.0.starts_with('_') {
                    cats_b.push(4);
                }

                // Category 5: Git dependencies
                if has_git(&a.1) {
                    cats_a.push(5);
                }
                if has_git(&b.1) {
                    cats_b.push(5);
                }

                // Category 6: Using workspace = true
                if uses_workspace(&a.1) {
                    cats_a.push(6);
                }
                if uses_workspace(&b.1) {
                    cats_b.push(6);
                }

                // Category 7: Has extra fields
                if has_extra_fields(&a.1) {
                    cats_a.push(7);
                }
                if has_extra_fields(&b.1) {
                    cats_b.push(7);
                }

                // Category 8: Optional (at bottom)
                if is_optional(&a.1) {
                    cats_a.push(8);
                }
                if is_optional(&b.1) {
                    cats_b.push(8);
                }

                // Pad with MAX for comparison (more categories = earlier)
                while cats_a.len() < 8 {
                    cats_a.push(usize::MAX);
                }
                while cats_b.len() < 8 {
                    cats_b.push(usize::MAX);
                }

                // Compare category lists
                let cat_cmp = cats_a.cmp(&cats_b);
                if cat_cmp != std::cmp::Ordering::Equal {
                    return cat_cmp;
                }

                // Final tiebreaker: normalized name, then original name
                let (norm_a, orig_a) = normalized_name_for_sort(&a.0);
                let (norm_b, orig_b) = normalized_name_for_sort(&b.0);
                norm_a.cmp(&norm_b).then_with(|| orig_a.cmp(&orig_b))
            });

            // Remove all and re-insert in sorted order (preserves formatting)
            let keys: Vec<String> = deps_table.iter().map(|(k, _)| k.to_string()).collect();
            for key in keys {
                deps_table.remove(&key);
            }
            for (key, value) in entries {
                deps_table.insert(&key, value);
            }
        }
    }

    Ok(())
}
fn build_dependency_value(
    resolution: &ResolutionFields,
    workspace_root: &Path,
    needs_default_features_false: bool,
) -> Result<Item> {
    let mut has_extra_fields = false;
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
    if !has_extra_fields && !needs_default_features_false && resolution.version.is_some() {
        let version_str = resolution.version.as_ref().unwrap().to_string();
        return Ok(value(version_str));
    }
    let mut table = InlineTable::new();
    if let Some(ref package) = resolution.package {
        table.insert("package", Value::from(package.as_str()));
    }
    if let Some(ref version) = resolution.version {
        table.insert("version", Value::from(version.to_string()));
    }
    if let Some(ref path) = resolution.path {
        let rel_path = if let Ok(stripped) = path.strip_prefix(workspace_root) {
            stripped
        } else {
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
    if needs_default_features_false {
        table.insert("default-features", Value::from(false));
    }
    Ok(Item::Value(Value::InlineTable(table)))
}
/// Determine sort order for workspace dependencies
fn workspace_dep_sort_key(
    key: &str,
    resolution: &ResolutionFields,
) -> (bool, bool, bool, bool, bool, String) {
    (
        resolution.path.is_none(),
        resolution.git.is_none(),
        resolution.registry.is_none(),
        resolution.package.is_none(),
        resolution.version.is_some(),
        key.to_lowercase(),
    )
}
/// Parse resolution fields from a TOML value (for sorting non-updated deps)
fn parse_resolution_from_value(value: &Item) -> Result<ResolutionFields> {
    let mut resolution = ResolutionFields::default();

    // Handle simple string version (e.g., anyhow = "1.0.0")
    if let Some(version_str) = value.as_str() {
        resolution.version = Some(version_str.to_string());
        return Ok(resolution);
    }

    if let Some(table) = value.as_inline_table() {
        resolution.version = table
            .get("version")
            .and_then(|v| v.as_str())
            .map(String::from);
        resolution.path = table
            .get("path")
            .and_then(|v| v.as_str())
            .map(PathBuf::from);
        resolution.git = table.get("git").and_then(|v| v.as_str()).map(String::from);
        resolution.registry = table
            .get("registry")
            .and_then(|v| v.as_str())
            .map(String::from);
        resolution.package = table
            .get("package")
            .and_then(|v| v.as_str())
            .map(String::from);
        resolution.branch = table
            .get("branch")
            .and_then(|v| v.as_str())
            .map(String::from);
        resolution.tag = table.get("tag").and_then(|v| v.as_str()).map(String::from);
        resolution.rev = table.get("rev").and_then(|v| v.as_str()).map(String::from);
    } else if let Some(table) = value.as_table() {
        resolution.version = table
            .get("version")
            .and_then(|v| v.as_str())
            .map(String::from);
        resolution.path = table
            .get("path")
            .and_then(|v| v.as_str())
            .map(PathBuf::from);
        resolution.git = table.get("git").and_then(|v| v.as_str()).map(String::from);
        resolution.registry = table
            .get("registry")
            .and_then(|v| v.as_str())
            .map(String::from);
        resolution.package = table
            .get("package")
            .and_then(|v| v.as_str())
            .map(String::from);
        resolution.branch = table
            .get("branch")
            .and_then(|v| v.as_str())
            .map(String::from);
        resolution.tag = table.get("tag").and_then(|v| v.as_str()).map(String::from);
        resolution.rev = table.get("rev").and_then(|v| v.as_str()).map(String::from);
    } else if let Some(s) = value.as_str() {
        resolution.version = Some(s.to_string());
    }
    Ok(resolution)
}
/// Sort workspace dependencies table according to our priority rules
fn sort_workspace_dependencies(
    deps_table: &mut dyn toml_edit::TableLike,
    updates: &HashMap<String, (ResolutionFields, String, bool, Option<Vec<String>>)>,
) -> Result<()> {
    let mut entries: Vec<(String, Item, (bool, bool, bool, bool, bool, String))> = Vec::new();
    for (key, value) in deps_table.iter() {
        let key_str = key.to_string();
        let resolution = if let Some((res, _, _, _)) = updates.get(&key_str) {
            res.clone()
        } else {
            parse_resolution_from_value(value)?
        };
        let sort_key = workspace_dep_sort_key(&key_str, &resolution);
        entries.push((key_str, value.clone(), sort_key));
    }
    entries.sort_by(|a, b| a.2.cmp(&b.2));
    let keys: Vec<String> = deps_table.iter().map(|(k, _)| k.to_string()).collect();
    for key in keys {
        deps_table.remove(&key);
    }
    for (key, value, _) in entries {
        deps_table.insert(&key, value);
    }
    Ok(())
}
/// Sort key for feature dependencies
/// Returns: (category, !ends_with_default, normalized_name, original_name)
/// - category 0: bare names (no dep: prefix, no /)
/// - category 1: dependency references (with dep: or /)
/// - !ends_with_default: false sorts before true (so /default items come first)
/// - normalized_name: lexicographic ordering with - and _ normalized, dep:
///   prefix removed
/// - original_name: tiebreaker
fn feature_dep_sort_key(dep: &str) -> (u8, bool, String, String) {
    let has_dep_prefix = dep.starts_with("dep:");
    let has_slash = dep.contains('/');
    let ends_with_default = dep.ends_with("/default");

    // Category: bare names (0), then dep/slash references (1)
    let category = if has_dep_prefix || has_slash { 1 } else { 0 };

    // Remove dep: prefix for normalization
    let without_prefix = if has_dep_prefix {
        dep.strip_prefix("dep:").unwrap_or(dep)
    } else {
        dep
    };

    // Normalize - and _ for comparison
    let (normalized, _) = normalized_name_for_sort(without_prefix);

    // Use !ends_with_default so /default items sort first
    (category, !ends_with_default, normalized, dep.to_string())
}
/// Sort the [features] section in a Cargo.toml
fn sort_features_section(doc: &mut DocumentMut) -> Result<()> {
    let features_table = match doc.get_mut("features").and_then(|f| f.as_table_mut()) {
        Some(table) => table,
        None => return Ok(()), // No features section
    };

    // Step 1: Extract and sort each feature's dependency list
    let mut feature_entries: Vec<(String, Item)> = Vec::new();

    for (key, value) in features_table.iter() {
        let key_str = key.to_string();

        // Sort the dependency list if it's an array
        let sorted_value = if let Some(array) = value.as_array() {
            let mut deps: Vec<String> = array
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect();

            deps.sort_by_key(|dep| feature_dep_sort_key(dep));

            let mut new_array = toml_edit::Array::new();
            for dep in deps {
                new_array.push(dep);
            }
            Item::Value(Value::Array(new_array))
        } else {
            value.clone()
        };

        feature_entries.push((key_str, sorted_value));
    }

    // Step 2: Sort the features themselves (default first, then lexicographic)
    feature_entries.sort_by(|a, b| match (a.0.as_str(), b.0.as_str()) {
        ("default", "default") => std::cmp::Ordering::Equal,
        ("default", _) => std::cmp::Ordering::Less,
        (_, "default") => std::cmp::Ordering::Greater,
        (a_key, b_key) => a_key.cmp(b_key),
    });

    // Step 3: Rebuild the table in sorted order
    let all_keys: Vec<String> = features_table.iter().map(|(k, _)| k.to_string()).collect();
    for key in all_keys {
        features_table.remove(&key);
    }
    for (key, value) in feature_entries {
        features_table.insert(&key, value);
    }

    Ok(())
}
fn update_member_toml(
    member_path: &Path,
    _all_deps: &HashMap<String, Vec<(PathBuf, String, Dependency)>>,
    workspace_updates: &HashMap<String, (ResolutionFields, String, bool, Option<Vec<String>>)>,
    workspace_root: &Path,
    workspace_doc: &DocumentMut,
    old_workspace_deps: &HashMap<String, ResolutionFields>,
    workspace_crate_names: &HashSet<String>,
    stats: &mut NormalizationStats,
) -> Result<()> {
    let member_toml = member_path.join("Cargo.toml");
    let content = std::fs::read_to_string(&member_toml)?;
    let mut doc = content.parse::<DocumentMut>()?;

    // Get member package name for sorting
    let member_package_name = doc
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .unwrap_or("")
        .to_string();

    // Ensure workspace metadata inheritance
    ensure_workspace_metadata_inheritance(&mut doc)?;

    // Sort package metadata arrays
    sort_package_metadata_arrays(&mut doc)?;

    let mut dep_name_to_workspace_key: HashMap<String, String> = HashMap::new();
    let mut dep_name_to_needs_default_features: HashMap<String, bool> = HashMap::new();
    let mut dep_name_to_workspace_features: HashMap<String, Vec<String>> = HashMap::new();
    for (workspace_key, (_resolution, dep_name, needs_df_false, ws_features)) in workspace_updates {
        dep_name_to_workspace_key.insert(dep_name.clone(), workspace_key.clone());
        dep_name_to_needs_default_features.insert(dep_name.clone(), *needs_df_false);
        if let Some(features) = ws_features {
            dep_name_to_workspace_features.insert(dep_name.clone(), features.clone());
        }
    }
    for section in &["dependencies", "dev-dependencies", "build-dependencies"] {
        if let Some(deps) = doc.get_mut(section).and_then(|s| s.as_table_mut()) {
            let keys: Vec<String> = deps.iter().map(|(k, _)| k.to_string()).collect();
            for key in keys {
                if let Some(dep_item) = deps.get(&key) {
                    let currently_uses_workspace = dep_item
                        .as_inline_table()
                        .and_then(|t| t.get("workspace"))
                        .and_then(|v| v.as_bool())
                        == Some(true);
                    let dep = if currently_uses_workspace {
                        if let Some(old_resolution) = old_workspace_deps.get(&key) {
                            let config = extract_config_fields(dep_item);
                            Some(Dependency {
                                key: key.clone(),
                                name: old_resolution
                                    .package
                                    .clone()
                                    .unwrap_or_else(|| key.clone()),
                                resolution: old_resolution.clone(),
                                config,
                            })
                        } else {
                            parse_dependency(
                                &key,
                                dep_item,
                                member_path,
                                workspace_root,
                                Some(workspace_doc),
                            )
                            .ok()
                            .flatten()
                        }
                    } else {
                        parse_dependency(
                            &key,
                            dep_item,
                            member_path,
                            workspace_root,
                            Some(workspace_doc),
                        )
                        .ok()
                        .flatten()
                    };

                    if let Some(dep) = dep {
                        if let Some(workspace_key) = dep_name_to_workspace_key.get(&dep.name) {
                            if should_use_workspace(&dep, workspace_updates, workspace_key) {
                                let mut table = InlineTable::new();
                                table.insert("workspace", Value::from(true));
                                if let Some(optional) = dep.config.optional {
                                    table.insert("optional", Value::from(optional));
                                }
                                let workspace_needs_df_false = dep_name_to_needs_default_features
                                    .get(&dep.name)
                                    .copied()
                                    .unwrap_or(false);
                                if workspace_needs_df_false {
                                    if dep.config.default_features == Some(false) {
                                        // Use member features if present, otherwise use workspace
                                        // features
                                        let features_to_use =
                                            dep.config.features.clone().or_else(|| {
                                                dep_name_to_workspace_features
                                                    .get(&dep.name)
                                                    .cloned()
                                            });
                                        if let Some(ref features) = features_to_use {
                                            let arr: toml_edit::Array = features
                                                .iter()
                                                .map(|s| Value::from(s.as_str()))
                                                .collect();
                                            table.insert("features", Value::Array(arr));
                                        }
                                        table.insert("default-features", Value::from(false));
                                    } else {
                                        // Use member features if present, otherwise use workspace
                                        // features
                                        let base_features =
                                            dep.config.features.clone().or_else(|| {
                                                dep_name_to_workspace_features
                                                    .get(&dep.name)
                                                    .cloned()
                                            });
                                        let features_with_default =
                                            prepend_default_feature(base_features);
                                        let arr: toml_edit::Array = features_with_default
                                            .iter()
                                            .map(|s| Value::from(s.as_str()))
                                            .collect();
                                        table.insert("features", Value::Array(arr));
                                    }
                                } else {
                                    // Use member features if present, otherwise use workspace
                                    // features
                                    if let Some(ref features) = dep.config.features {
                                        let arr: toml_edit::Array = features
                                            .iter()
                                            .map(|s| Value::from(s.as_str()))
                                            .collect();
                                        table.insert("features", Value::Array(arr));
                                    } else if let Some(ws_features) =
                                        dep_name_to_workspace_features.get(&dep.name)
                                    {
                                        // Propagate workspace features to member
                                        let arr: toml_edit::Array = ws_features
                                            .iter()
                                            .map(|s| Value::from(s.as_str()))
                                            .collect();
                                        table.insert("features", Value::Array(arr));
                                    }
                                    if let Some(default_features) = dep.config.default_features {
                                        table.insert(
                                            "default-features",
                                            Value::from(default_features),
                                        );
                                    }
                                }
                                // Use dotted key syntax for simple case (workspace only)
                                if table.len() == 1 && table.contains_key("workspace") {
                                    // Use Key::parse to create a dotted key
                                    let dotted_key_str = format!("{}.workspace", key);
                                    if let Ok(dotted_key) = dotted_key_str.parse::<Key>() {
                                        deps.insert_formatted(&dotted_key, value(true));
                                    } else {
                                        // Fallback to inline table if parsing fails
                                        deps[&key] = Item::Value(Value::InlineTable(table));
                                    }
                                } else {
                                    // Use inline table for complex cases with multiple fields
                                    deps[&key] = Item::Value(Value::InlineTable(table));
                                }
                            } else if currently_uses_workspace {
                                // This dependency uses workspace but shouldn't - inline it
                                if let Some(old_resolution) = old_workspace_deps.get(&key) {
                                    let loser_dep = Dependency {
                                        key: key.clone(),
                                        name: dep.name.clone(),
                                        resolution: old_resolution.clone(),
                                        config: dep.config.clone(),
                                    };
                                    inline_dependency(&key, &loser_dep, deps, member_path)?;
                                } else {
                                    inline_dependency(&key, &dep, deps, member_path)?;
                                }
                            }
                        } else if currently_uses_workspace {
                            // This dependency uses workspace but shouldn't - inline it
                            if let Some(old_resolution) = old_workspace_deps.get(&key) {
                                let loser_dep = Dependency {
                                    key: key.clone(),
                                    name: dep.name.clone(),
                                    resolution: old_resolution.clone(),
                                    config: dep.config.clone(),
                                };
                                inline_dependency(&key, &loser_dep, deps, member_path)?;
                            } else {
                                inline_dependency(&key, &dep, deps, member_path)?;
                            }
                        }
                    }
                }
            }
        }
    }

    // Sort dependencies sections
    sort_member_dependencies(&mut doc, &member_package_name, workspace_crate_names)?;

    // Sort features section
    sort_features_section(&mut doc)?;

    let doc_str = apply_dotted_key_syntax(doc.to_string());
    if doc_str != content {
        if stats.record_file_edit(&member_toml, false) {
            eprintln!("  Editing: {}", member_toml.display());
        }
        std::fs::write(&member_toml, doc_str)?;
    }
    Ok(())
}
fn should_use_workspace(
    dep: &Dependency,
    workspace_updates: &HashMap<String, (ResolutionFields, String, bool, Option<Vec<String>>)>,
    workspace_key: &str,
) -> bool {
    if let Some((workspace_resolution, _, _, _)) = workspace_updates.get(workspace_key) {
        workspace_resolution.matches_except_version(&dep.resolution)
            && versions_compatible_opt(&workspace_resolution.version, &dep.resolution.version)
    } else {
        false
    }
}
fn inline_dependency(
    key: &str,
    dep: &Dependency,
    deps: &mut dyn toml_edit::TableLike,
    member_path: &Path,
) -> Result<()> {
    let has_extra_fields = dep.resolution.package.is_some()
        || dep.resolution.path.is_some()
        || dep.resolution.git.is_some()
        || dep.resolution.branch.is_some()
        || dep.resolution.tag.is_some()
        || dep.resolution.rev.is_some()
        || dep.resolution.registry.is_some();
    let has_config_fields = dep.config.optional.is_some()
        || dep.config.features.is_some()
        || dep.config.default_features.is_some();
    if !has_extra_fields && !has_config_fields && dep.resolution.version.is_some() {
        let version_str = dep.resolution.version.as_ref().unwrap().to_string();
        deps.insert(key, value(version_str));
        return Ok(());
    }
    let mut table = InlineTable::new();
    if let Some(ref package) = dep.resolution.package {
        table.insert("package", Value::from(package.as_str()));
    }
    if let Some(ref version) = dep.resolution.version {
        table.insert("version", Value::from(version.to_string()));
    }
    if let Some(ref path) = dep.resolution.path {
        let rel_path = if let Ok(stripped) = path.strip_prefix(member_path) {
            stripped
        } else {
            &pathdiff::diff_paths(path, member_path)
                .ok_or_else(|| anyhow::anyhow!("Could not create relative path"))?
        };
        table.insert("path", Value::from(rel_path.display().to_string()));
    }
    if let Some(ref git) = dep.resolution.git {
        table.insert("git", Value::from(git.as_str()));
    }
    if let Some(ref branch) = dep.resolution.branch {
        table.insert("branch", Value::from(branch.as_str()));
    }
    if let Some(ref tag) = dep.resolution.tag {
        table.insert("tag", Value::from(tag.as_str()));
    }
    if let Some(ref rev) = dep.resolution.rev {
        table.insert("rev", Value::from(rev.as_str()));
    }
    if let Some(ref registry) = dep.resolution.registry {
        table.insert("registry", Value::from(registry.as_str()));
    }
    if let Some(optional) = dep.config.optional {
        table.insert("optional", Value::from(optional));
    }
    if let Some(ref features) = dep.config.features {
        let arr: toml_edit::Array = features.iter().map(|s| Value::from(s.as_str())).collect();
        table.insert("features", Value::Array(arr));
    }
    if let Some(default_features) = dep.config.default_features {
        table.insert("default-features", Value::from(default_features));
    }
    deps.insert(key, Item::Value(Value::InlineTable(table)));
    Ok(())
}
fn extract_config_fields(value: &Item) -> ConfigFields {
    let table = if let Some(t) = value.as_inline_table() {
        Some(t as &dyn toml_edit::TableLike)
    } else {
        value.as_table().map(|t| t as &dyn toml_edit::TableLike)
    };
    let mut config = ConfigFields {
        optional: None,
        features: None,
        default_features: None,
    };
    if let Some(table) = table {
        config.optional = table.get("optional").and_then(|v| v.as_bool());
        config.features = table.get("features").and_then(|v| v.as_array()).map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        });
        config.default_features = table.get("default-features").and_then(|v| v.as_bool());
    }
    config
}
/// Extract features from a workspace dependency value
fn extract_features_from_value(value: &Item) -> Option<Vec<String>> {
    let table = if let Some(t) = value.as_inline_table() {
        Some(t as &dyn toml_edit::TableLike)
    } else {
        value.as_table().map(|t| t as &dyn toml_edit::TableLike)
    };

    table.and_then(|t| {
        t.get("features").and_then(|v| {
            v.as_array().map(|arr| {
                arr.iter()
                    .filter_map(|item| item.as_str().map(String::from))
                    .collect()
            })
        })
    })
}
/// Prepend "default" to features list, avoiding duplication
fn prepend_default_feature(features: Option<Vec<String>>) -> Vec<String> {
    let mut result = vec!["default".to_string()];
    if let Some(feat) = features {
        for f in feat {
            if f != "default" {
                result.push(f);
            }
        }
    }
    result
}
#[cfg(test)]
mod tests {
    use {
        super::*,
        std::fs,
        tempfile::TempDir,
    };
    fn create_test_workspace() -> Result<TempDir> {
        let dir = TempDir::new()?;
        let workspace_root = dir.path();
        fs::write(
            workspace_root.join("Cargo.toml"),
            r#"[workspace]
members = ["crate-a", "crate-b"]

[workspace.dependencies]
"#,
        )?;
        fs::create_dir(workspace_root.join("crate-a"))?;
        fs::write(
            workspace_root.join("crate-a/Cargo.toml"),
            r#"[package]
name = "crate-a"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "1.0.0"
tokio = "1.0.0"
"#,
        )?;
        fs::create_dir(workspace_root.join("crate-b"))?;
        fs::write(
            workspace_root.join("crate-b/Cargo.toml"),
            r#"[package]
name = "crate-b"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "1.0.0"
anyhow = "1.0.0"
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
        let v4 = Version::parse("0.2.3").unwrap();
        let v5 = Version::parse("0.2.7").unwrap();
        let v6 = Version::parse("0.3.0").unwrap();
        assert!(versions_compatible(&v4, &v5));
        assert!(!versions_compatible(&v4, &v6));
        let v7 = Version::parse("0.0.3").unwrap();
        let v8 = Version::parse("0.0.3").unwrap();
        let v9 = Version::parse("0.0.4").unwrap();
        assert!(versions_compatible(&v7, &v8));
        assert!(!versions_compatible(&v7, &v9));
    }
    #[test]
    fn test_normalize_shared_dependencies() -> Result<()> {
        let temp = create_test_workspace()?;
        let mut stats = NormalizationStats::default();
        normalize_workspace_dependencies(temp.path(), &mut stats)?;
        let workspace_content = fs::read_to_string(temp.path().join("Cargo.toml"))?;
        assert!(workspace_content.contains("serde"));
        let crate_a_content = fs::read_to_string(temp.path().join("crate-a/Cargo.toml"))?;
        assert!(crate_a_content.contains("workspace = true"));
        Ok(())
    }
    #[test]
    fn test_idempotency_with_workspace_true() -> Result<()> {
        let temp = create_test_workspace()?;
        let mut stats = NormalizationStats::default();
        normalize_workspace_dependencies(temp.path(), &mut stats)?;
        let workspace_content_1 = fs::read_to_string(temp.path().join("Cargo.toml"))?;
        let crate_a_content_1 = fs::read_to_string(temp.path().join("crate-a/Cargo.toml"))?;
        let mut stats = NormalizationStats::default();
        normalize_workspace_dependencies(temp.path(), &mut stats)?;
        let workspace_content_2 = fs::read_to_string(temp.path().join("Cargo.toml"))?;
        let crate_a_content_2 = fs::read_to_string(temp.path().join("crate-a/Cargo.toml"))?;
        assert_eq!(
            workspace_content_1, workspace_content_2,
            "Workspace Cargo.toml changed on second run"
        );
        assert_eq!(
            crate_a_content_1, crate_a_content_2,
            "Member Cargo.toml changed on second run"
        );
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
    #[test]
    fn test_loser_inlining() -> Result<()> {
        let temp = TempDir::new()?;
        let workspace_root = temp.path();
        fs::write(
            workspace_root.join("Cargo.toml"),
            r#"[workspace]
members = ["crate-a", "crate-b", "crate-c"]

[workspace.dependencies]
"#,
        )?;
        fs::create_dir(workspace_root.join("crate-a"))?;
        fs::write(
            workspace_root.join("crate-a/Cargo.toml"),
            r#"[package]
name = "crate-a"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "1.0.0"
"#,
        )?;
        fs::create_dir(workspace_root.join("crate-b"))?;
        fs::write(
            workspace_root.join("crate-b/Cargo.toml"),
            r#"[package]
name = "crate-b"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "1.0.0"
"#,
        )?;
        fs::create_dir(workspace_root.join("crate-c"))?;
        fs::write(
            workspace_root.join("crate-c/Cargo.toml"),
            r#"[package]
name = "crate-c"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "2.0.0"
"#,
        )?;
        let mut stats = NormalizationStats::default();
        normalize_workspace_dependencies(workspace_root, &mut stats)?;
        let workspace_content = fs::read_to_string(workspace_root.join("Cargo.toml"))?;
        assert!(
            workspace_content.contains("serde = \"1.0.0\""),
            "serde 1.0.0 should win initially"
        );
        let crate_a_content = fs::read_to_string(workspace_root.join("crate-a/Cargo.toml"))?;
        let crate_b_content = fs::read_to_string(workspace_root.join("crate-b/Cargo.toml"))?;
        assert!(
            crate_a_content.contains("workspace = true"),
            "crate-a should use workspace = true"
        );
        assert!(
            crate_b_content.contains("workspace = true"),
            "crate-b should use workspace = true"
        );
        let crate_c_content = fs::read_to_string(workspace_root.join("crate-c/Cargo.toml"))?;
        assert!(
            crate_c_content.contains("serde = \"2.0.0\""),
            "crate-c should have serde 2.0.0 inlined"
        );
        // Check that serde dependency doesn't use workspace (may have workspace
        // metadata fields though)
        assert!(
            !crate_c_content.contains("serde = { workspace = true }")
                && !crate_c_content.contains("serde.workspace = true"),
            "crate-c serde dependency should not use workspace"
        );
        fs::create_dir(workspace_root.join("crate-d"))?;
        fs::write(
            workspace_root.join("crate-d/Cargo.toml"),
            r#"[package]
name = "crate-d"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "2.0.0"
"#,
        )?;
        fs::write(
            workspace_root.join("Cargo.toml"),
            workspace_content.replace(
                r#"members = ["crate-a", "crate-b", "crate-c"]"#,
                r#"members = ["crate-a", "crate-b", "crate-c", "crate-d"]"#,
            ),
        )?;
        let mut stats = NormalizationStats::default();
        normalize_workspace_dependencies(workspace_root, &mut stats)?;
        let workspace_content_2 = fs::read_to_string(workspace_root.join("Cargo.toml"))?;
        assert!(
            workspace_content_2.contains("serde = \"2.0.0\""),
            "serde 2.0.0 should win after adding crate-d"
        );
        let crate_a_content_2 = fs::read_to_string(workspace_root.join("crate-a/Cargo.toml"))?;
        let crate_b_content_2 = fs::read_to_string(workspace_root.join("crate-b/Cargo.toml"))?;
        assert!(
            crate_a_content_2.contains("serde = \"1.0.0\""),
            "crate-a should have serde 1.0.0 inlined. Content:\n{}",
            crate_a_content_2
        );
        // Check that serde dependency doesn't use workspace (may have workspace
        // metadata fields though)
        assert!(
            !crate_a_content_2.contains("serde = { workspace = true }")
                && !crate_a_content_2.contains("serde.workspace = true"),
            "crate-a serde dependency should not use workspace"
        );
        assert!(
            crate_b_content_2.contains("serde = \"1.0.0\""),
            "crate-b should have serde 1.0.0 inlined"
        );
        assert!(
            !crate_b_content_2.contains("serde = { workspace = true }")
                && !crate_b_content_2.contains("serde.workspace = true"),
            "crate-b serde dependency should not use workspace"
        );
        let crate_c_content_2 = fs::read_to_string(workspace_root.join("crate-c/Cargo.toml"))?;
        let crate_d_content = fs::read_to_string(workspace_root.join("crate-d/Cargo.toml"))?;
        assert!(
            crate_c_content_2.contains("workspace = true")
                || crate_c_content_2.contains("serde.workspace = true"),
            "crate-c should use workspace inheritance (either inline table or dotted key)"
        );
        assert!(
            crate_d_content.contains("workspace = true")
                || crate_d_content.contains("serde.workspace = true"),
            "crate-d should use workspace inheritance (either inline table or dotted key)"
        );
        Ok(())
    }

    #[test]
    fn test_workspace_metadata_inheritance() -> Result<()> {
        let toml_content = r#"[package]
name = "test-crate"
"#;

        let mut doc = toml_content.parse::<DocumentMut>()?;
        ensure_workspace_metadata_inheritance(&mut doc)?;

        let result = doc.to_string();

        // Check workspace inheritance (inline table format)
        assert!(
            result.contains("repository.workspace = true")
                || result.contains("repository = { workspace = true }"),
            "Should have repository workspace inheritance"
        );
        assert!(
            result.contains("license.workspace = true")
                || result.contains("license = { workspace = true }"),
            "Should have license workspace inheritance"
        );
        assert!(
            result.contains("version.workspace = true")
                || result.contains("version = { workspace = true }"),
            "Should have version workspace inheritance"
        );
        assert!(
            result.contains("edition.workspace = true")
                || result.contains("edition = { workspace = true }"),
            "Should have edition workspace inheritance"
        );

        // Check empty values
        assert!(
            result.contains(r#"description = """#),
            "Should have empty description"
        );
        assert!(
            result.contains("categories = []"),
            "Should have empty categories"
        );
        assert!(
            result.contains("keywords = []"),
            "Should have empty keywords"
        );

        Ok(())
    }

    #[test]
    fn test_workspace_metadata_preserves_existing() -> Result<()> {
        let toml_content = r#"[package]
name = "test-crate"
version = "1.0.0"
description = "Custom description"
"#;

        let mut doc = toml_content.parse::<DocumentMut>()?;
        ensure_workspace_metadata_inheritance(&mut doc)?;

        let result = doc.to_string();

        // Should preserve existing values
        assert!(
            result.contains(r#"version = "1.0.0""#),
            "Should preserve existing version"
        );
        assert!(
            result.contains(r#"description = "Custom description""#),
            "Should preserve existing description"
        );

        // Should add missing ones (inline table format)
        assert!(
            result.contains("repository.workspace = true")
                || result.contains("repository = { workspace = true }"),
            "Should add repository workspace inheritance"
        );
        assert!(
            result.contains("license.workspace = true")
                || result.contains("license = { workspace = true }"),
            "Should add license workspace inheritance"
        );

        Ok(())
    }

    #[test]
    fn test_features_section_sorting() -> Result<()> {
        let toml_content = r#"[package]
name = "test"
version = "0.1.0"

[features]
wasm = ["dep:wasm-bindgen", "jeb-value/wasm"]
bin = ["default", "fs", "dep:color-eyre", "jeb-stream/stdio"]
default = ["serde"]
"#;

        let mut doc = toml_content.parse::<DocumentMut>()?;
        sort_features_section(&mut doc)?;

        let result = doc.to_string();
        let features_start = result.find("[features]").unwrap();
        let features_section = &result[features_start..];

        // Check that "default" comes first
        let default_pos = features_section.find("default = ").unwrap();
        let bin_pos = features_section.find("bin = ").unwrap();
        let wasm_pos = features_section.find("wasm = ").unwrap();

        assert!(default_pos < bin_pos, "default should come before bin");
        assert!(bin_pos < wasm_pos, "bin should come before wasm");

        Ok(())
    }

    #[test]
    fn test_feature_deps_sorting() -> Result<()> {
        let toml_content = r#"[features]
test = ["dep:color-eyre", "default", "fs", "jeb-stream/stdio", "dep:anyhow"]
"#;

        let mut doc = toml_content.parse::<DocumentMut>()?;
        sort_features_section(&mut doc)?;

        let result = doc.to_string();

        // Should be: bare names first (default, fs), then dep: items sorted
        assert!(
            result.contains(
                r#"test = ["default", "fs", "dep:anyhow", "dep:color-eyre", "jeb-stream/stdio"]"#
            ),
            "Features should be sorted: bare names first, then dep/slash references"
        );

        Ok(())
    }

    #[test]
    fn test_feature_deps_default_suffix_priority() -> Result<()> {
        let toml_content = r#"[features]
test = ["jeb-stream/stdio", "jeb-value/default", "dep:color-eyre", "jeb-stream/default"]
"#;

        let mut doc = toml_content.parse::<DocumentMut>()?;
        sort_features_section(&mut doc)?;

        let result = doc.to_string();

        // Items ending with /default should come before other items in category 1
        // Expected order: items with /default first (sorted), then items without
        // (sorted) (sorted by: category 1, !ends_with_default [false=has
        // /default, true=no /default], normalized name)
        assert!(
            result.contains(r#"test = ["jeb-stream/default", "jeb-value/default", "dep:color-eyre", "jeb-stream/stdio"]"#),
            "Items with /default should sort first, then other items, but actual result was:\n{}",
            result
        );

        Ok(())
    }

    #[test]
    fn test_no_features_section() -> Result<()> {
        let toml_content = r#"[package]
name = "test"
version = "0.1.0"
"#;

        let mut doc = toml_content.parse::<DocumentMut>()?;
        // Should not error when there's no features section
        sort_features_section(&mut doc)?;

        Ok(())
    }

    #[test]
    fn test_workspace_features_propagation() -> Result<()> {
        let temp = TempDir::new()?;
        let workspace_root = temp.path();

        // Create workspace with a dependency that has features
        fs::write(
            workspace_root.join("Cargo.toml"),
            r#"[workspace]
members = ["crate-a", "crate-b"]

[workspace.dependencies]
serde = { version = "1.0", features = ["derive", "alloc"] }
"#,
        )?;

        // Create crate-a that uses workspace dependency without specifying features
        fs::create_dir(workspace_root.join("crate-a"))?;
        fs::write(
            workspace_root.join("crate-a/Cargo.toml"),
            r#"[package]
name = "crate-a"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { workspace = true }
"#,
        )?;

        // Create crate-b that uses workspace dependency with its own features
        fs::create_dir(workspace_root.join("crate-b"))?;
        fs::write(
            workspace_root.join("crate-b/Cargo.toml"),
            r#"[package]
name = "crate-b"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { workspace = true, features = ["rc"] }
"#,
        )?;

        let mut stats = NormalizationStats::default();
        normalize_workspace_dependencies(workspace_root, &mut stats)?;

        // Check workspace - features should be removed
        let workspace_content = fs::read_to_string(workspace_root.join("Cargo.toml"))?;
        assert!(
            !workspace_content.contains(r#"features = ["derive", "alloc"]"#),
            "Workspace should not have features in serde dependency after normalization"
        );

        // Check crate-a - should have workspace features propagated
        let crate_a_content = fs::read_to_string(workspace_root.join("crate-a/Cargo.toml"))?;
        assert!(
            crate_a_content.contains("derive") && crate_a_content.contains("alloc"),
            "crate-a should have workspace features propagated. Content:\n{}",
            crate_a_content
        );

        // Check crate-b - should keep its own features (not get workspace features)
        let crate_b_content = fs::read_to_string(workspace_root.join("crate-b/Cargo.toml"))?;
        assert!(
            crate_b_content.contains("rc"),
            "crate-b should keep its own features"
        );
        assert!(
            !crate_b_content.contains("derive"),
            "crate-b should NOT get workspace features since it has its own. Content:\n{}",
            crate_b_content
        );

        Ok(())
    }
}
