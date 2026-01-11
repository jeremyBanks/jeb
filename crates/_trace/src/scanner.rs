//! File discovery and scanning
//! [impl _trace.files]

use {
    glob::{
        MatchOptions,
        glob_with,
    },
    std::{
        fs,
        path::PathBuf,
    },
};

/// A file with its content
#[derive(Debug)]
pub struct ScannedFile {
    pub path: PathBuf,
    pub content: String,
}

/// Scan for files matching the _trace patterns
/// [impl _trace.files.globs]
/// [impl _trace.files.language-agnostic]
pub fn scan_files(root: &PathBuf) -> Vec<ScannedFile> {
    let mut files = Vec::new();

    // Options to include hidden directories (like .claude/)
    let options = MatchOptions {
        require_literal_leading_dot: false,
        ..Default::default()
    };

    // Pattern 1: **/*.md (all markdown files, including hidden directories)
    let md_pattern = root.join("**/*.md");
    if let Ok(paths) = glob_with(md_pattern.to_str().unwrap_or(""), options) {
        for entry in paths.flatten() {
            // Skip .git directory
            if is_in_git_dir(&entry) {
                continue;
            }
            if let Some(file) = read_file(&entry) {
                files.push(file);
            }
        }
    }

    // Pattern 2: src/**/* (all files under src/)
    let src_pattern = root.join("src/**/*");
    if let Ok(paths) = glob_with(src_pattern.to_str().unwrap_or(""), options) {
        for entry in paths.flatten() {
            // Skip directories and .git
            if entry.is_dir() || is_in_git_dir(&entry) {
                continue;
            }
            if let Some(file) = read_file(&entry) {
                files.push(file);
            }
        }
    }

    // Sort for deterministic ordering
    files.sort_by(|a, b| a.path.cmp(&b.path));

    files
}

/// Check if a path is inside a .git directory
/// [impl _trace.files.globs]
fn is_in_git_dir(path: &PathBuf) -> bool {
    path.components().any(|c| c.as_os_str() == ".git")
}

/// Read a file if it's text content
fn read_file(path: &PathBuf) -> Option<ScannedFile> {
    // Try to read as UTF-8 text
    match fs::read_to_string(path) {
        Ok(content) => Some(ScannedFile {
            path: path.clone(),
            content,
        }),
        Err(_) => {
            // Skip binary files or files we can't read
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        std::env,
    };

    /// [test _trace.files]
    /// [test _trace.files.globs]
    #[test]
    fn test_scan_finds_spec() {
        let root = env::current_dir().unwrap();
        let files = scan_files(&root);

        // Should find SPEC.md at minimum (tests **/*.md pattern)
        assert!(
            files.iter().any(|f| f.path.ends_with("SPEC.md")),
            "Should find SPEC.md"
        );
    }

    /// [test _trace.files.globs]
    #[test]
    fn test_scan_finds_source_files() {
        let root = env::current_dir().unwrap();
        let files = scan_files(&root);

        // Should find source files under src/ (tests src/**/* pattern)
        assert!(
            files
                .iter()
                .any(|f| f.path.to_string_lossy().contains("src/")),
            "Should find files under src/"
        );
    }

    /// [test _trace.files.language-agnostic]
    #[test]
    fn test_scan_is_language_agnostic() {
        let root = env::current_dir().unwrap();
        let files = scan_files(&root);

        // Should find both .rs and .md files (language-agnostic)
        let has_rs = files
            .iter()
            .any(|f| f.path.extension().map(|e| e == "rs").unwrap_or(false));
        let has_md = files
            .iter()
            .any(|f| f.path.extension().map(|e| e == "md").unwrap_or(false));

        assert!(has_rs, "Should find .rs files");
        assert!(has_md, "Should find .md files");
    }
}
