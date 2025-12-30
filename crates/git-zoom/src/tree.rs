//! Tree manipulation functions.

use crate::git;

/// Replace a subtree at a given path within a tree.
/// The path can be nested (e.g., "src/lib/core").
/// If intermediate components are blobs, they are replaced with trees.
pub fn replace_subtree(base_tree: &str, path: &str, new_subtree: &str) -> git::Result<String> {
    let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    if parts.is_empty() {
        return Ok(new_subtree.to_string());
    }
    replace_subtree_recursive(base_tree, &parts, new_subtree)
}

fn replace_subtree_recursive(
    tree: &str,
    path_parts: &[&str],
    new_subtree: &str,
) -> git::Result<String> {
    if path_parts.is_empty() {
        return Ok(new_subtree.to_string());
    }

    let target_name = path_parts[0];
    let remaining_path = &path_parts[1..];

    // List current tree entries
    let entries = git::ls_tree(tree)?;

    // Build new entries, replacing/adding the target
    let mut new_entries: Vec<(String, String, String, String)> = Vec::new();
    let mut found = false;

    for (mode, obj_type, hash, name) in entries {
        if name == target_name {
            found = true;
            if remaining_path.is_empty() {
                // Replace this entry entirely with new_subtree
                new_entries.push((
                    "040000".to_string(),
                    "tree".to_string(),
                    new_subtree.to_string(),
                    name,
                ));
            } else {
                // Need to recurse; if existing entry is a blob, replace with empty tree first
                let subtree_hash = if obj_type == "blob" {
                    git::empty_tree()?
                } else {
                    hash
                };
                let new_hash = replace_subtree_recursive(&subtree_hash, remaining_path, new_subtree)?;
                new_entries.push(("040000".to_string(), "tree".to_string(), new_hash, name));
            }
        } else {
            new_entries.push((mode, obj_type, hash, name));
        }
    }

    if !found {
        // Need to create new entry (and possibly intermediate trees)
        if remaining_path.is_empty() {
            new_entries.push((
                "040000".to_string(),
                "tree".to_string(),
                new_subtree.to_string(),
                target_name.to_string(),
            ));
        } else {
            // Create empty tree, recurse, then add
            let empty = git::empty_tree()?;
            let new_hash = replace_subtree_recursive(&empty, remaining_path, new_subtree)?;
            new_entries.push((
                "040000".to_string(),
                "tree".to_string(),
                new_hash,
                target_name.to_string(),
            ));
        }
    }

    // Create new tree from entries
    git::mktree(&new_entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::process::Command;
    use tempfile::TempDir;

    fn setup_test_repo() -> TempDir {
        let dir = TempDir::new().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        Command::new("git")
            .args(["init"])
            .current_dir(dir.path())
            .output()
            .unwrap();

        Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(dir.path())
            .output()
            .unwrap();

        Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(dir.path())
            .output()
            .unwrap();

        dir
    }

    #[test]
    fn test_replace_subtree_simple() {
        let dir = setup_test_repo();

        // Create initial structure: src/lib/foo.txt
        fs::create_dir_all(dir.path().join("src/lib")).unwrap();
        fs::write(dir.path().join("src/lib/foo.txt"), "original").unwrap();
        fs::write(dir.path().join("root.txt"), "root").unwrap();

        Command::new("git")
            .args(["add", "."])
            .current_dir(dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "initial"])
            .current_dir(dir.path())
            .output()
            .unwrap();

        let base_tree = git::rev_parse("HEAD^{tree}").unwrap();

        // Create a new subtree with different content
        fs::write(dir.path().join("src/lib/foo.txt"), "modified").unwrap();
        fs::write(dir.path().join("src/lib/bar.txt"), "new file").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(dir.path())
            .output()
            .unwrap();

        // Get the new src/lib tree
        let output = Command::new("git")
            .args(["write-tree", "--prefix=src/lib"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        let new_subtree = String::from_utf8_lossy(&output.stdout).trim().to_string();

        // Replace src/lib in base_tree with new_subtree
        let result_tree = replace_subtree(&base_tree, "src/lib", &new_subtree).unwrap();

        // Verify: result_tree should have root.txt and src/lib with new content
        let entries = git::ls_tree(&result_tree).unwrap();
        assert_eq!(entries.len(), 2); // root.txt and src

        // Check that src/lib has the new content
        let src_tree = git::tree_at_path(&result_tree, "src").unwrap().unwrap();
        let src_entries = git::ls_tree(&src_tree).unwrap();
        assert_eq!(src_entries.len(), 1); // lib
        assert_eq!(src_entries[0].3, "lib");
        assert_eq!(src_entries[0].2, new_subtree);
    }

    #[test]
    fn test_replace_subtree_create_path() {
        let dir = setup_test_repo();

        // Create initial structure with just root.txt
        fs::write(dir.path().join("root.txt"), "root").unwrap();

        Command::new("git")
            .args(["add", "."])
            .current_dir(dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "initial"])
            .current_dir(dir.path())
            .output()
            .unwrap();

        let base_tree = git::rev_parse("HEAD^{tree}").unwrap();

        // Create a new subtree
        fs::create_dir_all(dir.path().join("newdir")).unwrap();
        fs::write(dir.path().join("newdir/new.txt"), "new content").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(dir.path())
            .output()
            .unwrap();

        let output = Command::new("git")
            .args(["write-tree", "--prefix=newdir"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        let new_subtree = String::from_utf8_lossy(&output.stdout).trim().to_string();

        // Replace non-existent path src/lib with new_subtree
        let result_tree = replace_subtree(&base_tree, "src/lib", &new_subtree).unwrap();

        // Verify: result_tree should have root.txt and src/lib
        let entries = git::ls_tree(&result_tree).unwrap();
        assert_eq!(entries.len(), 2); // root.txt and src

        // Check src/lib exists and has new content
        let lib_tree = git::tree_at_path(&result_tree, "src/lib").unwrap().unwrap();
        assert_eq!(lib_tree, new_subtree);
    }
}
