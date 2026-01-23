//! File discovery and scanning
//! [impl _trace.files]

use {
    ignore::WalkBuilder,
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

    // Use WalkBuilder which respects .gitignore automatically
    // Include hidden files (for .claude/ etc) but exclude .git explicitly
    let walker = WalkBuilder::new(root)
        .hidden(false) // Include hidden files/directories like .claude/
        .git_ignore(true) // Respect .gitignore
        .git_exclude(true) // Respect .git/info/exclude
        .require_git(false) // Still work in non-git directories
        .build();

    for result in walker {
        let entry = match result {
            Ok(entry) => entry,
            Err(_) => continue,
        };

        let path = entry.path();

        // Skip directories
        if entry.file_type().map_or(true, |ft| ft.is_dir()) {
            continue;
        }

        // Skip .git directory explicitly (even though gitignore should handle it)
        if path.components().any(|c| c.as_os_str() == ".git") {
            continue;
        }

        // Only include:
        // 1. All .md files (**/*.md)
        // 2. All files under any src/ directory (**/src/**/*)
        let path_str = path.to_string_lossy();
        let is_markdown = path.extension().map_or(false, |ext| ext == "md");
        let is_in_src = path_str.contains("/src/");

        if is_markdown || is_in_src {
            if let Some(file) = read_file(&path.to_path_buf()) {
                files.push(file);
            }
        }
    }

    // Sort for deterministic ordering
    files.sort_by(|a, b| a.path.cmp(&b.path));

    files
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
