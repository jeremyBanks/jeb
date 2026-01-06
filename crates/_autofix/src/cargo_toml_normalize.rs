use {
    crate::lockfile::{
        build_dependency_graph,
        parse_cargo_lock,
    },
    anyhow::{
        Context,
        Result,
    },
    glob::glob,
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
        Item,
        Value,
    },
};

pub fn main() -> i32 {
    eprintln!("Running: Cargo.toml normalization (features and section ordering)");
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
    let members = resolve_workspace_members(&workspace_root)?;

    // Parse workspace Cargo.lock for dependency graph
    let workspace_lock = workspace_root.join("Cargo.lock");
    let lockfile_graph = if workspace_lock.exists() {
        let (_, packages) = parse_cargo_lock(&workspace_lock)?;
        Some(build_dependency_graph(&packages))
    } else {
        None
    };

    // Run initial cargo check to establish baseline
    let initial_check = std::process::Command::new("cargo")
        .arg("check")
        .current_dir(&workspace_root)
        .output();
    let initial_build_success = matches!(initial_check, Ok(output) if output.status.success());

    let mut stats = FeatureNormalizationStats::default();
    let mut original_contents: HashMap<PathBuf, String> = HashMap::new();

    // Store original contents and process each member
    for member_dir in &members {
        let member_toml = member_dir.join("Cargo.toml");
        if !member_toml.exists() {
            continue;
        }

        let content = std::fs::read_to_string(&member_toml)?;
        original_contents.insert(member_toml.clone(), content.clone());
    }

    // Process each member crate
    for member_dir in members {
        let member_toml = member_dir.join("Cargo.toml");
        if !member_toml.exists() {
            continue;
        }

        stats.crates_examined += 1;

        let content = std::fs::read_to_string(&member_toml)?;
        let mut doc = content.parse::<DocumentMut>()?;

        let features_modified = normalize_crate_features(&mut doc, &mut stats, &lockfile_graph)?;
        let sections_modified = sort_cargo_toml_sections(&mut doc)?;

        if features_modified || sections_modified {
            stats.crates_modified += 1;
            stats.edited_files.insert(member_toml.clone());
            std::fs::write(&member_toml, doc.to_string())?;
        }
    }

    // Run final cargo check
    let final_check = std::process::Command::new("cargo")
        .arg("check")
        .current_dir(&workspace_root)
        .output();
    let final_build_success = matches!(final_check, Ok(ref output) if output.status.success());

    // Rollback if build broke
    if initial_build_success && !final_build_success {
        eprintln!("Error: Workspace built before normalization but fails after.");
        if let Ok(output) = &final_check {
            eprintln!(
                "Cargo check stderr:\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        eprintln!("Rolling back all changes...");
        for (path, content) in &original_contents {
            std::fs::write(path, content)?;
        }
        anyhow::bail!("Normalization broke the build. All changes have been reverted.");
    }

    eprintln!("\nSummary:");
    eprintln!("  Crates examined: {}", stats.crates_examined);
    eprintln!("  Crates modified: {}", stats.crates_modified);
    eprintln!("  Features created: {}", stats.features_created);
    eprintln!("  Features deleted: {}", stats.features_deleted);
    eprintln!(
        "  Invalid dependency references removed: {}",
        stats.invalid_refs_removed
    );
    eprintln!(
        "  Default features created: {}",
        stats.default_features_created
    );

    Ok(())
}

#[derive(Default)]
struct FeatureNormalizationStats {
    crates_examined: usize,
    crates_modified: usize,
    features_created: usize,
    features_deleted: usize,
    invalid_refs_removed: usize,
    default_features_created: usize,
    edited_files: HashSet<PathBuf>,
}

#[derive(Debug)]
struct DependencyInfo {
    cargo_toml_key: String,
    #[allow(dead_code)]
    normalized_name: String,
    optional: bool,
}

/// Normalize features for a single crate
fn normalize_crate_features(
    doc: &mut DocumentMut,
    stats: &mut FeatureNormalizationStats,
    lockfile_graph: &Option<HashMap<String, HashSet<String>>>,
) -> Result<bool> {
    // Collect all dependencies from all sections
    let all_deps = collect_all_dependencies(doc)?;

    if all_deps.is_empty() {
        return Ok(false);
    }

    // Parse existing features
    let mut features = parse_existing_features(doc);
    let features_before_changes = features.keys().cloned().collect::<HashSet<_>>();

    // Clean up invalid feature dependencies
    let invalid_removed = clean_invalid_feature_deps(&mut features, &all_deps);
    stats.invalid_refs_removed += invalid_removed;

    // Create features for optional dependencies without features
    let optional_dep_names: HashSet<String> = all_deps
        .iter()
        .filter_map(|(name, info)| {
            if info.optional {
                Some(name.clone())
            } else {
                None
            }
        })
        .collect();

    // Map from normalized name (underscores) to cargo key (may have hyphens)
    let optional_dep_cargo_keys: HashMap<String, String> = all_deps
        .iter()
        .filter_map(|(name, info)| {
            if info.optional {
                Some((name.clone(), info.cargo_toml_key.clone()))
            } else {
                None
            }
        })
        .collect();

    for dep_name in &optional_dep_names {
        if !features.contains_key(dep_name) {
            let dep_info = &all_deps[dep_name];
            let feature_value = format!("dep:{}", dep_info.cargo_toml_key);
            features.insert(dep_name.clone(), vec![feature_value]);
        }
    }

    // Add transitive feature dependencies based on lockfile graph
    // If optional dep A depends on optional dep B (per Cargo.lock), add B to A's
    // feature
    if let Some(graph) = lockfile_graph {
        for dep_name in &optional_dep_names {
            // Get the cargo key for this dep (may have hyphens)
            let cargo_key = optional_dep_cargo_keys
                .get(dep_name)
                .map(|s| s.as_str())
                .unwrap_or(dep_name);

            // Look up dependencies in the lockfile graph (uses actual crate names with
            // hyphens)
            if let Some(transitive_deps) = graph.get(cargo_key) {
                // Find which of these are also optional deps in this crate
                let feature_deps: Vec<String> = transitive_deps
                    .iter()
                    .filter_map(|trans_dep| {
                        // Normalize the transitive dep name for comparison
                        let normalized = normalize_dep_name_for_feature(trans_dep);
                        if optional_dep_names.contains(&normalized) && &normalized != dep_name {
                            Some(normalized)
                        } else {
                            None
                        }
                    })
                    .collect();

                // Add these as feature dependencies
                if let Some(feature_list) = features.get_mut(dep_name) {
                    for feat_dep in feature_deps {
                        if !feature_list.contains(&feat_dep) {
                            feature_list.push(feat_dep);
                        }
                    }
                }
            }
        }
    }

    // Identify new features we're adding (before we add default)
    let new_optional_features: HashSet<String> = features
        .keys()
        .filter(|k| !features_before_changes.contains(*k))
        .cloned()
        .collect();

    // Handle default feature
    let mut creating_default = false;
    if !new_optional_features.is_empty() && !features.contains_key("default") {
        let mut default_deps: Vec<String> = features_before_changes.iter().cloned().collect();
        default_deps.sort();
        features.insert("default".to_string(), default_deps);
        stats.default_features_created += 1;
        creating_default = true;
    }

    // Calculate actual deletions
    let deleted_features: Vec<String> = features_before_changes
        .iter()
        .filter(|k| !features.contains_key(*k))
        .cloned()
        .collect();

    // Update statistics
    stats.features_created += new_optional_features.len();
    if creating_default {
        stats.features_created += 1; // Count the default feature as new
    }
    stats.features_deleted += deleted_features.len();

    // Update the document if there are features to write
    if !features.is_empty() || doc.get("features").is_some() {
        update_features_section(doc, features, &optional_dep_names)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Collect all dependencies from all sections of Cargo.toml
fn collect_all_dependencies(doc: &DocumentMut) -> Result<HashMap<String, DependencyInfo>> {
    let mut deps = HashMap::new();

    for section in &["dependencies", "dev-dependencies", "build-dependencies"] {
        if let Some(deps_table) = doc.get(section).and_then(|s| s.as_table()) {
            for (key, value) in deps_table.iter() {
                let optional = is_optional_dependency(value);
                let normalized = normalize_dep_name_for_feature(key);

                deps.insert(normalized.clone(), DependencyInfo {
                    cargo_toml_key: key.to_string(),
                    normalized_name: normalized,
                    optional,
                });
            }
        }
    }

    Ok(deps)
}

/// Check if a dependency is marked as optional
fn is_optional_dependency(value: &Item) -> bool {
    get_optional_flag(value).unwrap_or(false)
}

/// Extract optional flag from a dependency value
fn get_optional_flag(value: &Item) -> Option<bool> {
    match value {
        Item::Value(Value::InlineTable(table)) => table.get("optional").and_then(|v| v.as_bool()),
        Item::Table(table) => table.get("optional").and_then(|v| v.as_bool()),
        _ => None,
    }
}

/// Normalize dependency name to feature name (hyphens -> underscores)
fn normalize_dep_name_for_feature(cargo_toml_name: &str) -> String {
    cargo_toml_name.replace('-', "_")
}

/// Parse existing features from the document
fn parse_existing_features(doc: &DocumentMut) -> HashMap<String, Vec<String>> {
    let mut features = HashMap::new();

    if let Some(features_table) = doc.get("features").and_then(|f| f.as_table()) {
        for (key, value) in features_table.iter() {
            let mut feature_deps = Vec::new();

            if let Some(array) = value.as_array() {
                for item in array.iter() {
                    if let Some(s) = item.as_str() {
                        feature_deps.push(s.to_string());
                    }
                }
            }

            features.insert(key.to_string(), feature_deps);
        }
    }

    features
}

/// Clean up invalid feature dependencies (those referencing non-existent
/// optional deps)
///
/// Only deletes a feature if:
/// 1. It was auto-generated (had `dep:same_name` where name matches feature
///    name)
/// 2. That dep is no longer optional (removed or made non-optional)
///
/// Does NOT delete intentionally empty features like `_implicit_all = []`
fn clean_invalid_feature_deps(
    features: &mut HashMap<String, Vec<String>>,
    all_deps: &HashMap<String, DependencyInfo>,
) -> usize {
    let mut total_removed = 0;

    // First pass: identify features that should be deleted
    // A feature should be deleted if it had `dep:feature_name` and that dep is gone
    let mut features_to_delete: HashSet<String> = HashSet::new();

    for (feature_name, feature_deps) in features.iter() {
        // Check if this feature has `dep:feature_name` (normalized)
        let has_self_dep = feature_deps.iter().any(|dep| {
            if let Some(stripped) = dep.strip_prefix("dep:") {
                normalize_dep_name_for_feature(stripped) == *feature_name
            } else {
                false
            }
        });

        // If it had a self-referencing dep:, check if that dep is still optional
        if has_self_dep {
            let dep_still_optional = all_deps
                .get(feature_name)
                .map(|info| info.optional)
                .unwrap_or(false);

            if !dep_still_optional {
                // The dep is no longer optional - mark feature for deletion
                features_to_delete.insert(feature_name.clone());
            }
        }
    }

    // Cascade: if feature A references feature B, and B is being deleted,
    // we need to remove that reference. Iterate until stable.
    loop {
        let mut changed = false;

        for (feature_name, feature_deps) in features.iter_mut() {
            if features_to_delete.contains(feature_name) {
                continue;
            }

            let before_len = feature_deps.len();

            // Remove references to features being deleted
            feature_deps.retain(|dep| !features_to_delete.contains(dep));

            // Remove invalid dep: references (dep no longer optional)
            // Note: NAME/feature refs are valid for non-optional deps, only remove dep:NAME
            // refs
            feature_deps.retain(|dep_ref| {
                if let Some(stripped) = dep_ref.strip_prefix("dep:") {
                    // This is a dep:NAME reference - check if dep is still optional
                    let dep_name = normalize_dep_name_for_feature(stripped);
                    all_deps
                        .get(&dep_name)
                        .map(|info| info.optional)
                        .unwrap_or(false)
                } else {
                    // NAME/feature refs or bare feature names - keep them
                    true
                }
            });

            let removed = before_len - feature_deps.len();
            if removed > 0 {
                total_removed += removed;
                changed = true;
            }

            // If this was an auto-generated feature and is now empty, mark for deletion
            let was_auto_generated = all_deps.contains_key(feature_name);
            if feature_deps.is_empty()
                && was_auto_generated
                && !features_to_delete.contains(feature_name)
            {
                // Check if it originally had a self dep
                features_to_delete.insert(feature_name.clone());
                changed = true;
            }
        }

        if !changed {
            break;
        }
    }

    // Delete marked features
    for feature_name in &features_to_delete {
        features.remove(feature_name);
    }

    total_removed
}

/// Extract dependency name from a dependency reference (dep:NAME or
/// NAME/feature)
fn extract_dep_reference(dep_ref: &str) -> Option<String> {
    if let Some(stripped) = dep_ref.strip_prefix("dep:") {
        Some(normalize_dep_name_for_feature(stripped))
    } else {
        dep_ref
            .find('/')
            .map(|slash_pos| normalize_dep_name_for_feature(&dep_ref[..slash_pos]))
    }
}

/// Update the features section in the document
fn update_features_section(
    doc: &mut DocumentMut,
    features: HashMap<String, Vec<String>>,
    internal_feature_names: &HashSet<String>,
) -> Result<()> {
    // Case 1: Features section already exists - modify in place
    if let Some(features_table) = doc.get_mut("features").and_then(|f| f.as_table_mut()) {
        // Remove features that are no longer needed
        let current_keys: Vec<String> = features_table.iter().map(|(k, _)| k.to_string()).collect();
        for key in current_keys {
            if !features.contains_key(&key) {
                features_table.remove(&key);
            }
        }

        // Add/update features
        for (feature_name, feature_deps) in features.iter() {
            // Sort the dependencies within each feature
            let mut sorted_deps = feature_deps.clone();
            sorted_deps.sort_by_key(|dep| feature_dep_sort_key(dep, internal_feature_names));

            let mut array = toml_edit::Array::new();
            for dep in sorted_deps {
                array.push(dep);
            }
            features_table.insert(feature_name, toml_edit::Item::Value(Value::Array(array)));
        }

        // Sort in place (reuse workspace_deps pattern)
        // Extract all entries, sort them, remove all, re-insert in order
        let mut sorted_entries: Vec<(String, Item)> = features_table
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect();

        sorted_entries.sort_by(|a, b| match (&a.0[..], &b.0[..]) {
            ("default", "default") => std::cmp::Ordering::Equal,
            ("default", _) => std::cmp::Ordering::Less,
            (_, "default") => std::cmp::Ordering::Greater,
            (a_key, b_key) => a_key.cmp(b_key),
        });

        let all_keys: Vec<String> = features_table.iter().map(|(k, _)| k.to_string()).collect();
        for key in all_keys {
            features_table.remove(&key);
        }
        for (key, value) in sorted_entries {
            features_table.insert(&key, value);
        }
    }
    // Case 2: No features section yet - create it
    else if !features.is_empty() {
        let mut features_table = toml_edit::Table::new();

        // Sort features before inserting
        let mut sorted_features: Vec<_> = features.into_iter().collect();
        sorted_features.sort_by(|a, b| match (&a.0[..], &b.0[..]) {
            ("default", "default") => std::cmp::Ordering::Equal,
            ("default", _) => std::cmp::Ordering::Less,
            (_, "default") => std::cmp::Ordering::Greater,
            (a_key, b_key) => a_key.cmp(b_key),
        });

        for (feature_name, mut feature_deps) in sorted_features {
            // Sort the dependencies within each feature
            feature_deps.sort_by_key(|dep| feature_dep_sort_key(dep, internal_feature_names));

            let mut array = toml_edit::Array::new();
            for dep in feature_deps {
                array.push(dep);
            }
            features_table.insert(&feature_name, toml_edit::Item::Value(Value::Array(array)));
        }

        doc.insert("features", toml_edit::Item::Table(features_table));
    }

    Ok(())
}

/// Sort key for feature dependencies
/// Order: internal bare names, external bare names, slash refs (/default
/// first), dep: refs
fn feature_dep_sort_key(dep: &str, internal_names: &HashSet<String>) -> (u32, u32, String) {
    if dep.contains('/') {
        // Slash refs: sort with /default first
        let has_default = dep.ends_with("/default");
        (2, if has_default { 0 } else { 1 }, dep.to_string())
    } else if dep.starts_with("dep:") {
        // dep: refs come last
        (3, 0, dep.to_string())
    } else {
        // Bare feature names: internal (matching optional deps) come first
        let is_internal = internal_names.contains(dep);
        (0, if is_internal { 0 } else { 1 }, dep.to_string())
    }
}

/// Sort top-level sections in Cargo.toml according to canonical Cargo ordering
fn sort_cargo_toml_sections(doc: &mut DocumentMut) -> Result<bool> {
    // Canonical ordering from Cargo documentation
    let section_order = [
        "cargo-features",
        "package",
        "lib",
        "bin",
        "example",
        "test",
        "bench",
        "dependencies",
        "dev-dependencies",
        "build-dependencies",
        "target",
        "badges",
        "features",
        "lints",
        "hints",
        "patch",
        "replace",
        "profile",
        "workspace",
    ];

    // Collect all current sections
    let current_keys: Vec<String> = doc.iter().map(|(k, _)| k.to_string()).collect();

    // Check if reordering is needed
    let needs_reordering = {
        let mut last_order_idx = -1i32;
        let mut needs_order = false;

        for key in &current_keys {
            let order_idx = section_order
                .iter()
                .position(|&s| s == key)
                .map(|i| i as i32)
                .unwrap_or(section_order.len() as i32);

            if order_idx < last_order_idx {
                needs_order = true;
                break;
            }
            last_order_idx = order_idx;
        }

        needs_order
    };

    if !needs_reordering {
        return Ok(false);
    }

    // Use set_position() to explicitly set the position of each section
    // This should affect the serialization order
    for key in &current_keys {
        let target_position = section_order
            .iter()
            .position(|&s| s == key)
            .unwrap_or(section_order.len());

        // Get mutable reference to the item and set its position
        if let Some(item) = doc.get_mut(key)
            && let Some(table) = item.as_table_mut()
        {
            table.set_position(target_position);
        }
    }

    Ok(true)
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
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let mut resolved = Vec::new();

    for pattern in members {
        let pattern_path = workspace_root.join(&pattern);
        let pattern_str = pattern_path.to_string_lossy();
        for entry in glob(&pattern_str).context("Failed to read glob pattern")? {
            let path = entry.context("Failed to read glob entry")?;
            if path.is_dir() {
                resolved.push(path);
            }
        }
    }

    Ok(resolved)
}
