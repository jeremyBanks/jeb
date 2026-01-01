use {
    anyhow::{Context, Result},
    glob::glob, semver::Version,
    std::{
        collections::{HashMap, HashSet},
        path::{Path, PathBuf},
    },
    toml_edit::{DocumentMut, InlineTable, Item, Value, value},
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
    let mut stats = NormalizationStats::default();
    normalize_workspace_dependencies(&workspace_root, &mut stats)?;
    eprintln!("\nSummary:");
    eprintln!("  Workspaces examined: {}", stats.workspaces_processed);
    eprintln!("  Crates examined: {}", stats.crates_examined);
    eprintln!("  Workspace Cargo.toml files edited: {}", stats.workspace_tomls_edited);
    eprintln!("  Member Cargo.toml files edited: {}", stats.member_tomls_edited);
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
impl ResolutionFields {
    /// Check if two resolution fields are equal except for version
    /// compatibility
    fn matches_except_version(&self, other: &Self) -> bool {
        self.package == other.package && self.path == other.path && self.git == other.git
            && self.branch == other.branch && self.tag == other.tag
            && self.rev == other.rev && self.registry == other.registry
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
                if !components.is_empty() && let Some(last) = components.last() {
                    match last {
                        std::path::Component::RootDir
                        | std::path::Component::Prefix(_) => {}
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
                {
                    if let Ok(Some(ws_parsed)) = parse_dependency(
                        key,
                        ws_dep,
                        workspace_root,
                        workspace_root,
                        None,
                    ) {
                        let optional = t.get("optional").and_then(|v| v.as_bool());
                        let default_features = t
                            .get("default-features")
                            .and_then(|v| v.as_bool());
                        let features = t
                            .get("features")
                            .and_then(|v| {
                                v.as_array()
                                    .map(|arr| {
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
                        return Ok(
                            Some(Dependency {
                                key: key.to_string(),
                                name,
                                resolution: ws_parsed.resolution,
                                config: ConfigFields {
                                    optional,
                                    features,
                                    default_features,
                                },
                            }),
                        );
                    }
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
            features = t
                .get("features")
                .and_then(|v| {
                    v.as_array()
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|item| item.as_str().map(String::from))
                                .collect()
                        })
                });
        }
        Item::Table(t) => {
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
            features = t
                .get("features")
                .and_then(|v| {
                    v.as_array()
                        .map(|arr| {
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
    Ok(
        Some(Dependency {
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
        }),
    )
}
#[derive(Default)]
struct NormalizationStats {
    workspaces_processed: usize,
    crates_examined: usize,
    workspace_tomls_edited: usize,
    member_tomls_edited: usize,
    edited_files: HashSet<PathBuf>,
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
    let members = resolve_workspace_members(workspace_root)?;
    stats.crates_examined = members.len();
    eprintln!("  Found {} member crate(s)", members.len());
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
                    if let Some(dep) = parse_dependency(
                        key,
                        value,
                        member_path,
                        workspace_root,
                        Some(&workspace_doc),
                    )? {
                        all_deps
                            .entry(dep.name.clone())
                            .or_default()
                            .push((member_path.clone(), section.to_string(), dep));
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
    let mut workspace_updates: HashMap<String, (ResolutionFields, String, bool)> = HashMap::new();
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
            workspace_updates
                .insert(
                    key,
                    (
                        winner.resolution.clone(),
                        dep_name.clone(),
                        winner.needs_default_features_false,
                    ),
                );
        }
    }
    let old_workspace_deps = capture_old_workspace_deps(&workspace_doc, workspace_root);
    update_workspace_toml(&mut workspace_doc, &workspace_updates, workspace_root)?;
    let workspace_doc_str = workspace_doc.to_string();
    if workspace_doc_str != workspace_content {
        if stats.record_file_edit(&workspace_toml_path, true) {
            eprintln!("  Editing: {}", workspace_toml_path.display());
        }
        std::fs::write(&workspace_toml_path, workspace_doc_str)?;
    }
    for member_path in &members {
        update_member_toml(
            member_path,
            &all_deps,
            &workspace_updates,
            workspace_root,
            &workspace_doc,
            &old_workspace_deps,
            stats,
        )?;
    }
    eprintln!("Running final cargo check...");
    let final_check = std::process::Command::new("cargo")
        .arg("check")
        .current_dir(workspace_root)
        .output();
    let final_build_success = match final_check {
        Ok(output) => output.status.success(),
        Err(e) => {
            eprintln!("Error: Failed to run final cargo check: {}", e);
            false
        }
    };
    if initial_build_success && !final_build_success {
        eprintln!("Error: Workspace built before normalization but fails after.");
        eprintln!("Rolling back all changes...");
        for (path, content) in &original_contents {
            if let Err(e) = std::fs::write(path, content) {
                eprintln!("Warning: Failed to restore {}: {}", path.display(), e);
            }
        }
        anyhow::bail!("Normalization broke the build. All changes have been reverted.");
    }
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
        if let (Some(current), Some(new)) = (
            &self.resolution.version,
            &dep.resolution.version,
        ) {
            match (current.as_str(), new.as_str()) {
                ("*", new_ver) if new_ver != "*" => {
                    self.resolution.version = Some(new.clone());
                }
                (current_ver, "*") if current_ver != "*" => {}
                ("*", "*") => {}
                _ => {
                    match (Version::parse(current), Version::parse(new)) {
                        (Ok(curr_v), Ok(new_v)) => {
                            if new_v > curr_v {
                                self.resolution.version = Some(new.clone());
                            }
                        }
                        _ => {}
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
        keys.into_iter().next().unwrap_or_else(|| "unknown".to_string())
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
    candidates
        .sort_by(|a, b| {
            match (&a.resolution.version, &b.resolution.version) {
                (Some(v1_str), Some(v2_str)) => {
                    let v1_is_star = v1_str == "*";
                    let v2_is_star = v2_str == "*";
                    if v1_is_star && !v2_is_star {
                        return std::cmp::Ordering::Greater;
                    } else if !v1_is_star && v2_is_star {
                        return std::cmp::Ordering::Less;
                    } else if v1_is_star && v2_is_star {} else {
                        match (Version::parse(v1_str), Version::parse(v2_str)) {
                            (Ok(v1), Ok(v2)) => {
                                let cmp = v2.cmp(&v1);
                                if cmp != std::cmp::Ordering::Equal {
                                    return cmp;
                                }
                            }
                            _ => {}
                        }
                    }
                }
                (Some(_), None) => return std::cmp::Ordering::Less,
                (None, Some(_)) => return std::cmp::Ordering::Greater,
                (None, None) => {}
            }
            a.resolution.as_sorted_pairs().cmp(&b.resolution.as_sorted_pairs())
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
            if let Ok(Some(dep)) = parse_dependency(
                key,
                value,
                workspace_root,
                workspace_root,
                None,
            ) {
                old_deps.insert(key.to_string(), dep.resolution);
            }
        }
    }
    old_deps
}
fn update_workspace_toml(
    doc: &mut DocumentMut,
    updates: &HashMap<String, (ResolutionFields, String, bool)>,
    workspace_root: &Path,
) -> Result<()> {
    if doc.get("workspace").is_none() {
        doc["workspace"] = toml_edit::table();
    }
    let workspace = doc["workspace"].as_table_mut().context("workspace is not a table")?;
    if workspace.get("dependencies").is_none() {
        workspace["dependencies"] = toml_edit::table();
    }
    let deps = workspace["dependencies"]
        .as_table_mut()
        .context("dependencies is not a table")?;
    let mut used_deps: HashSet<String> = HashSet::new();
    for (key, (resolution, _dep_name, needs_default_features_false)) in updates {
        used_deps.insert(key.clone());
        let value = build_dependency_value(
            resolution,
            workspace_root,
            *needs_default_features_false,
        )?;
        deps.insert(key.as_str(), value);
    }
    let all_keys: Vec<String> = deps.iter().map(|(k, _)| k.to_string()).collect();
    for key in all_keys {
        if !used_deps.contains(&key) {
            deps.remove(&key);
        }
    }
    sort_workspace_dependencies(deps, updates)?;
    Ok(())
}
fn build_dependency_value(
    resolution: &ResolutionFields,
    workspace_root: &Path,
    needs_default_features_false: bool,
) -> Result<Item> {
    let mut has_extra_fields = false;
    if resolution.package.is_some() || resolution.path.is_some()
        || resolution.git.is_some() || resolution.branch.is_some()
        || resolution.tag.is_some() || resolution.rev.is_some()
        || resolution.registry.is_some()
    {
        has_extra_fields = true;
    }
    if !has_extra_fields && !needs_default_features_false && resolution.version.is_some()
    {
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
    if let Some(table) = value.as_inline_table() {
        resolution.version = table
            .get("version")
            .and_then(|v| v.as_str())
            .map(String::from);
        resolution.path = table.get("path").and_then(|v| v.as_str()).map(PathBuf::from);
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
        resolution.path = table.get("path").and_then(|v| v.as_str()).map(PathBuf::from);
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
    updates: &HashMap<String, (ResolutionFields, String, bool)>,
) -> Result<()> {
    let mut entries: Vec<(String, Item, (bool, bool, bool, bool, bool, String))> = Vec::new();
    for (key, value) in deps_table.iter() {
        let key_str = key.to_string();
        let resolution = if let Some((res, _, _)) = updates.get(&key_str) {
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
fn update_member_toml(
    member_path: &Path,
    _all_deps: &HashMap<String, Vec<(PathBuf, String, Dependency)>>,
    workspace_updates: &HashMap<String, (ResolutionFields, String, bool)>,
    workspace_root: &Path,
    workspace_doc: &DocumentMut,
    old_workspace_deps: &HashMap<String, ResolutionFields>,
    stats: &mut NormalizationStats,
) -> Result<()> {
    let member_toml = member_path.join("Cargo.toml");
    let content = std::fs::read_to_string(&member_toml)?;
    let mut doc = content.parse::<DocumentMut>()?;
    let mut dep_name_to_workspace_key: HashMap<String, String> = HashMap::new();
    let mut dep_name_to_needs_default_features: HashMap<String, bool> = HashMap::new();
    for (workspace_key, (_resolution, dep_name, needs_df_false)) in workspace_updates {
        dep_name_to_workspace_key.insert(dep_name.clone(), workspace_key.clone());
        dep_name_to_needs_default_features.insert(dep_name.clone(), *needs_df_false);
    }
    for section in &["dependencies", "dev-dependencies", "build-dependencies"] {
        if let Some(deps) = doc.get_mut(section).and_then(|s| s.as_table_mut()) {
            let keys: Vec<String> = deps.iter().map(|(k, _)| k.to_string()).collect();
            for key in keys {
                if let Some(dep_item) = deps.get(&key) {
                    let currently_uses_workspace = dep_item
                        .as_inline_table()
                        .and_then(|t| t.get("workspace"))
                        .and_then(|v| v.as_bool()) == Some(true);
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
                        if let Some(workspace_key) = dep_name_to_workspace_key
                            .get(&dep.name)
                        {
                            if should_use_workspace(
                                &dep,
                                workspace_updates,
                                workspace_key,
                            ) {
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
                                        if let Some(ref features) = dep.config.features {
                                            let arr: toml_edit::Array = features
                                                .iter()
                                                .map(|s| Value::from(s.as_str()))
                                                .collect();
                                            table.insert("features", Value::Array(arr));
                                        }
                                        table.insert("default-features", Value::from(false));
                                    } else {
                                        let features_with_default = prepend_default_feature(
                                            dep.config.features.clone(),
                                        );
                                        let arr: toml_edit::Array = features_with_default
                                            .iter()
                                            .map(|s| Value::from(s.as_str()))
                                            .collect();
                                        table.insert("features", Value::Array(arr));
                                    }
                                } else {
                                    if let Some(ref features) = dep.config.features {
                                        let arr: toml_edit::Array = features
                                            .iter()
                                            .map(|s| Value::from(s.as_str()))
                                            .collect();
                                        table.insert("features", Value::Array(arr));
                                    }
                                    if let Some(default_features) = dep.config.default_features
                                    {
                                        table
                                            .insert("default-features", Value::from(default_features));
                                    }
                                }
                                deps[&key] = Item::Value(Value::InlineTable(table));
                            } else if currently_uses_workspace {
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
    let doc_str = doc.to_string();
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
    workspace_updates: &HashMap<String, (ResolutionFields, String, bool)>,
    workspace_key: &str,
) -> bool {
    if let Some((workspace_resolution, _, _)) = workspace_updates.get(workspace_key) {
        workspace_resolution.matches_except_version(&dep.resolution)
            && versions_compatible_opt(
                &workspace_resolution.version,
                &dep.resolution.version,
            )
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
        || dep.resolution.path.is_some() || dep.resolution.git.is_some()
        || dep.resolution.branch.is_some() || dep.resolution.tag.is_some()
        || dep.resolution.rev.is_some() || dep.resolution.registry.is_some();
    let has_config_fields = dep.config.optional.is_some()
        || dep.config.features.is_some() || dep.config.default_features.is_some();
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
        let arr: toml_edit::Array = features
            .iter()
            .map(|s| Value::from(s.as_str()))
            .collect();
        table.insert("features", Value::Array(arr));
    }
    if let Some(default_features) = dep.config.default_features {
        table.insert("default-features", Value::from(default_features));
    }
    deps.insert(key, Item::Value(Value::InlineTable(table)));
    Ok(())
}
fn has_config_fields(value: &Item) -> bool {
    if let Some(table) = value.as_inline_table() {
        return table.contains_key("optional") || table.contains_key("features");
    }
    if let Some(table) = value.as_table() {
        return table.contains_key("optional") || table.contains_key("features");
    }
    false
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
        config.features = table
            .get("features")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter().filter_map(|v| v.as_str().map(String::from)).collect()
            });
        config.default_features = table
            .get("default-features")
            .and_then(|v| v.as_bool());
    }
    config
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
    use {super::*, std::fs, tempfile::TempDir};
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
        assert_eq!(workspace_root, temp.path().canonicalize() ?);
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
        assert!(versions_compatible(& v1, & v2));
        assert!(! versions_compatible(& v1, & v3));
        let v4 = Version::parse("0.2.3").unwrap();
        let v5 = Version::parse("0.2.7").unwrap();
        let v6 = Version::parse("0.3.0").unwrap();
        assert!(versions_compatible(& v4, & v5));
        assert!(! versions_compatible(& v4, & v6));
        let v7 = Version::parse("0.0.3").unwrap();
        let v8 = Version::parse("0.0.3").unwrap();
        let v9 = Version::parse("0.0.4").unwrap();
        assert!(versions_compatible(& v7, & v8));
        assert!(! versions_compatible(& v7, & v9));
    }
    #[test]
    fn test_normalize_shared_dependencies() -> Result<()> {
        let temp = create_test_workspace()?;
        let mut stats = NormalizationStats::default();
        normalize_workspace_dependencies(temp.path(), &mut stats)?;
        let workspace_content = fs::read_to_string(temp.path().join("Cargo.toml"))?;
        assert!(workspace_content.contains("serde"));
        let crate_a_content = fs::read_to_string(
            temp.path().join("crate-a/Cargo.toml"),
        )?;
        assert!(crate_a_content.contains("workspace = true"));
        Ok(())
    }
    #[test]
    fn test_idempotency_with_workspace_true() -> Result<()> {
        let temp = create_test_workspace()?;
        let mut stats = NormalizationStats::default();
        normalize_workspace_dependencies(temp.path(), &mut stats)?;
        let workspace_content_1 = fs::read_to_string(temp.path().join("Cargo.toml"))?;
        let crate_a_content_1 = fs::read_to_string(
            temp.path().join("crate-a/Cargo.toml"),
        )?;
        let mut stats = NormalizationStats::default();
        normalize_workspace_dependencies(temp.path(), &mut stats)?;
        let workspace_content_2 = fs::read_to_string(temp.path().join("Cargo.toml"))?;
        let crate_a_content_2 = fs::read_to_string(
            temp.path().join("crate-a/Cargo.toml"),
        )?;
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
        assert!(versions_compatible(& v1, & v2));
        assert!(! versions_compatible(& v1, & v3));
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
        let crate_a_content = fs::read_to_string(
            workspace_root.join("crate-a/Cargo.toml"),
        )?;
        let crate_b_content = fs::read_to_string(
            workspace_root.join("crate-b/Cargo.toml"),
        )?;
        assert!(
            crate_a_content.contains("workspace = true"),
            "crate-a should use workspace = true"
        );
        assert!(
            crate_b_content.contains("workspace = true"),
            "crate-b should use workspace = true"
        );
        let crate_c_content = fs::read_to_string(
            workspace_root.join("crate-c/Cargo.toml"),
        )?;
        assert!(
            crate_c_content.contains("serde = \"2.0.0\""),
            "crate-c should have serde 2.0.0 inlined"
        );
        assert!(
            ! crate_c_content.contains("workspace = true"),
            "crate-c should not use workspace = true"
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
            workspace_content
                .replace(
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
        let crate_a_content_2 = fs::read_to_string(
            workspace_root.join("crate-a/Cargo.toml"),
        )?;
        let crate_b_content_2 = fs::read_to_string(
            workspace_root.join("crate-b/Cargo.toml"),
        )?;
        assert!(
            crate_a_content_2.contains("serde = \"1.0.0\""),
            "crate-a should have serde 1.0.0 inlined"
        );
        assert!(
            ! crate_a_content_2.contains("workspace = true"),
            "crate-a should not use workspace = true"
        );
        assert!(
            crate_b_content_2.contains("serde = \"1.0.0\""),
            "crate-b should have serde 1.0.0 inlined"
        );
        assert!(
            ! crate_b_content_2.contains("workspace = true"),
            "crate-b should not use workspace = true"
        );
        let crate_c_content_2 = fs::read_to_string(
            workspace_root.join("crate-c/Cargo.toml"),
        )?;
        let crate_d_content = fs::read_to_string(
            workspace_root.join("crate-d/Cargo.toml"),
        )?;
        assert!(
            crate_c_content_2.contains("workspace = true"),
            "crate-c should use workspace = true"
        );
        assert!(
            crate_d_content.contains("workspace = true"),
            "crate-d should use workspace = true"
        );
        Ok(())
    }
}
