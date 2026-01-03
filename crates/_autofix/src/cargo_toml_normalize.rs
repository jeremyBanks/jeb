use {
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
    eprintln!("Running: Cargo.toml feature normalization");
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

        let features_modified = normalize_crate_features(&mut doc, &mut stats)?;
        // DISABLED: Section reordering doesn't work with toml_edit - it preserves
        // original section order even when removing and re-inserting. This is a
        // limitation of how toml_edit tracks position/formatting.
        // let sections_modified = sort_cargo_toml_sections(&mut doc)?;
        let sections_modified = false;

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
    let final_build_success = matches!(final_check, Ok(output) if output.status.success());

    // Rollback if build broke
    if initial_build_success && !final_build_success {
        eprintln!("Error: Workspace built before normalization but fails after.");
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

    for dep_name in optional_dep_names {
        if !features.contains_key(&dep_name) {
            let dep_info = &all_deps[&dep_name];
            let feature_value = format!("dep:{}", dep_info.cargo_toml_key);
            features.insert(dep_name, vec![feature_value]);
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
        update_features_section(doc, features)?;
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

/// Clean up invalid feature dependencies (those referencing non-existent deps)
fn clean_invalid_feature_deps(
    features: &mut HashMap<String, Vec<String>>,
    all_deps: &HashMap<String, DependencyInfo>,
) -> usize {
    let mut total_removed = 0;
    let mut features_to_delete = Vec::new();

    for (feature_name, feature_deps) in features.iter_mut() {
        let original_len = feature_deps.len();

        // Keep only valid dependency references and non-dependency refs
        feature_deps.retain(|dep_ref| {
            if let Some(dep_name) = extract_dep_reference(dep_ref) {
                // This is a dependency reference (dep:NAME or NAME/feature)
                all_deps.contains_key(&dep_name)
            } else {
                // Not a dependency reference (e.g., another feature), keep it
                true
            }
        });

        let removed = original_len - feature_deps.len();
        total_removed += removed;

        // Mark feature for deletion if it became empty
        if feature_deps.is_empty() && original_len > 0 {
            features_to_delete.push(feature_name.clone());
        }
    }

    // Remove features that became empty
    for feature_name in features_to_delete {
        features.remove(&feature_name);
    }

    total_removed
}

/// Extract dependency name from a dependency reference (dep:NAME or
/// NAME/feature)
fn extract_dep_reference(dep_ref: &str) -> Option<String> {
    if let Some(stripped) = dep_ref.strip_prefix("dep:") {
        Some(normalize_dep_name_for_feature(stripped))
    } else if let Some(slash_pos) = dep_ref.find('/') {
        Some(normalize_dep_name_for_feature(&dep_ref[..slash_pos]))
    } else {
        None
    }
}

/// Update the features section in the document
fn update_features_section(
    doc: &mut DocumentMut,
    features: HashMap<String, Vec<String>>,
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
            sorted_deps.sort_by_key(|dep| feature_dep_sort_key(dep));

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
            feature_deps.sort_by_key(|dep| feature_dep_sort_key(dep));

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

/// Sort key for feature dependencies (bare names first, then dep:/slash refs)
fn feature_dep_sort_key(dep: &str) -> (u32, String) {
    if dep.contains('/') {
        // Slash refs: sort with /default first
        let has_default = dep.ends_with("/default");
        (if has_default { 1 } else { 2 }, dep.to_string())
    } else if dep.starts_with("dep:") {
        // dep: refs
        (3, dep.to_string())
    } else {
        // Bare feature names
        (0, dep.to_string())
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

    // Collect entries with their values (preserving decoration/comments)
    let mut entries: Vec<(String, Item)> = current_keys
        .iter()
        .filter_map(|key| doc.get(key).map(|value| (key.clone(), value.clone())))
        .collect();

    // Sort by canonical order
    entries.sort_by_key(|(key, _)| {
        section_order
            .iter()
            .position(|&s| s == key)
            .unwrap_or(section_order.len())
    });

    // Remove all sections
    for key in &current_keys {
        doc.remove(key);
    }

    // Re-insert in sorted order
    for (key, value) in entries {
        doc.insert(&key, value);
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
