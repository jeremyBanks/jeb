//! Shared Cargo.lock parsing utilities

use {
    anyhow::Result,
    std::{
        collections::{
            HashMap,
            HashSet,
        },
        path::Path,
    },
    toml_edit::DocumentMut,
};

/// Represents a package entry from Cargo.lock
#[derive(Debug, Clone)]
pub struct LockPackage {
    pub name: String,
    pub version: String,
    pub source: Option<String>,
    pub checksum: Option<String>,
    pub dependencies: Vec<String>,
}

/// Parse Cargo.lock (version 4 format) into a list of packages
pub fn parse_cargo_lock(lock_path: &Path) -> Result<(i64, Vec<LockPackage>)> {
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
/// Returns a map from package name to set of package names it depends on.
pub fn build_dependency_graph(packages: &[LockPackage]) -> HashMap<String, HashSet<String>> {
    let mut graph: HashMap<String, HashSet<String>> = HashMap::new();

    for pkg in packages {
        let deps = graph.entry(pkg.name.clone()).or_default();
        for dep in &pkg.dependencies {
            deps.insert(dep.clone());
        }
    }

    graph
}

/// Get direct dependencies of a package from the lockfile graph.
/// Returns None if the package is not in the graph.
pub fn get_direct_dependencies(
    pkg_name: &str,
    graph: &HashMap<String, HashSet<String>>,
) -> Option<HashSet<String>> {
    graph.get(pkg_name).cloned()
}

/// Get all transitive dependencies starting from a set of root package names.
/// Uses BFS to traverse the dependency graph.
pub fn get_transitive_dependencies(
    roots: &HashSet<String>,
    graph: &HashMap<String, HashSet<String>>,
) -> HashSet<String> {
    let mut needed: HashSet<String> = HashSet::new();
    let mut queue: Vec<String> = roots.iter().cloned().collect();

    while let Some(pkg_name) = queue.pop() {
        if needed.contains(&pkg_name) {
            continue;
        }
        needed.insert(pkg_name.clone());
        if let Some(deps) = graph.get(&pkg_name) {
            for dep in deps {
                if !needed.contains(dep) {
                    queue.push(dep.clone());
                }
            }
        }
    }

    needed
}
