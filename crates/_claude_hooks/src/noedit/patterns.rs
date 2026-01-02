use eyre::{Context, ContextCompat, Result};
use git2::{Commit, Repository};
use ignore::gitignore::GitignoreBuilder;
use std::path::Path;
use walkdir::WalkDir;

use super::git_ops::read_noedit_files_from_commit;

/// Matcher for .noedit patterns
pub struct NoeditMatcher {
    gitignore: ignore::gitignore::Gitignore,
}

impl NoeditMatcher {
    /// Create a matcher from .noedit files in the working tree
    pub fn from_working_tree(cwd: &Path) -> Result<Self> {
        let mut builder = GitignoreBuilder::new(cwd);

        // Add implicit rule: .noedit itself is always protected unless negated
        builder
            .add_line(None, ".noedit")
            .context("Failed to add implicit .noedit pattern")?;

        // Find all .noedit files in working tree
        for entry in WalkDir::new(cwd).follow_links(false) {
            let entry = entry.context("Failed to read directory entry")?;

            if entry.file_name() == ".noedit" {
                let path = entry.path();
                let metadata = std::fs::metadata(path)
                    .context("Failed to read .noedit file metadata")?;

                // Special case: empty .noedit matches everything in this directory
                if metadata.len() == 0 {
                    let dir = path
                        .parent()
                        .context("Failed to get parent directory of .noedit")?;
                    builder
                        .add_line(Some(dir.to_path_buf()), "**/*")
                        .context("Failed to add empty .noedit pattern")?;
                } else {
                    builder.add(path);
                }
            }
        }

        let gitignore = builder.build().context("Failed to build gitignore")?;
        Ok(Self { gitignore })
    }

    /// Create a matcher from .noedit files in a specific commit
    pub fn from_commit(repo: &Repository, commit: &Commit) -> Result<Self> {
        let workdir = repo
            .workdir()
            .context("Repository has no working directory")?;
        let mut builder = GitignoreBuilder::new(workdir);

        // Add implicit rule: .noedit itself is always protected unless negated
        builder
            .add_line(None, ".noedit")
            .context("Failed to add implicit .noedit pattern")?;

        // Read all .noedit files from commit
        let noedit_files = read_noedit_files_from_commit(repo, commit)?;

        for noedit_file in noedit_files {
            // Special case: empty .noedit matches everything
            if noedit_file.content.is_empty() {
                let dir = noedit_file
                    .path
                    .parent()
                    .map(|p| workdir.join(p))
                    .unwrap_or_else(|| workdir.to_path_buf());

                builder
                    .add_line(Some(dir), "**/*")
                    .context("Failed to add empty .noedit pattern from commit")?;
            } else {
                // Parse content as .gitignore patterns
                let dir = noedit_file
                    .path
                    .parent()
                    .map(|p| workdir.join(p))
                    .unwrap_or_else(|| workdir.to_path_buf());

                for line in noedit_file.content.lines() {
                    builder
                        .add_line(Some(dir.clone()), line)
                        .context("Failed to add .noedit pattern line from commit")?;
                }
            }
        }

        let gitignore = builder.build().context("Failed to build gitignore")?;
        Ok(Self { gitignore })
    }

    /// Check if a path matches any .noedit pattern
    pub fn matches(&self, path: &Path) -> bool {
        let matched = self.gitignore.matched(path, path.is_dir());
        matched.is_ignore()
    }

    /// Check if a path (potentially absolute) matches, converting to relative if needed
    pub fn matches_path(&self, base: &Path, path: &Path) -> bool {
        let relative_path = if path.is_absolute() {
            path.strip_prefix(base).unwrap_or(path)
        } else {
            path
        };
        self.matches(relative_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_basic_pattern_matching() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let noedit_path = temp_dir.path().join(".noedit");
        fs::write(&noedit_path, "*.secret\n")?;

        let matcher = NoeditMatcher::from_working_tree(temp_dir.path())?;

        assert!(matcher.matches(Path::new("test.secret")));
        assert!(!matcher.matches(Path::new("test.txt")));

        Ok(())
    }

    #[test]
    fn test_empty_noedit_matches_everything() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let noedit_path = temp_dir.path().join(".noedit");
        fs::write(&noedit_path, "")?;

        let matcher = NoeditMatcher::from_working_tree(temp_dir.path())?;

        assert!(matcher.matches(Path::new("anything.txt")));
        assert!(matcher.matches(Path::new("dir/file.rs")));

        Ok(())
    }

    #[test]
    fn test_implicit_noedit_protection() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let noedit_path = temp_dir.path().join(".noedit");
        fs::write(&noedit_path, "*.tmp\n")?;

        let matcher = NoeditMatcher::from_working_tree(temp_dir.path())?;

        // .noedit itself should be protected
        assert!(matcher.matches(Path::new(".noedit")));

        Ok(())
    }
}
